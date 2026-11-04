use axum::{
    body::Body,
    extract::{
        ws::{Message as AxumWsMessage, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{HeaderMap, Method, Response, StatusCode},
    response::{Html, IntoResponse},
    routing::{any, get},
    Json, Router,
};
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
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
        default_cwd,
        http_client,
    });

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/style.css", get(serve_css))
        .route("/app.js", get(serve_js))
        .route("/healthz", get(serve_healthz))
        .route("/api/config", get(serve_config))
        .route("/api/status", get(serve_status))
        .route("/ws/codex", get(ws_codex_handler))
        .route("/ws/goose", get(ws_goose_handler))
        .route("/api/goose/*path", any(proxy_goose_http))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Agent Hub listening on http://{} (plain HTTP behind Traefik)", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");
    axum::serve(listener, app)
        .await
        .expect("Server terminated unexpectedly");
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

    match timeout(Duration::from_millis(800), TcpStream::connect(&target)).await {
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
    let (goose, codex_ws) = tokio::join!(
        probe_tcp_url(&state.goose_ws_url, 3284),
        probe_tcp_url(&state.codex_ws_url, 4500),
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
    })
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
