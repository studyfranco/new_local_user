#[cfg(all(target_os = "linux", not(target_env = "msvc")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Method, Response, StatusCode},
    response::IntoResponse,
    routing::{any, get, post},
};
use bytes::Bytes;
use regex::Regex;
use serde_json::{Value, json};
use std::{env, net::SocketAddr, path::Path as FsPath, process::Stdio, sync::Arc, time::Duration};
use tokio::{io::AsyncWriteExt, process::Command, time::sleep};
use tracing::{error, info};

#[derive(Clone)]
struct BridgeState {
    agent_name: String,
    secret: Option<String>,
    cli_web_url: String,
    tmux_session: String,
    http_client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "agent_bridge=info".into()),
        )
        .init();

    let port: u16 = env::var("BRIDGE_PORT")
        .or_else(|_| env::var("AGENT_BRIDGE_PORT"))
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3285);

    let agent_name = env::var("BRIDGE_AGENT")
        .or_else(|_| env::var("AGENT_BRIDGE_NAME"))
        .unwrap_or_else(|_| "cli".to_string());

    let secret = env::var("AGENT_BRIDGE_SECRET")
        .ok()
        .filter(|s| !s.trim().is_empty());

    let default_web_port = match agent_name.as_str() {
        "antigravity" => 4400,
        "goose" => 3284,
        "codex" => 4500,
        _ => 4400,
    };
    let cli_web_url = env::var("CLI_WEB_URL")
        .unwrap_or_else(|_| format!("http://127.0.0.1:{}", default_web_port));

    let http_client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to create HTTP client: {}", e);
            return;
        }
    };

    let tmux_session = env::var("TMUX_SESSION")
        .or_else(|_| env::var("AGENT_TMUX_SESSION"))
        .unwrap_or_else(|_| "the_new_user".to_string());

    let state = Arc::new(BridgeState {
        agent_name: agent_name.clone(),
        secret,
        cli_web_url,
        tmux_session,
        http_client,
    });

    let app = Router::new()
        .route("/healthz", get(handle_health))
        .route("/ready", get(handle_health))
        .route("/rpc", post(handle_rpc))
        .route("/web", any(proxy_cli_web_root))
        .route("/web/{*path}", any(proxy_cli_web_path))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(
        "Rust agent-bridge ({}) listening on http://{}",
        agent_name, addr
    );

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind agent-bridge on {}: {}", addr, e);
            return;
        }
    };
    if let Err(e) = axum::serve(listener, app).await {
        error!("agent-bridge server error: {}", e);
    }
}

async fn handle_health(State(state): State<Arc<BridgeState>>) -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "agent": state.agent_name,
        "cli_web_url": state.cli_web_url
    }))
}

async fn capture_tmux(session: &str, lines: u32) -> String {
    capture_tmux_ext(session, lines, false).await
}

async fn capture_tmux_ext(session: &str, lines: u32, ansi: bool) -> String {
    let mut args = vec!["capture-pane"];
    if ansi {
        args.push("-e");
    }
    let start_line = format!("-{}", lines);
    args.extend(["-pt", session, "-S", &start_line]);
    let out = Command::new("tmux").args(&args).output().await;
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim_end().to_string(),
        Ok(o) => String::from_utf8_lossy(&o.stderr).trim().to_string(),
        Err(e) => format!("tmux error: {}", e),
    }
}

async fn send_tmux(session: &str, text: &str) -> String {
    let _ = Command::new("tmux")
        .args(["send-keys", "-t", session, "-l", text])
        .status()
        .await;
    let _ = Command::new("tmux")
        .args(["send-keys", "-t", session, "Enter"])
        .status()
        .await;
    sleep(Duration::from_millis(1200)).await;
    capture_tmux(session, 120).await
}

async fn send_tmux_raw(session: &str, key_or_text: &str, special_key: bool) -> String {
    if special_key {
        let _ = Command::new("tmux")
            .args(["send-keys", "-t", session, key_or_text])
            .status()
            .await;
    } else {
        let _ = Command::new("tmux")
            .args(["send-keys", "-t", session, "-l", key_or_text])
            .status()
            .await;
    }
    sleep(Duration::from_millis(80)).await;
    capture_tmux_ext(session, 120, true).await
}

fn extract_urls_and_code(text: &str) -> (String, String) {
    let url = Regex::new(r"(https?://[^\s]+)")
        .ok()
        .and_then(|re| {
            re.captures(text)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
        })
        .unwrap_or_default();
    let code = Regex::new(r"\b([A-Z0-9]{4,5}-[A-Z0-9]{4,6})\b")
        .ok()
        .and_then(|re| {
            re.captures(text)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
        })
        .unwrap_or_default();
    (url, code)
}

