pub mod db;
pub mod entities;
pub mod migration;

#[cfg(all(target_os = "linux", not(target_env = "msvc")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use axum::{
    body::Body,
    extract::{
        ws::{Message as AxumWsMessage, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{HeaderMap, Method, Response, StatusCode},
    response::{Html, IntoResponse},
    routing::{any, get, post},
    Json, Router,
};
use bytes::Bytes;
use chrono::Utc;
use entities::{channel_message, cli_instance, orchestration_channel};
use futures_util::{SinkExt, StreamExt};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    Set,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    path::Path as FsPath,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpStream, UnixStream},
    sync::{broadcast, RwLock},
    time::timeout,
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message as TungMessage},
};
use tracing::{error, info, warn};
use url::Url;
use uuid::Uuid;

const INDEX_HTML: &str = include_str!("../static/index.html");
const STYLE_CSS: &str = include_str!("../static/style.css");
const APP_JS: &str = include_str!("../static/app.js");

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub goose_http_url: String,
    pub goose_ws_url: String,
    pub goose_secret: Option<String>,
    pub codex_ws_url: String,
    pub codex_unix_socket: String,
    pub claude_rpc_url: String,
    pub antigravity_rpc_url: String,
    pub bridge_secret: Option<String>,
    pub default_cwd: String,
    pub http_client: reqwest::Client,
    pub channel_hubs: Arc<RwLock<HashMap<Uuid, broadcast::Sender<String>>>>,
}

impl AppState {
    pub async fn get_channel_sender(&self, channel_id: Uuid) -> broadcast::Sender<String> {
        {
            let map = self.channel_hubs.read().await;
            if let Some(tx) = map.get(&channel_id) {
                return tx.clone();
            }
        }
        let mut map = self.channel_hubs.write().await;
        map.entry(channel_id)
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(256);
                tx
            })
            .clone()
    }
}

#[derive(Serialize)]
struct ConfigResponse {
    goose_http_url: String,
    goose_ws_url: String,
    has_goose_secret: bool,
    codex_ws_url: String,
    codex_unix_socket: String,
    codex_unix_available: bool,
    claude_rpc_url: String,
    antigravity_rpc_url: String,
    default_cwd: String,
}

#[derive(Serialize, Clone)]
struct TargetHealth {
    online: bool,
    detail: String,
}

#[derive(Serialize)]
struct StatusResponse {
    goose: TargetHealth,
    codex_ws: TargetHealth,
    codex_unix: TargetHealth,
    claude: TargetHealth,
    antigravity: TargetHealth,
}

#[derive(Serialize)]
struct InstanceWithStatus {
    #[serde(flatten)]
    model: cli_instance::Model,
    online: bool,
    status_detail: String,
}

#[derive(Deserialize)]
struct CreateInstanceRequest {
    name: String,
    kind: String,
    transport: Option<String>,
    endpoint_url: String,
    bridge_url: Option<String>,
    secret_token: Option<String>,
    default_cwd: Option<String>,
    role: Option<String>,
    role_prompt: Option<String>,
}

#[derive(Deserialize)]
struct UpdateInstanceRequest {
    name: Option<String>,
    kind: Option<String>,
    transport: Option<String>,
    endpoint_url: Option<String>,
    bridge_url: Option<String>,
    secret_token: Option<String>,
    default_cwd: Option<String>,
    role: Option<String>,
    role_prompt: Option<String>,
}

#[derive(Deserialize)]
struct OnboardInstanceRequest {
    channel_name: Option<String>,
    custom_prompt: Option<String>,
    lang: Option<String>,
}

#[derive(Deserialize)]
struct RolePresetsQuery {
    lang: Option<String>,
}

#[derive(Deserialize)]
struct GitStatusQuery {
    cwd: Option<String>,
}

#[derive(Deserialize)]
struct GitCommitRequest {
    cwd: Option<String>,
    message: String,
    files: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct GitRevertRequest {
    cwd: Option<String>,
    file: Option<String>,
}

#[derive(Deserialize)]
struct InstanceAuthRequest {
    /// "status" | "device_start" | "api_key" | "oauth_callback"
    action: String,
    value: Option<String>,
}

#[derive(Deserialize)]
struct CreateChannelRequest {
    name: String,
    description: Option<String>,
    participant_ids: Option<Vec<Uuid>>,
    retention_days: Option<i32>,
    max_messages: Option<i32>,
}

#[derive(Deserialize)]
struct UpdateChannelRequest {
    name: Option<String>,
    description: Option<String>,
    participant_ids: Option<Vec<Uuid>>,
    retention_days: Option<i32>,
    max_messages: Option<i32>,
}

#[derive(Deserialize)]
struct PublishChannelMessageRequest {
    sender_instance_id: Option<Uuid>,
    sender_name: Option<String>,
    recipient_name: Option<String>,
    content: String,
    auto_dispatch: Option<bool>,
}

#[derive(Deserialize)]
struct CodexWsQuery {
    instance_id: Option<Uuid>,
    mode: Option<String>,
    target: Option<String>,
}

#[derive(Deserialize)]
struct GooseWsQuery {
    instance_id: Option<Uuid>,
    target: Option<String>,
    token: Option<String>,
}

#[derive(Deserialize)]
struct AgentAskRequest {
    instance_id: Option<Uuid>,
    agent: Option<String>,
    prompt: String,
    cwd: Option<String>,
    mode: Option<String>,
}

#[derive(Deserialize)]
struct AgentTmuxRequest {
    instance_id: Option<Uuid>,
    agent: Option<String>,
    lines: Option<u32>,
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "agent_hub=info,axum=info".into()),
        )
        .init();

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:///tmp/agent_hub.db?mode=rwc".to_string());

    let goose_http_url = env::var("GOOSE_HTTP_URL")
        .unwrap_or_else(|_| "http://goose:3284".to_string())
        .trim_end_matches('/')
        .to_string();

    let goose_ws_url = env::var("GOOSE_WS_URL")
        .or_else(|_| env::var("GOOSE_URL"))
        .unwrap_or_else(|_| "ws://goose:3284/acp".to_string());

    let goose_secret = env::var("GOOSE_SERVER__SECRET_KEY")
        .or_else(|_| env::var("GOOSE_SECRET_KEY"))
        .ok()
        .filter(|s| !s.trim().is_empty());

    let codex_ws_url =
        env::var("CODEX_WS_URL").unwrap_or_else(|_| "ws://codex:4500".to_string());
    let codex_unix_socket =
        env::var("CODEX_UNIX_SOCKET").unwrap_or_else(|_| "/run/codex/app.sock".to_string());

    let claude_rpc_url =
        env::var("CLAUDE_RPC_URL").unwrap_or_else(|_| "http://claude:3285/rpc".to_string());
    let antigravity_rpc_url = env::var("ANTIGRAVITY_RPC_URL")
        .unwrap_or_else(|_| "http://antigravity:3286/rpc".to_string());
    let bridge_secret = env::var("AGENT_BRIDGE_SECRET")
        .ok()
        .filter(|s| !s.trim().is_empty());

    let default_cwd = env::var("DEFAULT_CWD").unwrap_or_else(|_| "/srv".to_string());

    let db = match db::connect_and_migrate(
        &database_url,
        &codex_ws_url,
        &goose_ws_url,
        goose_secret.clone(),
        &claude_rpc_url,
        &antigravity_rpc_url,
        &default_cwd,
    )
    .await
    {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to connect and migrate database: {}", e);
            return;
        }
    };

    let http_client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to initialize HTTP client: {}", e);
            return;
        }
    };

    let state = Arc::new(AppState {
        db,
        goose_http_url,
        goose_ws_url,
        goose_secret,
        codex_ws_url,
        codex_unix_socket,
        claude_rpc_url,
        antigravity_rpc_url,
        bridge_secret,
        default_cwd,
        http_client,
        channel_hubs: Arc::new(RwLock::new(HashMap::new())),
    });

    let app = build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(
        "Simply Agent Hub v0.2.0 listening on http://{} (DB: {})",
        addr, database_url
    );

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind TCP listener on {}: {}", addr, e);
            return;
        }
    };
    if let Err(e) = axum::serve(listener, app).await {
        error!("Server terminated with error: {}", e);
    }
}

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/style.css", get(serve_css))
        .route("/app.js", get(serve_js))
        .route("/healthz", get(serve_healthz))
        .route("/ready", get(serve_ready))
        .route("/api/config", get(serve_config))
        .route("/api/status", get(serve_status))
        // CLI Instances CRUD, Roles, Native Terminal & Auth
        .route("/api/roles/presets", get(list_role_presets))
        .route(
            "/api/instances",
            get(list_instances).post(create_instance),
        )
        .route(
            "/api/instances/{id}",
            get(get_instance).put(update_instance).delete(delete_instance),
        )
        .route("/api/instances/{id}/auth", post(instance_auth_handler))
        .route("/api/instances/{id}/onboard", post(onboard_instance_handler))
        .route("/ws/instances/{id}/terminal", get(ws_instance_terminal_handler))
        .route("/api/instances/{id}/web", any(proxy_instance_web_root))
        .route(
            "/api/instances/{id}/web/{*path}",
            any(proxy_instance_web_path),
        )
        // Git Diff, Status, Commit & Revert
        .route("/api/git/status", get(handle_git_status))
        .route("/api/git/commit", post(handle_git_commit))
        .route("/api/git/revert", post(handle_git_revert))
        // Unified Agent Ask & Tmux
        .route("/api/agents/ask", post(handle_agent_ask))
        .route("/api/agents/tmux", post(handle_agent_tmux))
        // Multi-Agent Orchestration Channels & Bus
        .route(
            "/api/orchestration/channels",
            get(list_channels).post(create_channel),
        )
        .route(
            "/api/orchestration/channels/{id}",
            get(get_channel).put(update_channel).delete(delete_channel),
        )
        .route(
            "/api/orchestration/channels/{id}/messages",
            get(list_channel_messages).post(publish_channel_message),
        )
        .route(
            "/api/orchestration/channels/{id}/publish",
            post(publish_channel_message),
        )
        .route(
            "/api/orchestration/channels/{id}/prune",
            post(prune_channel_messages),
        )
        .route(
            "/api/orchestration/channels/{id}/export",
            get(export_channel_markdown),
        )
        .route("/ws/orchestration/{id}", get(ws_orchestration_handler))
        // Direct WebSocket & HTTP Bridges
        .route("/ws/codex", get(ws_codex_handler))
        .route("/ws/goose", get(ws_goose_handler))
        .route("/api/goose/{*path}", any(proxy_goose_http))
        .with_state(state)
}

