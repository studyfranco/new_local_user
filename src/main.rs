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

    let app = build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Agent Hub listening on http://{} (plain HTTP behind Traefik)", addr);

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
        .route("/api/config", get(serve_config))
        .route("/api/status", get(serve_status))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use tokio::net::{TcpListener, UnixListener};

    #[tokio::test]
    async fn test_end_to_end_agent_hub_bridges_and_api() {
        // 1. Spawn Mock Goose Server (validating X-Secret-Key on both HTTP POST /acp and WS /acp)
        let goose_app = Router::new().route(
            "/acp",
            get(|headers: HeaderMap, ws: WebSocketUpgrade| async move {
                let key = headers
                    .get("x-secret-key")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");
                assert_eq!(key, "test-goose-secret-123", "Goose WS must receive X-Secret-Key");
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
                assert_eq!(key, "test-goose-secret-123", "Goose HTTP must receive X-Secret-Key");
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

        // 2. Spawn Mock Codex WebSocket Server (ws://127.0.0.1:<port>)
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
                                    let reply = json!({
                                        "jsonrpc": "2.0",
                                        "id": id,
                                        "result": { "turn": { "id": "turn-1", "status": "completed" } }
                                    });
                                    ws.send(TungMessage::Text(reply.to_string())).await.unwrap();
                                }
                                _ => {}
                            }
                        }
                    }
                });
            }
        });

        // 3. Spawn Mock Codex Unix Socket Server (/tmp/test-codex-hub.sock)
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
                                "result": { "data": [{ "id": "thread-unix-1", "preview": "Unix Socket Thread" }] }
                            });
                            let mut out = reply.to_string();
                            out.push('\n');
                            write_half.write_all(out.as_bytes()).await.unwrap();
                        }
                    }
                });
            }
        });

        // 4. Spawn Agent Hub Server pointing to the 3 mock backends
        let state = Arc::new(AppState {
            goose_http_url: format!("http://{}", goose_addr),
            goose_ws_url: format!("ws://{}/acp", goose_addr),
            goose_secret: Some("test-goose-secret-123".to_string()),
            codex_ws_url: format!("ws://{}", codex_ws_addr),
            codex_unix_socket: sock_path.clone(),
            default_cwd: "/srv".to_string(),
            http_client: reqwest::Client::new(),
        });

        let hub_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let hub_addr = hub_listener.local_addr().unwrap();
        let hub_app = build_router(state);
        tokio::spawn(async move {
            axum::serve(hub_listener, hub_app).await.unwrap();
        });

        let client = reqwest::Client::new();
        let base_http = format!("http://{}", hub_addr);
        let base_ws = format!("ws://{}", hub_addr);

        // 5a. Test Static Assets & Healthz
        let index_res = client.get(format!("{}/", base_http)).send().await.unwrap();
        assert_eq!(index_res.status(), 200);
        let index_body = index_res.text().await.unwrap();
        assert!(index_body.contains("Agent Hub"));

        let js_res = client.get(format!("{}/app.js", base_http)).send().await.unwrap();
        assert_eq!(js_res.status(), 200);

        let health_res: Value = client
            .get(format!("{}/healthz", base_http))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(health_res["status"], "ok");

        // 5b. Test /api/config & /api/status
        let cfg_res: Value = client
            .get(format!("{}/api/config", base_http))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(cfg_res["has_goose_secret"], true);
        assert_eq!(cfg_res["codex_unix_available"], true);

        let status_res: Value = client
            .get(format!("{}/api/status", base_http))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(status_res["goose"]["online"], true);
        assert_eq!(status_res["codex_ws"]["online"], true);
        assert_eq!(status_res["codex_unix"]["online"], true);

        // 5c. Test Goose HTTP Proxy (/api/goose/acp)
        let goose_http_res: Value = client
            .post(format!("{}/api/goose/acp", base_http))
            .json(&json!({ "jsonrpc": "2.0", "id": 10, "method": "initialize", "params": {} }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(goose_http_res["result"]["httpProxy"], "ok");

        // 5d. Test Goose WebSocket Bridge (/ws/goose)
        let (mut goose_ws, _) = connect_async(format!("{}/ws/goose", base_ws)).await.unwrap();
        goose_ws
            .send(TungMessage::Text(
                json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }).to_string(),
            ))
            .await
            .unwrap();
        let init_msg = goose_ws.next().await.unwrap().unwrap().into_text().unwrap();
        let init_val: Value = serde_json::from_str(&init_msg).unwrap();
        assert_eq!(init_val["result"]["agentInfo"]["name"], "goose-mock");

        goose_ws
            .send(TungMessage::Text(
                json!({ "jsonrpc": "2.0", "id": 2, "method": "session/prompt", "params": {} }).to_string(),
            ))
            .await
            .unwrap();
        let chunk_msg = goose_ws.next().await.unwrap().unwrap().into_text().unwrap();
        let chunk_val: Value = serde_json::from_str(&chunk_msg).unwrap();
        assert_eq!(
            chunk_val["params"]["update"]["content"]["text"],
            "Bonjour depuis Goose ACP!"
        );

        // 5e. Test Codex WebSocket Bridge (/ws/codex?mode=ws)
        let (mut codex_ws, _) = connect_async(format!("{}/ws/codex?mode=ws", base_ws))
            .await
            .unwrap();
        codex_ws
            .send(TungMessage::Text(
                json!({ "jsonrpc": "2.0", "id": 1, "method": "turn/start", "params": {} }).to_string(),
            ))
            .await
            .unwrap();
        let delta_msg = codex_ws.next().await.unwrap().unwrap().into_text().unwrap();
        let delta_val: Value = serde_json::from_str(&delta_msg).unwrap();
        assert_eq!(delta_val["params"]["delta"], "Réponse Codex WebSocket!");

        // 5f. Test Codex Unix Socket Bridge (/ws/codex?mode=unix)
        let (mut codex_unix_ws, _) = connect_async(format!("{}/ws/codex?mode=unix", base_ws))
            .await
            .unwrap();
        codex_unix_ws
            .send(TungMessage::Text(
                json!({ "jsonrpc": "2.0", "id": 77, "method": "thread/list", "params": {} }).to_string(),
            ))
            .await
            .unwrap();
        let unix_msg = codex_unix_ws
            .next()
            .await
            .unwrap()
            .unwrap()
            .into_text()
            .unwrap();
        let unix_val: Value = serde_json::from_str(&unix_msg).unwrap();
        assert_eq!(unix_val["result"]["data"][0]["id"], "thread-unix-1");

        let _ = std::fs::remove_file(&sock_path_clone);
    }
}