async fn handle_rpc(
    State(state): State<Arc<BridgeState>>,
    headers: HeaderMap,
    Json(req): Json<Value>,
) -> impl IntoResponse {
    if let Some(ref sec) = state.secret {
        let provided = headers
            .get("x-secret-key")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if provided != sec {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "Unauthorized bridge key" })),
            );
        }
    }

    let rpc_id = req.get("id").cloned().unwrap_or(json!(1));
    let method = req["method"].as_str().unwrap_or("");
    let params = &req["params"];

    let result = match method {
        "initialize" => json!({
            "agent": state.agent_name,
            "protocolVersion": 1,
            "tmuxSession": state.tmux_session,
            "cliWebUrl": state.cli_web_url,
        }),
        "auth/status" => {
            if state.agent_name == "codex" {
                let out = Command::new("codex")
                    .args(["login", "status"])
                    .output()
                    .await;
                let detail = match out {
                    Ok(o) => format!(
                        "{}{}",
                        String::from_utf8_lossy(&o.stdout),
                        String::from_utf8_lossy(&o.stderr)
                    )
                    .trim()
                    .to_string(),
                    Err(e) => e.to_string(),
                };
                let authenticated =
                    detail.contains("Logged in") && !detail.contains("Not logged in");
                json!({ "authenticated": authenticated, "detail": detail })
            } else {
                let pane = capture_tmux(&state.tmux_session, 80).await;
                let (detected_url, _) = extract_urls_and_code(&pane);
                let authenticated = pane.contains("Authenticated as")
                    || pane.contains("Antigravity server is running")
                    || pane.contains("Welcome to Claude");
                json!({
                    "authenticated": authenticated,
                    "web_url": detected_url,
                    "detail": pane
                })
            }
        }
        "auth/device_start" => {
            if state.agent_name == "codex" {
                let _ = Command::new("tmux")
                    .args(["kill-session", "-t", "web_auth"])
                    .status()
                    .await;
                let _ = Command::new("tmux")
                    .args([
                        "new-session",
                        "-d",
                        "-s",
                        "web_auth",
                        "codex login --device-auth; sleep 300",
                    ])
                    .status()
                    .await;
                sleep(Duration::from_millis(2500)).await;
                let raw = capture_tmux("web_auth", 60).await;
                let (url, code) = extract_urls_and_code(&raw);
                json!({
                    "status": "pending",
                    "url": if url.is_empty() { "https://auth.openai.com/codex/device".to_string() } else { url },
                    "code": code,
                    "raw": raw
                })
            } else {
                let raw = capture_tmux(&state.tmux_session, 80).await;
                let (url, code) = extract_urls_and_code(&raw);
                json!({
                    "status": "pending",
                    "url": url,
                    "code": code,
                    "raw": raw
                })
            }
        }
        "auth/api_key" => {
            let api_key = params["apiKey"].as_str().unwrap_or("");
            if state.agent_name == "codex" {
                let child = Command::new("codex")
                    .args(["login", "--with-api-key"])
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn();
                if let Ok(mut c) = child {
                    if let Some(mut stdin) = c.stdin.take() {
                        let _ = stdin.write_all(api_key.as_bytes()).await;
                    }
                    let out = c.wait_with_output().await;
                    let ok = out.as_ref().map(|o| o.status.success()).unwrap_or(false);
                    json!({ "authenticated": ok, "detail": "codex login --with-api-key executed" })
                } else {
                    json!({ "authenticated": false, "detail": "Failed to spawn codex" })
                }
            } else {
                let home = env::var("HOME").unwrap_or_else(|_| "/home/thenewuser".to_string());
                let cfg_dir = format!("{}/.gemini/antigravity-cli", home);
                let _ = std::fs::create_dir_all(&cfg_dir);
                let _ = std::fs::write(
                    format!("{}/settings.json", cfg_dir),
                    json!({ "modelProvider": "gemini", "apiKey": api_key }).to_string(),
                );
                json!({ "authenticated": true, "detail": "API key saved" })
            }
        }
        "auth/oauth_callback" => {
            let cb_url = params["callbackUrl"].as_str().unwrap_or("").trim();
            if cb_url.starts_with("http://localhost") || cb_url.starts_with("http://127.0.0.1") {
                match state.http_client.get(cb_url).send().await {
                    Ok(r) => {
                        let txt = r.text().await.unwrap_or_default();
                        json!({ "authenticated": true, "detail": txt })
                    }
                    Err(e) => json!({ "authenticated": false, "detail": e.to_string() }),
                }
            } else {
                let out = send_tmux(&state.tmux_session, cb_url).await;
                json!({ "authenticated": true, "detail": out })
            }
        }
        "tmux/capture" => {
            let lines = params["lines"].as_u64().unwrap_or(120) as u32;
            let session = params["session"].as_str().unwrap_or(&state.tmux_session);
            let ansi = params["ansi"].as_bool().unwrap_or(false);
            let out = capture_tmux_ext(session, lines, ansi).await;
            json!({ "output": out })
        }
        "tmux/send_keys" => {
            let text = params["text"].as_str().unwrap_or("");
            let session = params["session"].as_str().unwrap_or(&state.tmux_session);
            let out = send_tmux(session, text).await;
            json!({ "output": out })
        }
        "tmux/send_raw" => {
            let data = params["data"].as_str().unwrap_or("");
            let special = params["special"].as_bool().unwrap_or(false);
            let session = params["session"].as_str().unwrap_or(&state.tmux_session);
            let out = send_tmux_raw(session, data, special).await;
            json!({ "output": out })
        }
        "prompt/send" | "session/prompt" | "turn/start" => {
            let prompt = params["prompt"].as_str().unwrap_or("");
            let cwd = params["cwd"].as_str().unwrap_or("/srv");
            let mode = params["mode"].as_str().unwrap_or("auto");
            let reply =
                run_cli_prompt(&state.agent_name, &state.tmux_session, prompt, cwd, mode).await;
            json!({ "reply": reply, "agent": state.agent_name })
        }
        _ => json!({ "status": "ok", "agent": state.agent_name }),
    };

    (
        StatusCode::OK,
        Json(json!({ "jsonrpc": "2.0", "id": rpc_id, "result": result })),
    )
}