async fn serve_index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn serve_css() -> impl IntoResponse {
    ([("content-type", "text/css; charset=utf-8")], STYLE_CSS)
}

async fn serve_js() -> impl IntoResponse {
    (
        [("content-type", "application/javascript; charset=utf-8")],
        APP_JS,
    )
}

async fn serve_healthz() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

async fn serve_ready(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match cli_instance::Entity::find().all(&state.db).await {
        Ok(list) => (
            StatusCode::OK,
            Json(json!({ "status": "ready", "instances": list.len() })),
        ),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "status": "error", "detail": e.to_string() })),
        ),
    }
}

async fn serve_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let codex_unix_available = FsPath::new(&state.codex_unix_socket).exists();
    Json(ConfigResponse {
        goose_http_url: state.goose_http_url.clone(),
        goose_ws_url: state.goose_ws_url.clone(),
        has_goose_secret: state.goose_secret.is_some(),
        codex_ws_url: state.codex_ws_url.clone(),
        codex_unix_socket: state.codex_unix_socket.clone(),
        codex_unix_available,
        claude_rpc_url: state.claude_rpc_url.clone(),
        antigravity_rpc_url: state.antigravity_rpc_url.clone(),
        default_cwd: state.default_cwd.clone(),
    })
}

async fn probe_endpoint(transport: &str, raw_url: &str, default_port: u16) -> TargetHealth {
    if transport == "unix" || raw_url.starts_with('/') || raw_url.starts_with("unix://") {
        let sock_path = raw_url.trim_start_matches("unix://");
        if FsPath::new(sock_path).exists() {
            match timeout(Duration::from_millis(450), UnixStream::connect(sock_path)).await {
                Ok(Ok(_)) => TargetHealth {
                    online: true,
                    detail: format!("Socket ready ({})", sock_path),
                },
                Ok(Err(e)) => TargetHealth {
                    online: false,
                    detail: format!("Socket error: {}", e),
                },
                Err(_) => TargetHealth {
                    online: false,
                    detail: "Socket timeout".into(),
                },
            }
        } else {
            TargetHealth {
                online: false,
                detail: format!("Not mounted ({})", sock_path),
            }
        }
    } else {
        let parsed = match Url::parse(raw_url) {
            Ok(u) => u,
            Err(e) => {
                return TargetHealth {
                    online: false,
                    detail: format!("Invalid URL: {}", e),
                }
            }
        };
        let host = match parsed.host_str() {
            Some(h) => h,
            None => {
                return TargetHealth {
                    online: false,
                    detail: "Missing host".into(),
                }
            }
        };
        let port = parsed.port_or_known_default().unwrap_or(default_port);
        let target = format!("{}:{}", host, port);

        match timeout(Duration::from_millis(600), TcpStream::connect(&target)).await {
            Ok(Ok(_)) => TargetHealth {
                online: true,
                detail: format!("Reachable ({})", target),
            },
            Ok(Err(e)) => TargetHealth {
                online: false,
                detail: format!("Unreachable ({}): {}", target, e),
            },
            Err(_) => TargetHealth {
                online: false,
                detail: format!("Timeout ({})", target),
            },
        }
    }
}

async fn serve_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (goose, codex_ws, codex_unix, claude, antigravity) = tokio::join!(
        probe_endpoint("ws", &state.goose_ws_url, 3284),
        probe_endpoint("ws", &state.codex_ws_url, 4500),
        probe_endpoint("unix", &state.codex_unix_socket, 0),
        probe_endpoint("rpc", &state.claude_rpc_url, 3285),
        probe_endpoint("rpc", &state.antigravity_rpc_url, 3286),
    );

    Json(StatusResponse {
        goose,
        codex_ws,
        codex_unix,
        claude,
        antigravity,
    })
}

// ============================================================================
// CLI INSTANCES CRUD & AUTHENTICATION API (/api/instances)
// ============================================================================
async fn list_instances(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match cli_instance::Entity::find()
        .order_by_asc(cli_instance::Column::CreatedAt)
        .all(&state.db)
        .await
    {
        Ok(models) => {
            let mut enriched = Vec::with_capacity(models.len());
            for m in models {
                let default_port = match m.kind.as_str() {
                    "codex" => 4500,
                    "goose" => 3284,
                    "claude" => 3285,
                    "antigravity" => 3286,
                    _ => 8080,
                };
                let health = probe_endpoint(&m.transport, &m.endpoint_url, default_port).await;
                enriched.push(InstanceWithStatus {
                    model: m,
                    online: health.online,
                    status_detail: health.detail,
                });
            }
            (StatusCode::OK, Json(json!(enriched)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn get_instance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match cli_instance::Entity::find_by_id(id).one(&state.db).await {
        Ok(Some(m)) => {
            let health = probe_endpoint(&m.transport, &m.endpoint_url, 4500).await;
            (
                StatusCode::OK,
                Json(json!(InstanceWithStatus {
                    model: m,
                    online: health.online,
                    status_detail: health.detail,
                })),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Instance not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn create_instance(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateInstanceRequest>,
) -> impl IntoResponse {
    let now = Utc::now();
    let kind = req.kind.trim().to_lowercase();
    let default_transport = match kind.as_str() {
        "codex" | "goose" => "ws",
        _ => "rpc",
    };
    let transport = req
        .transport
        .unwrap_or_else(|| default_transport.to_string());

    let bridge_url = req.bridge_url.or_else(|| {
        if transport == "rpc" {
            Some(req.endpoint_url.clone())
        } else if let Ok(u) = Url::parse(&req.endpoint_url) {
            let host = u.host_str().unwrap_or("localhost");
            let bport = match kind.as_str() {
                "codex" => 3287,
                "goose" => 3288,
                "claude" => 3285,
                "antigravity" => 3286,
                _ => 3285,
            };
            Some(format!("http://{}:{}/rpc", host, bport))
        } else {
            None
        }
    });

    let role = req
        .role
        .as_deref()
        .map(|r| r.trim().to_lowercase())
        .filter(|r| !r.is_empty())
        .unwrap_or_else(|| "developer".to_string());
    let role_prompt = req
        .role_prompt
        .as_deref()
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| db::default_role_prompt(&role).to_string());

    let active = cli_instance::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(req.name.trim().to_string()),
        kind: Set(kind),
        transport: Set(transport),
        endpoint_url: Set(req.endpoint_url.trim().to_string()),
        bridge_url: Set(bridge_url),
        secret_token: Set(req.secret_token.filter(|s| !s.trim().is_empty())),
        default_cwd: Set(req.default_cwd.unwrap_or_else(|| state.default_cwd.clone())),
        role: Set(role),
        role_prompt: Set(role_prompt),
        created_at: Set(now),
        updated_at: Set(now),
    };

    match active.insert(&state.db).await {
        Ok(model) => (StatusCode::CREATED, Json(json!(model))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn update_instance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateInstanceRequest>,
) -> impl IntoResponse {
    let existing = match cli_instance::Entity::find_by_id(id).one(&state.db).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Instance not found" })),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        }
    };

    let mut active: cli_instance::ActiveModel = existing.into();
    if let Some(name) = req.name {
        active.name = Set(name.trim().to_string());
    }
    if let Some(kind) = req.kind {
        active.kind = Set(kind.trim().to_lowercase());
    }
    if let Some(transport) = req.transport {
        active.transport = Set(transport.trim().to_lowercase());
    }
    if let Some(endpoint_url) = req.endpoint_url {
        active.endpoint_url = Set(endpoint_url.trim().to_string());
    }
    if let Some(bridge_url) = req.bridge_url {
        active.bridge_url = Set(if bridge_url.trim().is_empty() {
            None
        } else {
            Some(bridge_url.trim().to_string())
        });
    }
    if let Some(secret_token) = req.secret_token {
        active.secret_token = Set(if secret_token.trim().is_empty() {
            None
        } else {
            Some(secret_token)
        });
    }
    if let Some(default_cwd) = req.default_cwd {
        active.default_cwd = Set(default_cwd.trim().to_string());
    }
    if let Some(role) = req.role {
        let r = role.trim().to_lowercase();
        if !r.is_empty() {
            active.role = Set(r.clone());
            if req
                .role_prompt
                .as_deref()
                .map(|s| s.trim().is_empty())
                .unwrap_or(true)
            {
                active.role_prompt = Set(db::default_role_prompt(&r).to_string());
            }
        }
    }
    if let Some(role_prompt) = req.role_prompt
        && !role_prompt.trim().is_empty()
    {
        active.role_prompt = Set(role_prompt.trim().to_string());
    }
    active.updated_at = Set(Utc::now());

    match active.update(&state.db).await {
        Ok(updated) => (StatusCode::OK, Json(json!(updated))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn delete_instance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match cli_instance::Entity::delete_by_id(id).exec(&state.db).await {
        Ok(res) if res.rows_affected > 0 => {
            (StatusCode::OK, Json(json!({ "deleted": true, "id": id })))
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Instance not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn instance_auth_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<InstanceAuthRequest>,
) -> impl IntoResponse {
    let inst = match cli_instance::Entity::find_by_id(id).one(&state.db).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Instance not found" })),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        }
    };

    let rpc_url = inst
        .bridge_url
        .clone()
        .unwrap_or_else(|| inst.endpoint_url.clone());

    let (method, params) = match req.action.as_str() {
        "status" => ("auth/status", json!({})),
        "device_start" => ("auth/device_start", json!({})),
        "api_key" => (
            "auth/api_key",
            json!({ "apiKey": req.value.clone().unwrap_or_default() }),
        ),
        "oauth_callback" => (
            "auth/oauth_callback",
            json!({ "callbackUrl": req.value.clone().unwrap_or_default() }),
        ),
        other => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("Unknown auth action: {}", other) })),
            )
        }
    };

    // If API key is provided, also store it in the instance's secret_token column
    if req.action == "api_key"
        && let Some(ref key_val) = req.value
    {
        let mut active: cli_instance::ActiveModel = inst.clone().into();
        active.secret_token = Set(Some(key_val.clone()));
        active.updated_at = Set(Utc::now());
        let _ = active.update(&state.db).await;
    }

    if rpc_url.starts_with("http://") || rpc_url.starts_with("https://") {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        let mut builder = state.http_client.post(&rpc_url).json(&payload);
        if let Some(sec) = inst.secret_token.as_ref().or(state.bridge_secret.as_ref()) {
            builder = builder.header("X-Secret-Key", sec);
        }
        if let Ok(resp) = builder.send().await
            && let Ok(val) = resp.json::<Value>().await
        {
            return (
                StatusCode::OK,
                Json(json!({
                    "instance_id": id,
                    "name": inst.name,
                    "kind": inst.kind,
                    "auth": val["result"]
                })),
            );
        }
    }

    // Fallback for Codex WebSocket app-server if bridge HTTP is not reachable
    if inst.kind == "codex"
        && inst.endpoint_url.starts_with("ws")
        && let Ok((mut ws, _)) = connect_async(&inst.endpoint_url).await
    {
        let _ = ws
                .send(TungMessage::Text(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "initialize",
                        "params": { "clientInfo": { "name": "agent-hub-auth", "version": "1.0.0" } }
                    })
                    .to_string()
                    .into(),
                ))
                .await;
            let _ = ws.next().await;

            if req.action == "api_key" {
                let key = req.value.unwrap_or_default();
                let _ = ws
                    .send(TungMessage::Text(
                        json!({
                            "jsonrpc": "2.0",
                            "id": 2,
                            "method": "account/login/start",
                            "params": { "type": "apiKey", "apiKey": key }
                        })
                        .to_string()
                        .into(),
                    ))
                    .await;
                return (
                    StatusCode::OK,
                    Json(json!({
                        "instance_id": id,
                        "name": inst.name,
                        "kind": inst.kind,
                        "auth": { "authenticated": true, "detail": "API key sent to codex app-server" }
                    })),
                );
            } else {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "instance_id": id,
                        "name": inst.name,
                        "kind": inst.kind,
                        "auth": { "authenticated": true, "detail": "codex app-server reachable via WebSocket" }
                    })),
                );
            }
    }

    (
        StatusCode::OK,
        Json(json!({
            "instance_id": id,
            "name": inst.name,
            "kind": inst.kind,
            "auth": {
                "authenticated": false,
                "detail": format!("Container unreachable at {}", rpc_url)
            }
        })),
    )
}

