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
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{env, net::SocketAddr, path::Path as FsPath, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpStream, UnixStream},
    time::timeout,
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message as TungMessage},
};
use tracing::{error, info, warn};
use url::Url;

const INDEX_HTML: &str = include_str!("../static/index.html");
const STYLE_CSS: &str = include_str!("../static/style.css");
const APP_JS: &str = include_str!("../static/app.js");

#[derive(Clone)]
struct AppState {
    goose_http_url: String,
    goose_ws_url: String,
    goose_secret: Option<String>,
    codex_ws_url: String,
    codex_unix_socket: String,
    claude_rpc_url: String,
    antigravity_rpc_url: String,
    bridge_secret: Option<String>,
    default_cwd: String,
    http_client: reqwest::Client,
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

#[derive(Serialize)]
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

#[derive(Deserialize)]
struct CodexWsQuery {
    mode: Option<String>,
    target: Option<String>,
}

#[derive(Deserialize)]
struct GooseWsQuery {
    target: Option<String>,
    token: Option<String>,
}

#[derive(Deserialize)]
struct AgentAskRequest {
    agent: String,
    prompt: String,
    cwd: Option<String>,
    mode: Option<String>,
}

#[derive(Deserialize)]
struct AgentTmuxRequest {
    agent: String,
    lines: Option<u32>,
}

#[tokio::main]
async fn main() {
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

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .expect("Failed to initialize HTTP client");

    let state = Arc::new(AppState {
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
    });

    let app = build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(
        "Simply Agent Hub listening on http://{} (plain HTTP behind Traefik)",
        addr
    );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");
    axum::serve(listener, app)
        .await
        .expect("Server terminated unexpectedly");
}

fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/style.css", get(serve_css))
        .route("/app.js", get(serve_js))
        .route("/healthz", get(serve_healthz))
        .route("/ready", get(serve_healthz))
        .route("/api/config", get(serve_config))
        .route("/api/status", get(serve_status))
        .route("/api/agents/ask", post(handle_agent_ask))
        .route("/api/agents/tmux", post(handle_agent_tmux))
        .route("/ws/codex", get(ws_codex_handler))
        .route("/ws/goose", get(ws_goose_handler))
        .route("/api/goose/*path", any(proxy_goose_http))
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

async fn probe_tcp_url(raw_url: &str, default_port: u16) -> TargetHealth {
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

    match timeout(Duration::from_millis(750), TcpStream::connect(&target)).await {
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

async fn serve_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (goose, codex_ws, claude, antigravity) = tokio::join!(
        probe_tcp_url(&state.goose_ws_url, 3284),
        probe_tcp_url(&state.codex_ws_url, 4500),
        probe_tcp_url(&state.claude_rpc_url, 3285),
        probe_tcp_url(&state.antigravity_rpc_url, 3286),
    );

    let codex_unix = if FsPath::new(&state.codex_unix_socket).exists() {
        match timeout(
            Duration::from_millis(500),
            UnixStream::connect(&state.codex_unix_socket),
        )
        .await
        {
            Ok(Ok(_)) => TargetHealth {
                online: true,
                detail: format!("Socket ready ({})", state.codex_unix_socket),
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
            detail: format!("Not mounted ({})", state.codex_unix_socket),
        }
    };

    Json(StatusResponse {
        goose,
        codex_ws,
        codex_unix,
        claude,
        antigravity,
    })
}

// ============================================================================
// INTER-AGENT ORCHESTRATION & UNIFIED ASK API (/api/agents/ask, /api/agents/tmux)
// ============================================================================
async fn handle_agent_ask(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AgentAskRequest>,
) -> impl IntoResponse {
    let agent = req.agent.trim().to_lowercase();
    let cwd = req.cwd.unwrap_or_else(|| state.default_cwd.clone());
    let mode = req.mode.unwrap_or_else(|| "auto".to_string());

    let result = match agent.as_str() {
        "codex" => ask_codex_oneshot(&state, &req.prompt, &cwd).await,
        "goose" => ask_goose_oneshot(&state, &req.prompt, &cwd).await,
        "claude" => {
            ask_http_rpc_peer(&state, &state.claude_rpc_url, "claude", &req.prompt, &cwd, &mode)
                .await
        }
        "antigravity" | "agy" => {
            ask_http_rpc_peer(
                &state,
                &state.antigravity_rpc_url,
                "antigravity",
                &req.prompt,
                &cwd,
                &mode,
            )
            .await
        }
        other => Err(format!("Agent inconnu : {}", other)),
    };

    match result {
        Ok(reply) => (
            StatusCode::OK,
            Json(json!({ "agent": agent, "reply": reply })),
        ),
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "agent": agent, "error": err })),
        ),
    }
}