async fn run_cli_prompt(
    agent_name: &str,
    tmux_session: &str,
    prompt: &str,
    cwd: &str,
    mode: &str,
) -> String {
    if mode == "tmux" {
        return send_tmux(tmux_session, prompt).await;
    }
    let workdir = if FsPath::new(cwd).is_dir() {
        cwd
    } else {
        "/tmp"
    };
    let out = match agent_name {
        "claude" => {
            Command::new("claude")
                .args(["-p", prompt, "--dangerously-skip-permissions"])
                .current_dir(workdir)
                .output()
                .await
        }
        "antigravity" => {
            Command::new("agy")
                .args(["-p", prompt, "--dangerously-skip-permissions"])
                .current_dir(workdir)
                .output()
                .await
        }
        "codex" => {
            Command::new("codex")
                .args([
                    "exec",
                    "--skip-git-repo-check",
                    "--dangerously-bypass-approvals-and-sandbox",
                    prompt,
                ])
                .current_dir(workdir)
                .output()
                .await
        }
        "goose" => {
            Command::new("goose")
                .args(["run", "-t", prompt])
                .current_dir(workdir)
                .output()
                .await
        }
        _ => return send_tmux(tmux_session, prompt).await,
    };

    if let Ok(o) = out {
        let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if o.status.success() && !s.is_empty() {
            return s;
        }
    }
    send_tmux(tmux_session, prompt).await
}

async fn proxy_cli_web_root(
    State(state): State<Arc<BridgeState>>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    proxy_cli_web_inner(&state, "", method, headers, body).await
}

async fn proxy_cli_web_path(
    State(state): State<Arc<BridgeState>>,
    Path(path): Path<String>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    proxy_cli_web_inner(&state, &path, method, headers, body).await
}

async fn proxy_cli_web_inner(
    state: &BridgeState,
    path: &str,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse + use<> {
    let url = if path.is_empty() {
        state.cli_web_url.clone()
    } else {
        format!(
            "{}/{}",
            state.cli_web_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    };
    let mut req = state.http_client.request(method, &url).body(body.to_vec());
    if let Some(ct) = headers.get("content-type").and_then(|v| v.to_str().ok()) {
        req = req.header("Content-Type", ct);
    }
    match req.send().await {
        Ok(resp) => {
            let status = StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::OK);
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
            Json(json!({ "error": format!("CLI web server unreachable ({}): {}", url, e) })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_urls_and_code() {
        let sample =
            "Open https://auth.openai.com/codex/device and enter code AB12-9876F to login.";
        let (url, code) = extract_urls_and_code(sample);
        assert_eq!(url, "https://auth.openai.com/codex/device");
        assert_eq!(code, "AB12-9876F");

        let empty = "No url or device code here";
        let (url2, code2) = extract_urls_and_code(empty);
        assert!(url2.is_empty());
        assert!(code2.is_empty());
    }
}