async fn proxy_instance_web_root(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    proxy_instance_web_inner(state, id, "".to_string(), method, headers, body).await
}

async fn proxy_instance_web_path(
    State(state): State<Arc<AppState>>,
    Path((id, path)): Path<(Uuid, String)>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    proxy_instance_web_inner(state, id, path, method, headers, body).await
}

async fn proxy_instance_web_inner(
    state: Arc<AppState>,
    id: Uuid,
    subpath: String,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let Some(inst) = cli_instance::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .ok()
        .flatten()
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Instance not found" })),
        )
            .into_response();
    };

    // Determine native web server base URL of the CLI instance (e.g. Antigravity :4400, Goose :3284, or agent-bridge /web)
    let base_web_url = if let Some(ref b) = inst.bridge_url {
        if b.ends_with("/rpc") {
            format!("{}/web", b.trim_end_matches("/rpc"))
        } else {
            b.clone()
        }
    } else if inst.endpoint_url.starts_with("http") {
        inst.endpoint_url.trim_end_matches("/rpc").to_string()
    } else {
        let host = Url::parse(&inst.endpoint_url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let port = match inst.kind.as_str() {
            "antigravity" => 4400,
            "goose" => 3284,
            "codex" => 4500,
            _ => 4400,
        };
        format!("http://{}:{}", host, port)
    };

    let target = if subpath.is_empty() {
        base_web_url
    } else {
        format!(
            "{}/{}",
            base_web_url.trim_end_matches('/'),
            subpath.trim_start_matches('/')
        )
    };

    let mut req = state
        .http_client
        .request(method, &target)
        .body(body.to_vec());
    if let Some(ct) = headers.get("content-type").and_then(|v| v.to_str().ok()) {
        req = req.header("Content-Type", ct);
    }
    if let Some(ref sec) = inst.secret_token {
        req = req.header("X-Secret-Key", sec);
    }

    match req.send().await {
        Ok(resp) => {
            let status =
                StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::OK);
            let mut builder = Response::builder().status(status);
            if let Some(ct) = resp.headers().get("content-type") {
                builder = builder.header("content-type", ct);
            }
            builder
                .body(Body::from_stream(resp.bytes_stream()))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("CLI native web server unreachable ({}): {}", target, e) })),
        )
            .into_response(),
    }
}