async fn handle_agent_tmux(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AgentTmuxRequest>,
) -> impl IntoResponse {
    let agent = req.agent.trim().to_lowercase();
    let url = if agent == "antigravity" || agent == "agy" {
        &state.antigravity_rpc_url
    } else {
        &state.claude_rpc_url
    };

    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tmux/capture",
        "params": { "lines": req.lines.unwrap_or(120) }
    });

    let mut builder = state.http_client.post(url).json(&payload);
    if let Some(ref sec) = state.bridge_secret {
        builder = builder.header("X-Secret-Key", sec);
    }

    match builder.send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(val) => {
                let output = val["result"]["output"].as_str().unwrap_or("").to_string();
                (StatusCode::OK, Json(json!({ "agent": agent, "output": output })))
            }
            Err(e) => (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("Invalid JSON from {}: {}", agent, e) })),
            ),
        },
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("Unreachable {} bridge ({}): {}", agent, url, e) })),
        ),
    }
}

async fn ask_http_rpc_peer(
    state: &AppState,
    url: &str,
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
    let mut req = state.http_client.post(url).json(&payload);
    if let Some(ref sec) = state.bridge_secret {
        req = req.header("X-Secret-Key", sec);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| format!("Impossible de joindre {} ({}): {}", agent_name, url, e))?;
    let val: Value = resp
        .json()
        .await
        .map_err(|e| format!("Réponse JSON invalide de {}: {}", agent_name, e))?;
    if let Some(err) = val.get("error") {
        return Err(err.to_string());
    }
    Ok(val["result"]["reply"]
        .as_str()
        .unwrap_or("")
        .to_string())
}

async fn ask_codex_oneshot(state: &AppState, prompt: &str, cwd: &str) -> Result<String, String> {
    let (mut ws, _) = connect_async(&state.codex_ws_url)
        .await
        .map_err(|e| format!("Connexion Codex WS échouée ({}): {}", state.codex_ws_url, e))?;

    // 1. initialize
    ws.send(TungMessage::Text(
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "clientInfo": { "name": "agent-hub-bridge", "version": "1.0.0" } }
        })
        .to_string(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    let _ = ws.next().await;

    ws.send(TungMessage::Text(
        json!({ "jsonrpc": "2.0", "method": "initialized", "params": {} }).to_string(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    // 2. thread/start
    ws.send(TungMessage::Text(
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "thread/start",
            "params": { "cwd": cwd }
        })
        .to_string(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    let mut thread_id = "default".to_string();
    while let Some(Ok(TungMessage::Text(txt))) = ws.next().await {
        if let Ok(v) = serde_json::from_str::<Value>(&txt) {
            if v.get("id") == Some(&json!(2)) {
                thread_id = v["result"]["thread"]["id"]
                    .as_str()
                    .or_else(|| v["result"]["threadId"].as_str())
                    .unwrap_or("default")
                    .to_string();
                break;
            }
        }
    }

    // 3. turn/start
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
        .to_string(),
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
            } else if method == "turn/completed" {
                break;
            } else if v.get("id") == Some(&json!(3)) && !collected.is_empty() {
                break;
            }
        }
    }

    Ok(collected)
}

async fn ask_goose_oneshot(state: &AppState, prompt: &str, cwd: &str) -> Result<String, String> {
    let mut req = state
        .goose_ws_url
        .as_str()
        .into_client_request()
        .map_err(|e| e.to_string())?;

    if let Some(ref token) = state.goose_secret {
        if let Ok(val) = token.parse() {
            req.headers_mut().insert("X-Secret-Key", val);
        }
    }

    let (mut ws, _) = connect_async(req)
        .await
        .map_err(|e| format!("Connexion Goose ACP échouée ({}): {}", state.goose_ws_url, e))?;

    // 1. initialize
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
        .to_string(),
    ))
    .await
    .map_err(|e| e.to_string())?;
    let _ = ws.next().await;

    // 2. session/new
    ws.send(TungMessage::Text(
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "session/new",
            "params": { "cwd": cwd, "mcpServers": [] }
        })
        .to_string(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    let mut session_id = "default".to_string();
    while let Some(Ok(TungMessage::Text(txt))) = ws.next().await {
        if let Ok(v) = serde_json::from_str::<Value>(&txt) {
            if v.get("id") == Some(&json!(2)) {
                session_id = v["result"]["sessionId"]
                    .as_str()
                    .unwrap_or("default")
                    .to_string();
                break;
            }
        }
    }

    // 3. session/prompt
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
        .to_string(),
    ))
    .await
    .map_err(|e| e.to_string())?;

    let mut collected = String::new();
    while let Some(Ok(TungMessage::Text(txt))) = ws.next().await {
        if let Ok(v) = serde_json::from_str::<Value>(&txt) {
            if v["method"].as_str() == Some("session/update") {
                let upd = &v["params"]["update"];
                if upd["sessionUpdate"].as_str() == Some("agent_message_chunk") {
                    if let Some(t) = upd["content"]["text"].as_str() {
                        collected.push_str(t);
                    }
                }
            } else if v.get("id") == Some(&json!(3)) {
                break;
            }
        }
    }

    Ok(collected)
}