// ============================================================================
// ORCHESTRATION CHANNELS & REAL-TIME MESSAGE BUS (/api/orchestration/...)
// ============================================================================
async fn list_channels(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match orchestration_channel::Entity::find()
        .order_by_asc(orchestration_channel::Column::CreatedAt)
        .all(&state.db)
        .await
    {
        Ok(list) => (StatusCode::OK, Json(json!(list))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn get_channel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match orchestration_channel::Entity::find_by_id(id)
        .one(&state.db)
        .await
    {
        Ok(Some(ch)) => (StatusCode::OK, Json(json!(ch))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Channel not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn create_channel(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateChannelRequest>,
) -> impl IntoResponse {
    let ids = req.participant_ids.unwrap_or_default();
    let ids_json = serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string());
    let active = orchestration_channel::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(req.name.trim().to_string()),
        description: Set(req.description),
        participant_ids_json: Set(ids_json),
        retention_days: Set(req.retention_days.unwrap_or(30).max(0)),
        max_messages: Set(req.max_messages.unwrap_or(200).max(0)),
        created_at: Set(Utc::now()),
    };

    match active.insert(&state.db).await {
        Ok(ch) => (StatusCode::CREATED, Json(json!(ch))),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn update_channel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateChannelRequest>,
) -> impl IntoResponse {
    let existing = match orchestration_channel::Entity::find_by_id(id)
        .one(&state.db)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Channel not found" })),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        }
    };

    let mut active: orchestration_channel::ActiveModel = existing.into();
    if let Some(name) = req.name {
        active.name = Set(name.trim().to_string());
    }
    if let Some(desc) = req.description {
        active.description = Set(Some(desc));
    }
    if let Some(ids) = req.participant_ids {
        active.participant_ids_json =
            Set(serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string()));
    }
    if let Some(days) = req.retention_days {
        active.retention_days = Set(days.max(0));
    }
    if let Some(max_m) = req.max_messages {
        active.max_messages = Set(max_m.max(0));
    }

    match active.update(&state.db).await {
        Ok(updated) => {
            let _ = apply_channel_retention(&state, &updated).await;
            (StatusCode::OK, Json(json!(updated)))
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn delete_channel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match orchestration_channel::Entity::delete_by_id(id)
        .exec(&state.db)
        .await
    {
        Ok(res) if res.rows_affected > 0 => {
            (StatusCode::OK, Json(json!({ "deleted": true, "id": id })))
        }
        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Channel not found" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn list_channel_messages(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match channel_message::Entity::find()
        .filter(channel_message::Column::ChannelId.eq(id))
        .order_by_asc(channel_message::Column::CreatedAt)
        .all(&state.db)
        .await
    {
        Ok(msgs) => (StatusCode::OK, Json(json!(msgs))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn persist_and_broadcast_channel_msg(
    state: &AppState,
    channel_id: Uuid,
    sender_instance_id: Option<Uuid>,
    sender_name: &str,
    recipient_name: &str,
    content: &str,
) -> Result<channel_message::Model, String> {
    let model = channel_message::ActiveModel {
        id: Set(Uuid::new_v4()),
        channel_id: Set(channel_id),
        sender_instance_id: Set(sender_instance_id),
        sender_name: Set(sender_name.to_string()),
        recipient_name: Set(recipient_name.to_string()),
        content: Set(content.to_string()),
        created_at: Set(Utc::now()),
    }
    .insert(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let tx = state.get_channel_sender(channel_id).await;
    if let Ok(serialized) = serde_json::to_string(&model) {
        let _ = tx.send(serialized);
    }
    Ok(model)
}

async fn publish_channel_message(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<Uuid>,
    Json(req): Json<PublishChannelMessageRequest>,
) -> impl IntoResponse {
    let channel = match orchestration_channel::Entity::find_by_id(channel_id)
        .one(&state.db)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "Orchestration channel not found" })),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
        }
    };

    let sender_name = req.sender_name.unwrap_or_else(|| "Utilisateur".to_string());
    let recipient_name = req.recipient_name.unwrap_or_else(|| "@all".to_string());
    let auto_dispatch = req.auto_dispatch.unwrap_or(true);

    let initial_msg = match persist_and_broadcast_channel_msg(
        &state,
        channel_id,
        req.sender_instance_id,
        &sender_name,
        &recipient_name,
        &req.content,
    )
    .await
    {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            )
        }
    };

    let mut replies = Vec::new();

    if auto_dispatch {
        let participant_ids: Vec<Uuid> =
            serde_json::from_str(&channel.participant_ids_json).unwrap_or_default();

        let all_instances = cli_instance::Entity::find()
            .all(&state.db)
            .await
            .unwrap_or_default();

        // Filter participants in channel order (or all instances if participant_ids is empty)
        let mut participants: Vec<cli_instance::Model> = if participant_ids.is_empty() {
            all_instances.clone()
        } else {
            participant_ids
                .iter()
                .filter_map(|pid| all_instances.iter().find(|i| &i.id == pid).cloned())
                .collect()
        };

        // If a specific recipient_name (not "@all") was targeted, filter to matching instance
        if recipient_name != "@all" && !recipient_name.is_empty() {
            let target_lower = recipient_name.trim_start_matches('@').to_lowercase();
            let specific: Vec<cli_instance::Model> = all_instances
                .iter()
                .filter(|i| {
                    i.name.to_lowercase() == target_lower
                        || i.kind.to_lowercase() == target_lower
                        || i.id.to_string() == target_lower
                })
                .cloned()
                .collect();
            if !specific.is_empty() {
                participants = specific;
            }
        }

        let mut running_context = format!(
            "[Orchestration channel '{}' — Message from {}]:\n{}\n",
            channel.name, sender_name, req.content
        );

        for inst in &participants {
            if Some(inst.id) == req.sender_instance_id || inst.name == sender_name {
                continue;
            }
            let role_instruction = if inst.role_prompt.trim().is_empty() {
                db::default_role_prompt(&inst.role).to_string()
            } else {
                inst.role_prompt.clone()
            };
            let prompt_for_agent = format!(
                "[System — Identity & Role]\nAgent: {} (CLI: {}, Role: {})\nMission: {}\n\n---\n{}\n---\nReply on channel '{}' adhering to your role '{}' (you can mention another agent with @AgentName if needed):",
                inst.name, inst.kind, inst.role, role_instruction, running_context, channel.name, inst.role
            );

            if let Ok(reply_text) = ask_instance_model(&state, inst, &prompt_for_agent, None, "auto").await
                && let Ok(saved_reply) = persist_and_broadcast_channel_msg(
                    &state,
                    channel_id,
                    Some(inst.id),
                    &inst.name,
                    "@all",
                    &reply_text,
                )
                .await
            {
                running_context.push_str(&format!("\n[Reply from {}]:\n{}\n", inst.name, reply_text));
                replies.push(saved_reply);
            }
        }
    }

    let _ = apply_channel_retention(&state, &channel).await;

    (
        StatusCode::OK,
        Json(json!({
            "message": initial_msg,
            "replies": replies
        })),
    )
}

async fn apply_channel_retention(
    state: &AppState,
    channel: &orchestration_channel::Model,
) -> Result<u64, DbErr> {
    let mut removed: u64 = 0;

    // 1. Delete messages older than retention_days (if > 0)
    if channel.retention_days > 0 {
        let cutoff = Utc::now() - chrono::Duration::days(channel.retention_days as i64);
        let res = channel_message::Entity::delete_many()
            .filter(channel_message::Column::ChannelId.eq(channel.id))
            .filter(channel_message::Column::CreatedAt.lt(cutoff))
            .exec(&state.db)
            .await?;
        removed += res.rows_affected;
    }

    // 2. Enforce max_messages cap (if > 0)
    if channel.max_messages > 0 {
        let all = channel_message::Entity::find()
            .filter(channel_message::Column::ChannelId.eq(channel.id))
            .order_by_asc(channel_message::Column::CreatedAt)
            .all(&state.db)
            .await?;
        let max_cap = channel.max_messages as usize;
        if all.len() > max_cap {
            let excess = all.len() - max_cap;
            for old_msg in all.into_iter().take(excess) {
                let r = channel_message::Entity::delete_by_id(old_msg.id)
                    .exec(&state.db)
                    .await?;
                removed += r.rows_affected;
            }
        }
    }

    Ok(removed)
}

async fn prune_channel_messages(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<Uuid>,
) -> impl IntoResponse {
    let Some(channel) = orchestration_channel::Entity::find_by_id(channel_id)
        .one(&state.db)
        .await
        .ok()
        .flatten()
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Channel not found" })),
        );
    };

    match apply_channel_retention(&state, &channel).await {
        Ok(pruned) => (
            StatusCode::OK,
            Json(json!({
                "channel_id": channel.id,
                "retention_days": channel.retention_days,
                "max_messages": channel.max_messages,
                "pruned": pruned,
                "pruned_messages": pruned
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn export_channel_markdown(
    State(state): State<Arc<AppState>>,
    Path(channel_id): Path<Uuid>,
) -> impl IntoResponse {
    let Some(channel) = orchestration_channel::Entity::find_by_id(channel_id)
        .one(&state.db)
        .await
        .ok()
        .flatten()
    else {
        return (
            StatusCode::NOT_FOUND,
            [("content-type", "text/plain; charset=utf-8")],
            "Channel not found".to_string(),
        );
    };

    let msgs = channel_message::Entity::find()
        .filter(channel_message::Column::ChannelId.eq(channel_id))
        .order_by_asc(channel_message::Column::CreatedAt)
        .all(&state.db)
        .await
        .unwrap_or_default();

    let mut md = format!(
        "# Orchestration Channel: #{}\n\n\
        - **Description**: {}\n\
        - **Retention Policy**: {} days / max {} messages\n\
        - **Exported Messages**: {}\n\n---\n\n",
        channel.name,
        channel.description.as_deref().unwrap_or("None"),
        channel.retention_days,
        channel.max_messages,
        msgs.len()
    );

    for m in msgs {
        md.push_str(&format!(
            "### [{}] **{}** → `{}`\n\n{}\n\n---\n\n",
            m.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
            m.sender_name,
            m.recipient_name,
            m.content
        ));
    }

    (
        StatusCode::OK,
        [("content-type", "text/markdown; charset=utf-8")],
        md,
    )
}

async fn ws_orchestration_handler(
    ws: WebSocketUpgrade,
    Path(channel_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_orchestration_ws(socket, channel_id, state))
}

async fn handle_orchestration_ws(socket: WebSocket, channel_id: Uuid, state: Arc<AppState>) {
    let tx = state.get_channel_sender(channel_id).await;
    let mut rx = tx.subscribe();

    let (mut ws_sink, mut ws_stream) = socket.split();

    let broadcast_to_ws = async move {
        while let Ok(msg) = rx.recv().await {
            if ws_sink.send(AxumWsMessage::Text(msg.into())).await.is_err() {
                break;
            }
        }
    };

    let state_clone = state.clone();
    let ws_to_broadcast = async move {
        while let Some(Ok(AxumWsMessage::Text(text))) = ws_stream.next().await {
            if let Ok(val) = serde_json::from_str::<Value>(&text) {
                let sender = val["sender_name"].as_str().unwrap_or("CLI-Agent");
                let recipient = val["recipient_name"].as_str().unwrap_or("@all");
                let content = val["content"].as_str().unwrap_or(&text);
                let _ = persist_and_broadcast_channel_msg(
                    &state_clone,
                    channel_id,
                    None,
                    sender,
                    recipient,
                    content,
                )
                .await;
            }
        }
    };

    tokio::select! {
        _ = broadcast_to_ws => {},
        _ = ws_to_broadcast => {},
    }
}

// ============================================================================
// UNIFIED ASK & TMUX API (/api/agents/ask, /api/agents/tmux)
// ============================================================================
async fn resolve_instance(
    state: &AppState,
    instance_id: Option<Uuid>,
    agent_str: Option<&str>,
) -> Option<cli_instance::Model> {
    if let Some(id) = instance_id
        && let Ok(Some(m)) = cli_instance::Entity::find_by_id(id).one(&state.db).await
    {
        return Some(m);
    }
    if let Some(ag) = agent_str {
        let lower = ag.trim().to_lowercase();
        if let Ok(all) = cli_instance::Entity::find()
            .order_by_asc(cli_instance::Column::CreatedAt)
            .all(&state.db)
            .await
        {
            if let Some(by_name) = all.iter().find(|i| i.name.to_lowercase() == lower) {
                return Some(by_name.clone());
            }
            let normalized_kind = if lower == "agy" { "antigravity" } else { lower.as_str() };
            if let Some(by_kind) = all.iter().find(|i| i.kind.to_lowercase() == normalized_kind) {
                return Some(by_kind.clone());
            }
        }
    }
    None
}

async fn ask_instance_model(
    state: &AppState,
    inst: &cli_instance::Model,
    prompt: &str,
    cwd_override: Option<&str>,
    mode: &str,
) -> Result<String, String> {
    let cwd = cwd_override.unwrap_or(&inst.default_cwd);
    match inst.kind.as_str() {
        "codex" => ask_codex_oneshot(&inst.endpoint_url, prompt, cwd).await,
        "goose" => {
            let sec = inst
                .secret_token
                .clone()
                .or_else(|| state.goose_secret.clone());
            ask_goose_oneshot(&inst.endpoint_url, sec.as_deref(), prompt, cwd).await
        }
        "claude" | "antigravity" => {
            let sec = inst
                .secret_token
                .clone()
                .or_else(|| state.bridge_secret.clone());
            ask_http_rpc_peer(
                &state.http_client,
                &inst.endpoint_url,
                sec.as_deref(),
                &inst.name,
                prompt,
                cwd,
                mode,
            )
            .await
        }
        other => Err(format!("Unsupported agent kind: {}", other)),
    }
}

async fn handle_agent_ask(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AgentAskRequest>,
) -> impl IntoResponse {
    let mode = req.mode.unwrap_or_else(|| "auto".to_string());
    if let Some(inst) = resolve_instance(&state, req.instance_id, req.agent.as_deref()).await {
        match ask_instance_model(&state, &inst, &req.prompt, req.cwd.as_deref(), &mode).await {
            Ok(reply) => (
                StatusCode::OK,
                Json(json!({
                    "instance_id": inst.id,
                    "name": inst.name,
                    "agent": inst.kind,
                    "reply": reply
                })),
            ),
            Err(err) => (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "instance_id": inst.id,
                    "name": inst.name,
                    "agent": inst.kind,
                    "error": err
                })),
            ),
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "No matching CLI instance found" })),
        )
    }
}

async fn handle_agent_tmux(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AgentTmuxRequest>,
) -> impl IntoResponse {
    let Some(inst) = resolve_instance(&state, req.instance_id, req.agent.as_deref()).await else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Instance not found" })),
        );
    };

    let url = inst
        .bridge_url
        .as_deref()
        .unwrap_or(&inst.endpoint_url);

    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tmux/capture",
        "params": { "lines": req.lines.unwrap_or(120) }
    });

    let mut builder = state.http_client.post(url).json(&payload);
    if let Some(sec) = inst.secret_token.as_ref().or(state.bridge_secret.as_ref()) {
        builder = builder.header("X-Secret-Key", sec);
    }

    match builder.send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(val) => {
                let output = val["result"]["output"].as_str().unwrap_or("").to_string();
                (
                    StatusCode::OK,
                    Json(json!({
                        "instance_id": inst.id,
                        "name": inst.name,
                        "agent": inst.kind,
                        "output": output
                    })),
                )
            }
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("Invalid JSON from {}: {}", inst.name, e) })),
            ),
        },
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("Unreachable {} bridge ({}): {}", inst.name, url, e) })),
        ),
    }
}

async fn ask_http_rpc_peer(
    http_client: &reqwest::Client,
    url: &str,
    secret: Option<&str>,
    agent_name: &str,
    prompt: &str,
    cwd: &str,
    mode: &str,
) -> Result<String, String> {
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "prompt/send",
        "params": { "prompt": prompt, "cwd": cwd, "mode": mode }
    });
    let mut req = http_client.post(url).json(&payload);
    if let Some(sec) = secret {
        req = req.header("X-Secret-Key", sec);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| format!("Unable to reach {} ({}): {}", agent_name, url, e))?;
    let val: Value = resp
        .json()
        .await
        .map_err(|e| format!("Invalid JSON response from {}: {}", agent_name, e))?;
    if let Some(err) = val.get("error") {
        return Err(err.to_string());
    }
    Ok(val["result"]["reply"]
        .as_str()
        .unwrap_or("")
        .to_string())
}

async fn ask_codex_oneshot(ws_url: &str, prompt: &str, cwd: &str) -> Result<String, String> {
    let (mut ws, _) = connect_async(ws_url)
        .await
        .map_err(|e| format!("Codex WS connection failed ({}): {}", ws_url, e))?;

    ws.send(TungMessage::Text(
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "clientInfo": { "name": "agent-hub-bridge", "version": "1.0.0" } }
        })
        .to_string()
        .into(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    let _ = ws.next().await;

    ws.send(TungMessage::Text(
        json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} })
            .to_string()
            .into(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    ws.send(TungMessage::Text(
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "thread/start",
            "params": { "cwd": cwd }
        })
        .to_string()
        .into(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    let mut thread_id = "default".to_string();
    while let Some(Ok(TungMessage::Text(txt))) = ws.next().await {
        if let Ok(v) = serde_json::from_str::<Value>(&txt)
            && v.get("id") == Some(&json!(2))
        {
            thread_id = v["result"]["thread"]["id"]
                .as_str()
                .or_else(|| v["result"]["threadId"].as_str())
                .unwrap_or("default")
                .to_string();
            break;
        }
    }

    ws.send(TungMessage::Text(
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "turn/start",
            "params": {
                "threadId": thread_id,
                "input": [{ "type": "text", "text": prompt }]
            }
        })
        .to_string()
        .into(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    let mut collected = String::new();
    while let Some(Ok(TungMessage::Text(txt))) = ws.next().await {
        if let Ok(v) = serde_json::from_str::<Value>(&txt) {
            let method = v["method"].as_str().unwrap_or("");
            if method == "item/agentMessage/delta" {
                if let Some(delta) = v["params"]["delta"].as_str() {
                    collected.push_str(delta);
                }
            } else if method == "turn/completed"
                || (v.get("id") == Some(&json!(3)) && !collected.is_empty())
            {
                break;
            }
        }
    }

    Ok(collected)
}

async fn ask_goose_oneshot(
    ws_url: &str,
    secret: Option<&str>,
    prompt: &str,
    cwd: &str,
) -> Result<String, String> {
    let mut req = ws_url
        .into_client_request()
        .map_err(|e| e.to_string())?;

    if let Some(token) = secret
        && let Ok(val) = token.parse()
    {
        req.headers_mut().insert("X-Secret-Key", val);
    }

    let (mut ws, _) = connect_async(req)
        .await
        .map_err(|e| format!("Goose ACP connection failed ({}): {}", ws_url, e))?;

    ws.send(TungMessage::Text(
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": 1,
                "clientCapabilities": { "fs": { "readTextFile": true, "writeTextFile": true }, "terminal": true },
                "clientInfo": { "name": "agent-hub-bridge", "version": "1.0.0" }
            }
        })
        .to_string()
        .into(),
    ))
    .await
    .map_err(|e| e.to_string())?;
    let _ = ws.next().await;

    ws.send(TungMessage::Text(
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "session/new",
            "params": { "cwd": cwd, "mcpServers": [] }
        })
        .to_string()
        .into(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    let mut session_id = "default".to_string();
    while let Some(Ok(TungMessage::Text(txt))) = ws.next().await {
        if let Ok(v) = serde_json::from_str::<Value>(&txt)
            && v.get("id") == Some(&json!(2))
        {
            session_id = v["result"]["sessionId"]
                .as_str()
                .unwrap_or("default")
                .to_string();
            break;
        }
    }

    ws.send(TungMessage::Text(
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "session/prompt",
            "params": {
                "sessionId": session_id,
                "prompt": [{ "type": "text", "text": prompt }]
            }
        })
        .to_string()
        .into(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    let mut collected = String::new();
    while let Some(Ok(TungMessage::Text(txt))) = ws.next().await {
        if let Ok(v) = serde_json::from_str::<Value>(&txt) {
            if v["method"].as_str() == Some("session/update") {
                let upd = &v["params"]["update"];
                if upd["sessionUpdate"].as_str() == Some("agent_message_chunk")
                    && let Some(t) = upd["content"]["text"].as_str()
                {
                    collected.push_str(t);
                }
            } else if v.get("id") == Some(&json!(3)) {
                break;
            }
        }
    }

    Ok(collected)
}

// ============================================================================
// CODEX BRIDGE (/ws/codex)
// ============================================================================
async fn ws_codex_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<CodexWsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_codex_ws(socket, query, state))
}

async fn handle_codex_ws(
    mut client_ws: WebSocket,
    query: CodexWsQuery,
    state: Arc<AppState>,
) {
    let mut mode = query.mode.clone().unwrap_or_else(|| "ws".to_string());
    let mut target = query.target.clone().unwrap_or_default();

    if let Some(id) = query.instance_id
        && let Ok(Some(inst)) = cli_instance::Entity::find_by_id(id).one(&state.db).await
    {
        if query.mode.is_none() {
            mode = inst.transport;
        }
        if target.trim().is_empty() {
            target = inst.endpoint_url;
        }
    }

    if target.trim().is_empty() {
        target = if mode == "unix" {
            state.codex_unix_socket.clone()
        } else {
            state.codex_ws_url.clone()
        };
    }

    if mode == "unix" || target.starts_with('/') || target.starts_with("unix://") {
        let sock_path = target.trim_start_matches("unix://");
        info!("Connecting to Codex Unix socket: {}", sock_path);
        match UnixStream::connect(sock_path).await {
            Ok(unix_stream) => {
                bridge_ws_to_unix(client_ws, unix_stream).await;
            }
            Err(e) => {
                warn!("Codex Unix socket connection failed ({}): {}", sock_path, e);
                let _ = send_bridge_error(
                    &mut client_ws,
                    &format!("Unable to reach Codex Unix socket ({}): {}", sock_path, e),
                )
                .await;
            }
        }
    } else {
        info!("Connecting to Codex WebSocket: {}", target);
        match connect_async(&target).await {
            Ok((upstream_ws, _)) => {
                bridge_ws_to_ws(client_ws, upstream_ws).await;
            }
            Err(e) => {
                warn!("Codex WebSocket connection failed ({}): {}", target, e);
                let _ = send_bridge_error(
                    &mut client_ws,
                    &format!("Unable to reach Codex WebSocket ({}): {}", target, e),
                )
                .await;
            }
        }
    }
}

// ============================================================================
// GOOSE ACP BRIDGE (/ws/goose)
// ============================================================================
async fn ws_goose_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<GooseWsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_goose_ws(socket, query, state))
}

async fn handle_goose_ws(
    mut client_ws: WebSocket,
    query: GooseWsQuery,
    state: Arc<AppState>,
) {
    let mut target_url = query.target.clone().unwrap_or_default();
    let mut secret = query.token.clone().filter(|s| !s.trim().is_empty());

    if let Some(id) = query.instance_id
        && let Ok(Some(inst)) = cli_instance::Entity::find_by_id(id).one(&state.db).await
    {
        if target_url.trim().is_empty() {
            target_url = inst.endpoint_url;
        }
        if secret.is_none() {
            secret = inst.secret_token;
        }
    }

    if target_url.trim().is_empty() {
        target_url = state.goose_ws_url.clone();
    }
    if secret.is_none() {
        secret = state.goose_secret.clone();
    }

    let mut req = match target_url.as_str().into_client_request() {
        Ok(r) => r,
        Err(e) => {
            let _ = send_bridge_error(
                &mut client_ws,
                &format!("Invalid Goose URL ({}): {}", target_url, e),
            )
            .await;
            return;
        }
    };

    if let Some(ref token) = secret {
        if let Ok(val) = token.parse() {
            req.headers_mut().insert("X-Secret-Key", val);
        }
        if let Ok(val) = format!("Bearer {}", token).parse() {
            req.headers_mut().insert("Authorization", val);
        }
    }

    info!("Connecting to Goose ACP WebSocket: {}", target_url);
    match connect_async(req).await {
        Ok((upstream_ws, _)) => {
            bridge_ws_to_ws(client_ws, upstream_ws).await;
        }
        Err(e) => {
            warn!("Goose WebSocket connection failed ({}): {}", target_url, e);
            let _ = send_bridge_error(
                &mut client_ws,
                &format!("Unable to reach Goose ACP ({}): {}", target_url, e),
            )
            .await;
        }
    }
}

// ============================================================================
// GOOSE HTTP PROXY (/api/goose/{*path})
// ============================================================================
async fn proxy_goose_http(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let url = format!("{}/{}", state.goose_http_url, path.trim_start_matches('/'));
    let mut req = state.http_client.request(method, &url).body(body.to_vec());

    if let Some(ct) = headers.get("content-type")
        && let Ok(ct_str) = ct.to_str()
    {
        req = req.header("Content-Type", ct_str);
    }
    if let Some(accept) = headers.get("accept")
        && let Ok(acc_str) = accept.to_str()
    {
        req = req.header("Accept", acc_str);
    }

    let secret = headers
        .get("x-secret-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| state.goose_secret.clone());

    if let Some(token) = secret {
        req = req
            .header("X-Secret-Key", &token)
            .header("Authorization", format!("Bearer {}", token));
    }

    match req.send().await {
        Ok(resp) => {
            let status =
                StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::OK);
            let mut builder = Response::builder().status(status);
            if let Some(ct) = resp.headers().get("content-type") {
                builder = builder.header("content-type", ct);
            }
            let stream = resp.bytes_stream();
            builder
                .body(Body::from_stream(stream))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => {
            error!("Goose HTTP proxy error ({}): {}", url, e);
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32000,
                        "message": format!("Goose HTTP proxy error ({}): {}", url, e)
                    }
                })),
            )
                .into_response()
        }
    }
}

// ============================================================================
// GENERIC BIDIRECTIONAL STREAM BRIDGES (Axum 0.8 + Tungstenite 0.28)
// ============================================================================
async fn send_bridge_error(ws: &mut WebSocket, message: &str) -> Result<(), axum::Error> {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "bridge/error",
        "params": { "message": message }
    });
    ws.send(AxumWsMessage::Text(payload.to_string().into())).await
}