// ============================================================================
// CODEX BRIDGE (/ws/codex -> ws://codex:4500 OR unix:///run/codex/app.sock)
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
    let mode = query.mode.as_deref().unwrap_or("ws");
    let target = query
        .target
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            if mode == "unix" {
                state.codex_unix_socket.clone()
            } else {
                state.codex_ws_url.clone()
            }
        });

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
                    &format!("Impossible de joindre le socket Unix Codex ({}): {}", sock_path, e),
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
                    &format!("Impossible de joindre Codex WebSocket ({}): {}", target, e),
                )
                .await;
            }
        }
    }
}

// ============================================================================
// GOOSE ACP BRIDGE (/ws/goose -> ws://goose:3284/acp with X-Secret-Key)
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
    let target_url = query
        .target
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| state.goose_ws_url.clone());

    let secret = query
        .token
        .filter(|s| !s.trim().is_empty())
        .or_else(|| state.goose_secret.clone());

    let mut req = match target_url.as_str().into_client_request() {
        Ok(r) => r,
        Err(e) => {
            let _ = send_bridge_error(
                &mut client_ws,
                &format!("URL Goose invalide ({}): {}", target_url, e),
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
                &format!("Impossible de joindre Goose ACP ({}): {}", target_url, e),
            )
            .await;
        }
    }
}