async fn bridge_ws_to_ws<S>(client_ws: WebSocket, upstream_ws: S)
where
    S: futures_util::Stream<Item = Result<TungMessage, tokio_tungstenite::tungstenite::Error>>
        + futures_util::Sink<TungMessage, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin
        + Send
        + 'static,
{
    let (mut client_sink, mut client_stream) = client_ws.split();
    let (mut up_sink, mut up_stream) = upstream_ws.split();

    let client_to_up = async move {
        while let Some(Ok(msg)) = client_stream.next().await {
            let out = match msg {
                AxumWsMessage::Text(t) => TungMessage::Text(t.to_string().into()),
                AxumWsMessage::Binary(b) => TungMessage::Binary(b),
                AxumWsMessage::Ping(p) => TungMessage::Ping(p),
                AxumWsMessage::Pong(p) => TungMessage::Pong(p),
                AxumWsMessage::Close(_) => {
                    let _ = up_sink.close().await;
                    break;
                }
            };
            if up_sink.send(out).await.is_err() {
                break;
            }
        }
    };

    let up_to_client = async move {
        while let Some(Ok(msg)) = up_stream.next().await {
            let out = match msg {
                TungMessage::Text(t) => AxumWsMessage::Text(t.to_string().into()),
                TungMessage::Binary(b) => AxumWsMessage::Binary(b),
                TungMessage::Ping(p) => AxumWsMessage::Ping(p),
                TungMessage::Pong(p) => AxumWsMessage::Pong(p),
                TungMessage::Close(_) => {
                    let _ = client_sink.close().await;
                    break;
                }
                TungMessage::Frame(_) => continue,
            };
            if client_sink.send(out).await.is_err() {
                break;
            }
        }
    };

    tokio::select! {
        _ = client_to_up => {},
        _ = up_to_client => {},
    }
}

async fn bridge_ws_to_unix(client_ws: WebSocket, unix_stream: UnixStream) {
    let (mut client_sink, mut client_stream) = client_ws.split();
    let (unix_read, mut unix_write) = unix_stream.into_split();
    let mut lines = BufReader::new(unix_read).lines();

    let client_to_unix = async move {
        while let Some(Ok(msg)) = client_stream.next().await {
            match msg {
                AxumWsMessage::Text(t) => {
                    let mut line = t.trim().to_string();
                    line.push('\n');
                    if unix_write.write_all(line.as_bytes()).await.is_err() {
                        break;
                    }
                    let _ = unix_write.flush().await;
                }
                AxumWsMessage::Close(_) => break,
                _ => {}
            }
        }
    };

    let unix_to_client = async move {
        while let Ok(Some(line)) = lines.next_line().await {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if client_sink
                .send(AxumWsMessage::Text(trimmed.to_string().into()))
                .await
                .is_err()
            {
                break;
            }
        }
    };

    tokio::select! {
        _ = client_to_unix => {},
        _ = unix_to_client => {},
    }
}

// ============================================================================
// ROLES PRESETS & AGENT ONBOARDING (/api/roles/presets, /api/instances/{id}/onboard)
// ============================================================================
async fn list_role_presets(
    Query(query): Query<RolePresetsQuery>,
) -> impl IntoResponse {
    let lang = query.lang.as_deref().unwrap_or("en");
    let is_fr = lang == "fr";
    let presets = json!([
        {
            "id": "architect",
            "label": if is_fr { "🏗️ Architecte Logiciel" } else { "🏗️ Software Architect" },
            "prompt": db::default_role_prompt_lang("architect", lang)
        },
        {
            "id": "developer",
            "label": if is_fr { "💻 Développeur Principal" } else { "💻 Lead Developer" },
            "prompt": db::default_role_prompt_lang("developer", lang)
        },
        {
            "id": "reviewer",
            "label": if is_fr { "🔍 Code Reviewer & Qualité" } else { "🔍 Code Reviewer & QA" },
            "prompt": db::default_role_prompt_lang("reviewer", lang)
        },
        {
            "id": "tester",
            "label": if is_fr { "🧪 Ingénieur QA & Tests E2E" } else { "🧪 QA & E2E Test Engineer" },
            "prompt": db::default_role_prompt_lang("tester", lang)
        },
        {
            "id": "security",
            "label": if is_fr { "🛡️ Auditeur Sécurité & DevSecOps" } else { "🛡️ Security & DevSecOps Auditor" },
            "prompt": db::default_role_prompt_lang("security", lang)
        },
        {
            "id": "general",
            "label": if is_fr { "🤖 Agent Polyvalent" } else { "🤖 Versatile Agent" },
            "prompt": db::default_role_prompt_lang("general", lang)
        }
    ]);
    (StatusCode::OK, Json(presets))
}

async fn onboard_instance_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<OnboardInstanceRequest>,
) -> impl IntoResponse {
    let Some(inst) = cli_instance::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .ok()
        .flatten()
    else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Instance not found" })),
        );
    };

    let channel_name = req.channel_name.unwrap_or_else(|| "general".to_string());
    let lang = req.lang.as_deref().unwrap_or("en");
    let base_role_prompt = req
        .custom_prompt
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            if inst.role_prompt.trim().is_empty() {
                db::default_role_prompt_lang(&inst.role, lang).to_string()
            } else {
                inst.role_prompt.clone()
            }
        });

    let onboarding_prompt = if lang == "fr" {
        format!(
            "[INITIALISATION D'AGENT — SIMPLY AGENT HUB]\n\
            1. Ton identité : Tu es l'instance '{}' (moteur CLI : '{}', dossier de travail : '{}').\n\
            2. Ton rôle attribué : '{}'.\n\
            3. Ta mission & tes consignes :\n{}\n\
            4. Fonctionnement de l'orchestration :\n\
            - Tu es connecté au canal d'orchestration '{}'.\n\
            - Tu peux interpeller un autre agent du collectif avec @NomAgent.\n\
            Confirme en une phrase concise que tu as bien pris connaissance de ton identité et de ton rôle '{}'.",
            inst.name, inst.kind, inst.default_cwd, inst.role, base_role_prompt, channel_name, inst.role
        )
    } else {
        format!(
            "[AGENT ONBOARDING — SIMPLY AGENT HUB]\n\
            1. Your identity: You are instance '{}' (CLI engine: '{}', working directory: '{}').\n\
            2. Your assigned role: '{}'.\n\
            3. Your mission & guidelines:\n{}\n\
            4. Orchestration rules:\n\
            - You are connected to orchestration channel '{}'.\n\
            - You can mention any collective agent with @AgentName.\n\
            Confirm in one concise sentence that you acknowledge your identity and role '{}'.",
            inst.name, inst.kind, inst.default_cwd, inst.role, base_role_prompt, channel_name, inst.role
        )
    };

    match ask_instance_model(&state, &inst, &onboarding_prompt, None, "auto").await {
        Ok(reply) => (
            StatusCode::OK,
            Json(json!({
                "instance_id": inst.id,
                "name": inst.name,
                "role": inst.role,
                "onboarding_prompt": onboarding_prompt,
                "reply": reply
            })),
        ),
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "instance_id": inst.id,
                "name": inst.name,
                "role": inst.role,
                "error": err
            })),
        ),
    }
}

// ============================================================================
// NATIVE INTERACTIVE TERMINAL WEBSOCKET (/ws/instances/{id}/terminal)
// ============================================================================
async fn ws_instance_terminal_handler(
    ws: WebSocketUpgrade,
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_instance_terminal_ws(socket, id, state))
}

async fn fetch_instance_tmux_ansi(state: &AppState, inst: &cli_instance::Model) -> Result<String, String> {
    let url = inst.bridge_url.as_deref().unwrap_or(&inst.endpoint_url);
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tmux/capture",
        "params": { "lines": 120, "ansi": true }
    });
    let mut builder = state.http_client.post(url).json(&payload);
    if let Some(sec) = inst.secret_token.as_ref().or(state.bridge_secret.as_ref()) {
        builder = builder.header("X-Secret-Key", sec);
    }
    let resp = builder.send().await.map_err(|e| e.to_string())?;
    let val: Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(val["result"]["output"].as_str().unwrap_or("").to_string())
}

async fn send_instance_tmux_input(
    state: &AppState,
    inst: &cli_instance::Model,
    method: &str,
    params: Value,
) -> Result<String, String> {
    let url = inst.bridge_url.as_deref().unwrap_or(&inst.endpoint_url);
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });
    let mut builder = state.http_client.post(url).json(&payload);
    if let Some(sec) = inst.secret_token.as_ref().or(state.bridge_secret.as_ref()) {
        builder = builder.header("X-Secret-Key", sec);
    }
    let resp = builder.send().await.map_err(|e| e.to_string())?;
    let val: Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(val["result"]["output"].as_str().unwrap_or("").to_string())
}

async fn handle_instance_terminal_ws(socket: WebSocket, id: Uuid, state: Arc<AppState>) {
    let Some(inst) = cli_instance::Entity::find_by_id(id)
        .one(&state.db)
        .await
        .ok()
        .flatten()
    else {
        return;
    };

    let (mut ws_sink, mut ws_stream) = socket.split();
    let (screen_tx, mut screen_rx) = tokio::sync::mpsc::channel::<String>(32);

    // Initial screen capture
    if let Ok(initial) = fetch_instance_tmux_ansi(&state, &inst).await {
        let _ = screen_tx
            .send(json!({ "type": "screen", "data": initial }).to_string())
            .await;
    }

    let state_poll = state.clone();
    let inst_poll = inst.clone();
    let tx_poll = screen_tx.clone();
    let poll_task = async move {
        let mut last_screen = String::new();
        loop {
            tokio::time::sleep(Duration::from_millis(650)).await;
            if let Ok(screen) = fetch_instance_tmux_ansi(&state_poll, &inst_poll).await
                && screen != last_screen
            {
                last_screen = screen.clone();
                if tx_poll
                    .send(json!({ "type": "screen", "data": screen }).to_string())
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    };

    let sink_task = async move {
        while let Some(msg) = screen_rx.recv().await {
            if ws_sink.send(AxumWsMessage::Text(msg.into())).await.is_err() {
                break;
            }
        }
    };

    let state_in = state.clone();
    let inst_in = inst.clone();
    let tx_in = screen_tx.clone();
    let input_task = async move {
        while let Some(Ok(msg)) = ws_stream.next().await {
            if let AxumWsMessage::Text(txt) = msg
                && let Ok(v) = serde_json::from_str::<Value>(&txt)
            {
                let msg_type = v["type"].as_str().unwrap_or("line");
                let updated = match msg_type {
                    "raw" => {
                        let data = v["data"].as_str().unwrap_or("");
                        let special = v["special"].as_bool().unwrap_or(false);
                        send_instance_tmux_input(
                            &state_in,
                            &inst_in,
                            "tmux/send_raw",
                            json!({ "data": data, "special": special }),
                        )
                        .await
                    }
                    "refresh" => fetch_instance_tmux_ansi(&state_in, &inst_in).await,
                    _ => {
                        let text = v["text"].as_str().or_else(|| v["data"].as_str()).unwrap_or("");
                        send_instance_tmux_input(
                            &state_in,
                            &inst_in,
                            "tmux/send_keys",
                            json!({ "text": text }),
                        )
                        .await
                    }
                };
                if let Ok(screen) = updated {
                    let _ = tx_in
                        .send(json!({ "type": "screen", "data": screen }).to_string())
                        .await;
                }
            }
        }
    };

    tokio::select! {
        _ = poll_task => {},
        _ = sink_task => {},
        _ = input_task => {},
    }
}

// ============================================================================
// INTEGRATED GIT DIFF, STATUS, COMMIT & REVERT (/api/git/*)
// ============================================================================
fn resolve_git_cwd(state: &AppState, requested: Option<&str>) -> String {
    let candidate = requested
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&state.default_cwd);
    if FsPath::new(candidate).is_dir() {
        candidate.to_string()
    } else {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| ".".to_string())
    }
}

async fn handle_git_status(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GitStatusQuery>,
) -> impl IntoResponse {
    let workdir = resolve_git_cwd(&state, query.cwd.as_deref());

    let branch_out = tokio::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(&workdir)
        .output()
        .await;
    let branch = branch_out
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "non-git".to_string());

    let status_out = tokio::process::Command::new("git")
        .args(["status", "--porcelain=v1"])
        .current_dir(&workdir)
        .output()
        .await;

    let mut files = Vec::new();
    if let Ok(o) = status_out
        && o.status.success()
    {
        for line in String::from_utf8_lossy(&o.stdout).lines() {
            if line.len() >= 3 {
                let status_code = line[0..2].trim().to_string();
                let file_path = line[3..].trim().to_string();
                files.push(json!({
                    "status": if status_code.is_empty() { "M".to_string() } else { status_code },
                    "path": file_path
                }));
            }
        }
    }

    let diff_out = tokio::process::Command::new("git")
        .args(["diff", "HEAD"])
        .current_dir(&workdir)
        .output()
        .await;
    let diff = diff_out
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let log_out = tokio::process::Command::new("git")
        .args(["log", "-n", "5", "--oneline"])
        .current_dir(&workdir)
        .output()
        .await;
    let recent_commits: Vec<String> = log_out
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.to_string())
                .collect()
        })
        .unwrap_or_default();

    (
        StatusCode::OK,
        Json(json!({
            "cwd": workdir,
            "branch": branch,
            "files": files,
            "diff": diff,
            "recent_commits": recent_commits
        })),
    )
}