// ============================================================================
// GOOSE HTTP PROXY (/api/goose/*path -> http://goose:3284/*path)
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

    if let Some(ct) = headers.get("content-type") {
        if let Ok(ct_str) = ct.to_str() {
            req = req.header("Content-Type", ct_str);
        }
    }
    if let Some(accept) = headers.get("accept") {
        if let Ok(acc_str) = accept.to_str() {
            req = req.header("Accept", acc_str);
        }
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
// GENERIC BIDIRECTIONAL STREAM BRIDGES
// ============================================================================
async fn send_bridge_error(ws: &mut WebSocket, message: &str) -> Result<(), axum::Error> {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "bridge/error",
        "params": { "message": message }
    });
    ws.send(AxumWsMessage::Text(payload.to_string())).await
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
                AxumWsMessage::Text(t) => TungMessage::Text(t),
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
                TungMessage::Text(t) => AxumWsMessage::Text(t),
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
                .send(AxumWsMessage::Text(trimmed.to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::{TcpListener, UnixListener};

    #[tokio::test]
    async fn test_end_to_end_agent_hub_bridges_and_inter_agent_api() {
        // 1. Spawn Mock Goose Server (validating X-Secret-Key on both HTTP POST /acp and WS /acp)
        let goose_app = Router::new().route(
            "/acp",
            get(|headers: HeaderMap, ws: WebSocketUpgrade| async move {
                let key = headers
                    .get("x-secret-key")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");
                assert_eq!(key, "test-goose-secret-123");
                ws.on_upgrade(|mut socket| async move {
                    while let Some(Ok(AxumWsMessage::Text(text))) = socket.next().await {
                        let req: Value = serde_json::from_str(&text).unwrap();
                        let id = req.get("id").cloned().unwrap_or(json!(1));
                        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
                        match method {
                            "initialize" => {
                                let reply = json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": { "protocolVersion": 1, "agentInfo": { "name": "goose-mock" } }
                                });
                                socket.send(AxumWsMessage::Text(reply.to_string())).await.unwrap();
                            }
                            "session/new" => {
                                let reply = json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": { "sessionId": "goose-sess-42" }
                                });
                                socket.send(AxumWsMessage::Text(reply.to_string())).await.unwrap();
                            }
                            "session/prompt" => {
                                let chunk = json!({
                                    "jsonrpc": "2.0",
                                    "method": "session/update",
                                    "params": {
                                        "sessionId": "goose-sess-42",
                                        "update": {
                                            "sessionUpdate": "agent_message_chunk",
                                            "content": { "type": "text", "text": "Bonjour depuis Goose ACP!" }
                                        }
                                    }
                                });
                                socket.send(AxumWsMessage::Text(chunk.to_string())).await.unwrap();
                                let reply = json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": { "stopReason": "end_turn" }
                                });
                                socket.send(AxumWsMessage::Text(reply.to_string())).await.unwrap();
                            }
                            _ => {}
                        }
                    }
                })
            })
            .post(|headers: HeaderMap, Json(req): Json<Value>| async move {
                let key = headers
                    .get("x-secret-key")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");
                assert_eq!(key, "test-goose-secret-123");
                let id = req.get("id").cloned().unwrap_or(json!(1));
                Json(json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": { "protocolVersion": 1, "httpProxy": "ok" }
                }))
            }),
        );

        let goose_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let goose_addr = goose_listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(goose_listener, goose_app).await.unwrap();
        });

        // 2. Spawn Mock Codex WebSocket Server
        let codex_ws_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let codex_ws_addr = codex_ws_listener.local_addr().unwrap();
        tokio::spawn(async move {
            while let Ok((stream, _)) = codex_ws_listener.accept().await {
                tokio::spawn(async move {
                    let Ok(mut ws) = tokio_tungstenite::accept_async(stream).await else {
                        return;
                    };
                    while let Some(Ok(TungMessage::Text(text))) = ws.next().await {
                        let req: Value = serde_json::from_str(&text).unwrap();
                        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
                        if let Some(id) = req.get("id").cloned() {
                            match method {
                                "initialize" => {
                                    let reply = json!({
                                        "jsonrpc": "2.0",
                                        "id": id,
                                        "result": { "serverInfo": { "name": "codex-ws-mock" } }
                                    });
                                    ws.send(TungMessage::Text(reply.to_string())).await.unwrap();
                                }
                                "thread/start" => {
                                    let reply = json!({
                                        "jsonrpc": "2.0",
                                        "id": id,
                                        "result": { "thread": { "id": "thread-ws-99" } }
                                    });
                                    ws.send(TungMessage::Text(reply.to_string())).await.unwrap();
                                }
                                "turn/start" => {
                                    let delta = json!({
                                        "jsonrpc": "2.0",
                                        "method": "item/agentMessage/delta",
                                        "params": { "delta": "Réponse Codex WebSocket!" }
                                    });
                                    ws.send(TungMessage::Text(delta.to_string())).await.unwrap();
                                    let done = json!({
                                        "jsonrpc": "2.0",
                                        "method": "turn/completed",
                                        "params": {}
                                    });
                                    ws.send(TungMessage::Text(done.to_string())).await.unwrap();
                                }
                                _ => {}
                            }
                        }
                    }
                });
            }
        });

        // 3. Spawn Mock Claude/Antigravity RPC Bridge Server
        let peer_app = Router::new().route(
            "/rpc",
            post(|Json(req): Json<Value>| async move {
                let id = req.get("id").cloned().unwrap_or(json!(1));
                let method = req["method"].as_str().unwrap_or("");
                if method == "tmux/capture" {
                    Json(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": { "output": "thenewuser@claude:/srv$ ready" }
                    }))
                } else {
                    let prompt = req["params"]["prompt"].as_str().unwrap_or("");
                    Json(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": { "reply": format!("Claude a analysé: {}", prompt) }
                    }))
                }
            }),
        );
        let peer_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let peer_addr = peer_listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(peer_listener, peer_app).await.unwrap();
        });

        // 4. Spawn Mock Codex Unix Socket Server
        let sock_path = format!("/tmp/test-codex-hub-{}.sock", std::process::id());
        let _ = std::fs::remove_file(&sock_path);
        let unix_listener = UnixListener::bind(&sock_path).unwrap();
        let sock_path_clone = sock_path.clone();
        tokio::spawn(async move {
            while let Ok((stream, _)) = unix_listener.accept().await {
                tokio::spawn(async move {
                    let (read_half, mut write_half) = stream.into_split();
                    let mut lines = BufReader::new(read_half).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        let req: Value = serde_json::from_str(&line).unwrap();
                        if let Some(id) = req.get("id").cloned() {
                            let reply = json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": { "data": [{ "id": "thread-unix-1" }] }
                            });
                            let mut out = reply.to_string();
                            out.push('\n');
                            write_half.write_all(out.as_bytes()).await.unwrap();
                        }
                    }
                });
            }
        });

        // 5. Spawn Simply Agent Hub
        let state = Arc::new(AppState {
            goose_http_url: format!("http://{}", goose_addr),
            goose_ws_url: format!("ws://{}/acp", goose_addr),
            goose_secret: Some("test-goose-secret-123".to_string()),
            codex_ws_url: format!("ws://{}", codex_ws_addr),
            codex_unix_socket: sock_path.clone(),
            claude_rpc_url: format!("http://{}/rpc", peer_addr),
            antigravity_rpc_url: format!("http://{}/rpc", peer_addr),
            bridge_secret: None,
            default_cwd: "/srv".to_string(),
            http_client: reqwest::Client::new(),
        });

        let hub_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let hub_addr = hub_listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(hub_listener, build_router(state)).await.unwrap();
        });

        let client = reqwest::Client::new();
        let base_http = format!("http://{}", hub_addr);

        // Verify /ready and /api/status across all 4 agents
        let ready: Value = client
            .get(format!("{}/ready", base_http))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(ready["status"], "ok");

        let status: Value = client
            .get(format!("{}/api/status", base_http))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(status["goose"]["online"], true);
        assert_eq!(status["codex_ws"]["online"], true);
        assert_eq!(status["codex_unix"]["online"], true);
        assert_eq!(status["claude"]["online"], true);
        assert_eq!(status["antigravity"]["online"], true);

        // Verify Inter-Agent Unified Ask API (/api/agents/ask) for Claude, Codex, and Goose!
        let ask_claude: Value = client
            .post(format!("{}/api/agents/ask", base_http))
            .json(&json!({ "agent": "claude", "prompt": "Vérifie /srv" }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(ask_claude["reply"], "Claude a analysé: Vérifie /srv");

        let ask_codex: Value = client
            .post(format!("{}/api/agents/ask", base_http))
            .json(&json!({ "agent": "codex", "prompt": "Corrige le bug" }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(ask_codex["reply"], "Réponse Codex WebSocket!");

        let ask_goose: Value = client
            .post(format!("{}/api/agents/ask", base_http))
            .json(&json!({ "agent": "goose", "prompt": "Exécute les tests" }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(ask_goose["reply"], "Bonjour depuis Goose ACP!");

        // Verify Tmux Capture API (/api/agents/tmux)
        let tmux_res: Value = client
            .post(format!("{}/api/agents/tmux", base_http))
            .json(&json!({ "agent": "claude", "lines": 50 }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(tmux_res["output"], "thenewuser@claude:/srv$ ready");

        let _ = std::fs::remove_file(&sock_path_clone);
    }
}