async fn handle_git_commit(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GitCommitRequest>,
) -> impl IntoResponse {
    if req.message.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Commit message cannot be empty" })),
        );
    }
    let workdir = resolve_git_cwd(&state, req.cwd.as_deref());

    if let Some(ref files) = req.files {
        if !files.is_empty() {
            let mut cmd = tokio::process::Command::new("git");
            cmd.arg("add").args(files).current_dir(&workdir);
            let _ = cmd.output().await;
        } else {
            let _ = tokio::process::Command::new("git")
                .args(["add", "-A"])
                .current_dir(&workdir)
                .output()
                .await;
        }
    } else {
        let _ = tokio::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(&workdir)
            .output()
            .await;
    }

    let out = tokio::process::Command::new("git")
        .args(["commit", "-m", req.message.trim()])
        .current_dir(&workdir)
        .output()
        .await;

    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            (
                StatusCode::OK,
                Json(json!({
                    "committed": o.status.success(),
                    "cwd": workdir,
                    "detail": if stdout.is_empty() { stderr } else { stdout }
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

async fn handle_git_revert(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GitRevertRequest>,
) -> impl IntoResponse {
    let workdir = resolve_git_cwd(&state, req.cwd.as_deref());
    let target = req.file.as_deref().unwrap_or(".");

    let out = tokio::process::Command::new("git")
        .args(["checkout", "--", target])
        .current_dir(&workdir)
        .output()
        .await;

    match out {
        Ok(o) => (
            StatusCode::OK,
            Json(json!({
                "reverted": o.status.success(),
                "cwd": workdir,
                "target": target,
                "detail": String::from_utf8_lossy(&o.stderr).trim().to_string()
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn spawn_test_hub(default_cwd: &str) -> (String, Arc<AppState>) {
        let db = db::connect_and_migrate(
            "sqlite::memory:",
            "ws://127.0.0.1:14500",
            "ws://127.0.0.1:13284/acp",
            None,
            "http://127.0.0.1:13285/rpc",
            "http://127.0.0.1:13286/rpc",
            default_cwd,
        )
        .await
        .unwrap_or_else(|e| panic!("db init failed: {e}"));

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|e| panic!("client init failed: {e}"));

        let state = Arc::new(AppState {
            db,
            goose_http_url: "http://127.0.0.1:13284".to_string(),
            goose_ws_url: "ws://127.0.0.1:13284/acp".to_string(),
            goose_secret: None,
            codex_ws_url: "ws://127.0.0.1:14500".to_string(),
            codex_unix_socket: "/tmp/nonexistent-codex.sock".to_string(),
            claude_rpc_url: "http://127.0.0.1:13285/rpc".to_string(),
            antigravity_rpc_url: "http://127.0.0.1:13286/rpc".to_string(),
            bridge_secret: None,
            default_cwd: default_cwd.to_string(),
            http_client,
            channel_hubs: Arc::new(RwLock::new(HashMap::new())),
        });

        let app = build_router(state.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap_or_else(|e| panic!("bind failed: {e}"));
        let addr = listener
            .local_addr()
            .unwrap_or_else(|e| panic!("local_addr failed: {e}"));
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        (format!("http://{}", addr), state)
    }

    async fn spawn_mock_bridge() -> String {
        let app = Router::new().route(
            "/rpc",
            post(|Json(req): Json<Value>| async move {
                let rpc_id = req.get("id").cloned().unwrap_or(json!(1));
                let method = req["method"].as_str().unwrap_or("");
                let params = &req["params"];
                let result = match method {
                    "prompt/send" => {
                        let p = params["prompt"].as_str().unwrap_or("");
                        json!({
                            "reply": format!("ACK_ROLE_PROMPT len={} head={}", p.len(), &p[..p.len().min(48)]),
                            "agent": "claude"
                        })
                    }
                    "tmux/capture" => {
                        let ansi = params["ansi"].as_bool().unwrap_or(false);
                        let out = if ansi {
                            "\u{1b}[32mthenewuser@container\u{1b}[0m:~$ ready"
                        } else {
                            "thenewuser@container:~$ ready"
                        };
                        json!({ "output": out })
                    }
                    "tmux/send_raw" => {
                        let data = params["data"].as_str().unwrap_or("");
                        let special = params["special"].as_bool().unwrap_or(false);
                        json!({
                            "output": format!("\u{1b}[36mRAW_ACK\u{1b}[0m special={} data={}", special, data)
                        })
                    }
                    "tmux/send_keys" => {
                        let text = params["text"].as_str().unwrap_or("");
                        json!({ "output": format!("LINE_ACK: {}", text) })
                    }
                    _ => json!({ "status": "ok" }),
                };
                Json(json!({ "jsonrpc": "2.0", "id": rpc_id, "result": result }))
            }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap_or_else(|e| panic!("mock bridge bind failed: {e}"));
        let addr = listener
            .local_addr()
            .unwrap_or_else(|e| panic!("mock bridge addr failed: {e}"));
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });

        format!("http://{}/rpc", addr)
    }

    #[tokio::test]
    async fn test_role_presets_and_instance_roles_crud() {
        let (base_url, state) = spawn_test_hub("/tmp").await;
        let client = &state.http_client;

        // 1. Check /api/roles/presets
        let presets_resp = client
            .get(format!("{}/api/roles/presets", base_url))
            .send()
            .await
            .unwrap_or_else(|e| panic!("presets request failed: {e}"));
        assert_eq!(presets_resp.status(), StatusCode::OK);
        let presets: Vec<Value> = presets_resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("presets json failed: {e}"));
        assert_eq!(presets.len(), 6);
        assert!(presets.iter().any(|p| p["id"] == "architect"));
        assert!(presets.iter().any(|p| p["id"] == "security"));

        // 2. Create an instance with role "security"
        let create_resp = client
            .post(format!("{}/api/instances", base_url))
            .json(&json!({
                "name": "SecBot",
                "kind": "claude",
                "transport": "http-rpc",
                "endpoint_url": "http://127.0.0.1:13285/rpc",
                "role": "security",
                "role_prompt": "Vérifie strictement les accès rootless."
            }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("create instance failed: {e}"));
        assert_eq!(create_resp.status(), StatusCode::CREATED);
        let created: Value = create_resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("created json failed: {e}"));
        let inst_id = created["id"]
            .as_str()
            .unwrap_or_else(|| panic!("missing id"));
        assert_eq!(created["role"], "security");
        assert_eq!(
            created["role_prompt"],
            "Vérifie strictement les accès rootless."
        );

        // 3. Update role to "reviewer" with empty role_prompt (should fallback to default reviewer prompt)
        let update_resp = client
            .put(format!("{}/api/instances/{}", base_url, inst_id))
            .json(&json!({
                "role": "reviewer",
                "role_prompt": ""
            }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("update instance failed: {e}"));
        assert_eq!(update_resp.status(), StatusCode::OK);
        let updated: Value = update_resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("updated json failed: {e}"));
        assert_eq!(updated["role"], "reviewer");
        assert!(
            updated["role_prompt"]
                .as_str()
                .unwrap_or("")
                .contains("Code Reviewer")
        );
    }

    #[tokio::test]
    async fn test_instance_onboarding_and_role_orchestration() {
        let mock_rpc_url = spawn_mock_bridge().await;
        let (base_url, state) = spawn_test_hub("/tmp").await;
        let client = &state.http_client;

        // Register a local instance backed by our mock bridge
        let create_resp = client
            .post(format!("{}/api/instances", base_url))
            .json(&json!({
                "name": "LocalArch",
                "kind": "claude",
                "transport": "http-rpc",
                "endpoint_url": mock_rpc_url,
                "bridge_url": mock_rpc_url,
                "role": "architect"
            }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("create failed: {e}"));
        let created: Value = create_resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("json failed: {e}"));
        let inst_id = created["id"].as_str().unwrap_or("");

        // Onboard the instance
        let onboard_resp = client
            .post(format!("{}/api/instances/{}/onboard", base_url, inst_id))
            .json(&json!({ "channel_name": "architecture-room" }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("onboard failed: {e}"));
        assert_eq!(onboard_resp.status(), StatusCode::OK);
        let onboard_val: Value = onboard_resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("onboard json failed: {e}"));
        assert!(
            onboard_val["onboarding_prompt"]
                .as_str()
                .unwrap_or("")
                .contains("AGENT ONBOARDING")
        );
        assert!(
            onboard_val["reply"]
                .as_str()
                .unwrap_or("")
                .starts_with("ACK_ROLE_PROMPT")
        );

        // Orchestration auto-dispatch with role injection
        let chan_resp = client
            .post(format!("{}/api/orchestration/channels", base_url))
            .json(&json!({
                "name": "arch-dispatch-test",
                "topic": "Test role injection",
                "retention_days": 7,
                "max_messages": 50
            }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("create channel failed: {e}"));
        let chan: Value = chan_resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("chan json failed: {e}"));
        let chan_id = chan["id"].as_str().unwrap_or("");

        let pub_resp = client
            .post(format!(
                "{}/api/orchestration/channels/{}/messages",
                base_url, chan_id
            ))
            .json(&json!({
                "sender_name": "User",
                "recipient_name": "@LocalArch",
                "content": "@LocalArch propose une architecture modulaire",
                "auto_dispatch": true
            }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("publish failed: {e}"));
        assert_eq!(pub_resp.status(), StatusCode::OK);
        let pub_val: Value = pub_resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("pub json failed: {e}"));
        let replies = pub_val["replies"]
            .as_array()
            .unwrap_or_else(|| panic!("missing replies array"));
        assert_eq!(replies.len(), 1);
        assert_eq!(replies[0]["sender_name"], "LocalArch");
        assert!(
            replies[0]["content"]
                .as_str()
                .unwrap_or("")
                .contains("ACK_ROLE_PROMPT")
        );
    }

    #[tokio::test]
    async fn test_channel_retention_prune_and_markdown_export() {
        let (base_url, state) = spawn_test_hub("/tmp").await;
        let client = &state.http_client;

        // Create channel with max_messages = 3
        let chan_resp = client
            .post(format!("{}/api/orchestration/channels", base_url))
            .json(&json!({
                "name": "retention-room",
                "topic": "Purge automatique",
                "retention_days": 10,
                "max_messages": 3
            }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("create channel failed: {e}"));
        assert_eq!(chan_resp.status(), StatusCode::CREATED);
        let chan: Value = chan_resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("chan json failed: {e}"));
        let chan_id = chan["id"].as_str().unwrap_or("");
        assert_eq!(chan["retention_days"], 10);
        assert_eq!(chan["max_messages"], 3);

        // Publish 5 messages
        for i in 1..=5 {
            let r = client
                .post(format!(
                    "{}/api/orchestration/channels/{}/messages",
                    base_url, chan_id
                ))
                .json(&json!({
                    "sender_name": "Tester",
                    "content": format!("Message numero {}", i),
                    "auto_dispatch": false
                }))
                .send()
                .await
                .unwrap_or_else(|e| panic!("msg {i} failed: {e}"));
            assert_eq!(r.status(), StatusCode::OK);
        }

        // Verify only the last 3 messages (3, 4, 5) are retained
        let msgs_resp = client
            .get(format!(
                "{}/api/orchestration/channels/{}/messages",
                base_url, chan_id
            ))
            .send()
            .await
            .unwrap_or_else(|e| panic!("get messages failed: {e}"));
        let msgs: Vec<Value> = msgs_resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("msgs json failed: {e}"));
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0]["content"], "Message numero 3");
        assert_eq!(msgs[2]["content"], "Message numero 5");

        // Update max_messages to 2 and trigger manual prune
        let upd_resp = client
            .put(format!(
                "{}/api/orchestration/channels/{}",
                base_url, chan_id
            ))
            .json(&json!({ "max_messages": 2 }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("update channel failed: {e}"));
        assert_eq!(upd_resp.status(), StatusCode::OK);

        let prune_resp = client
            .post(format!(
                "{}/api/orchestration/channels/{}/prune",
                base_url, chan_id
            ))
            .send()
            .await
            .unwrap_or_else(|e| panic!("prune failed: {e}"));
        let prune_val: Value = prune_resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("prune json failed: {e}"));
        assert_eq!(prune_val["pruned"], 0);

        // Export Markdown
        let export_resp = client
            .get(format!(
                "{}/api/orchestration/channels/{}/export",
                base_url, chan_id
            ))
            .send()
            .await
            .unwrap_or_else(|e| panic!("export failed: {e}"));
        assert_eq!(export_resp.status(), StatusCode::OK);
        let md = export_resp
            .text()
            .await
            .unwrap_or_else(|e| panic!("export text failed: {e}"));
        assert!(md.contains("# Orchestration Channel: #retention-room"));
        assert!(!md.contains("Message numero 3"));
        assert!(md.contains("Message numero 4"));
        assert!(md.contains("Message numero 5"));
    }

    #[tokio::test]
    async fn test_git_status_commit_and_revert_lifecycle() {
        let tmp = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir failed: {e}"));
        let repo_dir = tmp.path().to_string_lossy().to_string();

        // Initialize temporary git repo
        let _ = tokio::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo_dir)
            .output()
            .await;
        let _ = tokio::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&repo_dir)
            .output()
            .await;
        let _ = tokio::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&repo_dir)
            .output()
            .await;

        let file_path = tmp.path().join("hello.txt");
        std::fs::write(&file_path, "ligne 1\n")
            .unwrap_or_else(|e| panic!("write hello.txt failed: {e}"));
        let _ = tokio::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(&repo_dir)
            .output()
            .await;
        let _ = tokio::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&repo_dir)
            .output()
            .await;

        let (base_url, state) = spawn_test_hub(&repo_dir).await;
        let client = &state.http_client;

        // Modify hello.txt
        std::fs::write(&file_path, "ligne 1\nligne 2 ajoutee\n")
            .unwrap_or_else(|e| panic!("modify hello.txt failed: {e}"));

        // 1. Check /api/git/status
        let status_resp = client
            .get(format!("{}/api/git/status", base_url))
            .send()
            .await
            .unwrap_or_else(|e| panic!("git status failed: {e}"));
        assert_eq!(status_resp.status(), StatusCode::OK);
        let status_val: Value = status_resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("git status json failed: {e}"));
        assert_eq!(
            status_val["files"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0),
            1
        );
        assert!(
            status_val["diff"]
                .as_str()
                .unwrap_or("")
                .contains("+ligne 2 ajoutee")
        );

        // 2. Revert hello.txt via /api/git/revert
        let rev_resp = client
            .post(format!("{}/api/git/revert", base_url))
            .json(&json!({ "file": "hello.txt" }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("git revert failed: {e}"));
        assert_eq!(rev_resp.status(), StatusCode::OK);

        let status_after_rev: Value = client
            .get(format!("{}/api/git/status", base_url))
            .send()
            .await
            .unwrap_or_else(|e| panic!("status after revert failed: {e}"))
            .json()
            .await
            .unwrap_or_else(|e| panic!("json failed: {e}"));
        assert_eq!(
            status_after_rev["files"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(99),
            0
        );

        // 3. Modify again and commit via /api/git/commit
        std::fs::write(&file_path, "ligne 1\nligne 2 validee\n")
            .unwrap_or_else(|e| panic!("write failed: {e}"));
        let commit_resp = client
            .post(format!("{}/api/git/commit", base_url))
            .json(&json!({ "message": "feat: add second line" }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("git commit failed: {e}"));
        assert_eq!(commit_resp.status(), StatusCode::OK);
        let commit_val: Value = commit_resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("commit json failed: {e}"));
        assert_eq!(commit_val["committed"], true);
    }

    #[tokio::test]
    async fn test_native_terminal_websocket_streaming_and_raw_input() {
        let mock_rpc_url = spawn_mock_bridge().await;
        let (base_url, state) = spawn_test_hub("/tmp").await;
        let client = &state.http_client;

        let create_resp = client
            .post(format!("{}/api/instances", base_url))
            .json(&json!({
                "name": "TermInstance",
                "kind": "claude",
                "transport": "http-rpc",
                "endpoint_url": mock_rpc_url,
                "bridge_url": mock_rpc_url
            }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("create instance failed: {e}"));
        let created: Value = create_resp
            .json()
            .await
            .unwrap_or_else(|e| panic!("json failed: {e}"));
        let inst_id = created["id"].as_str().unwrap_or("");

        let ws_url = format!(
            "{}/ws/instances/{}/terminal",
            base_url.replacen("http://", "ws://", 1),
            inst_id
        );
        let (mut ws, _) = connect_async(&ws_url)
            .await
            .unwrap_or_else(|e| panic!("terminal ws connect failed: {e}"));

        // 1. Receive initial ANSI screen frame
        let first_msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
            .await
            .unwrap_or_else(|e| panic!("timeout waiting for initial screen: {e}"))
            .unwrap_or_else(|| panic!("ws stream closed"))
            .unwrap_or_else(|e| panic!("ws frame error: {e}"));
        let first_txt = first_msg
            .into_text()
            .unwrap_or_else(|e| panic!("not text: {e}"));
        let first_json: Value = serde_json::from_str(&first_txt)
            .unwrap_or_else(|e| panic!("invalid json: {e}"));
        assert_eq!(first_json["type"], "screen");
        assert!(
            first_json["data"]
                .as_str()
                .unwrap_or("")
                .contains("thenewuser@container")
        );

        // 2. Send raw input keystroke and verify updated screen frame
        ws.send(TungMessage::Text(
            json!({
                "type": "raw",
                "data": "echo hello",
                "special": false
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap_or_else(|e| panic!("ws send failed: {e}"));

        let reply_msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
            .await
            .unwrap_or_else(|e| panic!("timeout waiting for raw ack screen: {e}"))
            .unwrap_or_else(|| panic!("ws closed"))
            .unwrap_or_else(|e| panic!("ws error: {e}"));
        let reply_txt = reply_msg
            .into_text()
            .unwrap_or_else(|e| panic!("not text: {e}"));
        let reply_json: Value = serde_json::from_str(&reply_txt)
            .unwrap_or_else(|e| panic!("invalid json: {e}"));
        assert_eq!(reply_json["type"], "screen");
        assert!(
            reply_json["data"]
                .as_str()
                .unwrap_or("")
                .contains("RAW_ACK")
        );
    }
}
