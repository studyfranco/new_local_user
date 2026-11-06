use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{process::Command, time::sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message as TungMessage};

struct ChildGuard(tokio::process::Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.start_kill();
    }
}

async fn cdp_call(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    id: u64,
    method: &str,
    params: Value,
) -> Value {
    let payload = json!({
        "id": id,
        "method": method,
        "params": params
    });
    ws.send(TungMessage::Text(payload.to_string().into()))
        .await
        .unwrap_or_else(|e| panic!("CDP send failed for {method}: {e}"));

    while let Some(Ok(msg)) = ws.next().await {
        if let TungMessage::Text(txt) = msg
            && let Ok(v) = serde_json::from_str::<Value>(&txt)
            && v.get("id") == Some(&json!(id))
        {
            return v;
        }
    }
    panic!("CDP stream closed before response for id={id} method={method}");
}

async fn cdp_eval(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    id: u64,
    expression: &str,
) -> Value {
    let resp = cdp_call(
        ws,
        id,
        "Runtime.evaluate",
        json!({
            "expression": expression,
            "returnByValue": true,
            "awaitPromise": true
        }),
    )
    .await;
    resp["result"]["result"]["value"].clone()
}

async fn save_screenshot(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    id: u64,
    out_path: &Path,
) {
    let resp = cdp_call(ws, id, "Page.captureScreenshot", json!({ "format": "png" })).await;
    if let Some(b64) = resp["result"]["data"].as_str()
        && let Ok(bytes) = BASE64.decode(b64)
    {
        let _ = std::fs::write(out_path, bytes);
    }
}

#[tokio::test]
async fn chromium_tmux_cli_agents_e2e_verification() {
    if !Path::new("/usr/bin/chromium").exists() || !Path::new("/usr/bin/tmux").exists() {
        return;
    }

    // 1. Prepare real local tmux session for Antigravity ("agy_session")
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", "agy_session"])
        .status()
        .await;
    let _ = Command::new("tmux")
        .args(["new-session", "-d", "-s", "agy_session"])
        .status()
        .await;
    let _ = Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            "agy_session",
            "echo '=== ANTIGRAVITY AGY CLI TMUX SESSION ==='; /home/fallrik/.local/bin/agy --version",
            "Enter",
        ])
        .status()
        .await;

    // 2. Prepare real local tmux session for Codex ("codex_session")
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", "codex_session"])
        .status()
        .await;
    let _ = Command::new("tmux")
        .args(["new-session", "-d", "-s", "codex_session"])
        .status()
        .await;
    let _ = Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            "codex_session",
            "echo '=== OPENAI CODEX CLI TMUX SESSION ==='; echo 'codex ready v0.1.0'; PS1='codex> '",
            "Enter",
        ])
        .status()
        .await;

    sleep(Duration::from_millis(500)).await;

    let agent_hub_bin = PathBuf::from(env!("CARGO_BIN_EXE_agent-hub"));
    let agent_bridge_bin = PathBuf::from(env!("CARGO_BIN_EXE_agent-bridge"));

    // 3. Spawn local agent-bridge for Antigravity (port 13296, session agy_session)
    let bridge_agy = Command::new(&agent_bridge_bin)
        .env("BRIDGE_PORT", "13296")
        .env("BRIDGE_AGENT", "antigravity")
        .env("TMUX_SESSION", "agy_session")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn agy agent-bridge: {e}"));
    let _guard_bridge_agy = ChildGuard(bridge_agy);

    // 4. Spawn local agent-bridge for Codex (port 13297, session codex_session)
    let bridge_codex = Command::new(&agent_bridge_bin)
        .env("BRIDGE_PORT", "13297")
        .env("BRIDGE_AGENT", "codex")
        .env("TMUX_SESSION", "codex_session")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn codex agent-bridge: {e}"));
    let _guard_bridge_codex = ChildGuard(bridge_codex);

    // 5. Spawn agent-hub on port 18092 with a temporary SQLite DB
    let tmp_dir = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir failed: {e}"));
    let db_path = tmp_dir.path().join("chromium_tmux_e2e.db");
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")));

    let hub_child = Command::new(&agent_hub_bin)
        .env("PORT", "18092")
        .env(
            "DATABASE_URL",
            format!("sqlite://{}?mode=rwc", db_path.display()),
        )
        .env("ANTIGRAVITY_RPC_URL", "http://127.0.0.1:13296/rpc")
        .env("CODEX_RPC_URL", "http://127.0.0.1:13297/rpc")
        .env("CLAUDE_RPC_URL", "http://127.0.0.1:13296/rpc")
        .env("CODEX_WS_URL", "ws://127.0.0.1:14500")
        .env("GOOSE_WS_URL", "ws://127.0.0.1:13284/acp")
        .env("GOOSE_HTTP_URL", "http://127.0.0.1:13284")
        .env("DEFAULT_CWD", repo_root.to_string_lossy().to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn agent-hub: {e}"));
    let _guard_hub = ChildGuard(hub_child);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_else(|e| panic!("reqwest client failed: {e}"));

    // Wait for agent-hub and bridges to be ready
    for _ in 0..60 {
        let hub_ok = client
            .get("http://127.0.0.1:18092/healthz")
            .send()
            .await
            .is_ok_and(|r| r.status().is_success());
        let agy_ok = client
            .get("http://127.0.0.1:13296/healthz")
            .send()
            .await
            .is_ok_and(|r| r.status().is_success());
        let codex_ok = client
            .get("http://127.0.0.1:13297/healthz")
            .send()
            .await
            .is_ok_and(|r| r.status().is_success());
        if hub_ok && agy_ok && codex_ok {
            break;
        }
        sleep(Duration::from_millis(150)).await;
    }

    // 6. Launch real headless Chromium on remote debugging port 19225
    let chrome_profile = tmp_dir.path().join("chrome_profile");
    let chromium_child = Command::new("/usr/bin/chromium")
        .args([
            "--headless=new",
            "--no-sandbox",
            "--disable-gpu",
            "--disable-dev-shm-usage",
            "--remote-debugging-port=19225",
            "--window-size=1440,960",
            &format!("--user-data-dir={}", chrome_profile.display()),
            "http://127.0.0.1:18092/",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn chromium: {e}"));
    let _guard_chromium = ChildGuard(chromium_child);

    // Discover CDP page WebSocket URL
    let mut ws_debugger_url = String::new();
    for _ in 0..50 {
        if let Ok(resp) = client.get("http://127.0.0.1:19225/json/list").send().await
            && let Ok(targets) = resp.json::<Vec<Value>>().await
            && let Some(page) = targets.iter().find(|t| {
                t["type"] == "page" && t["url"].as_str().unwrap_or("").contains("127.0.0.1:18092")
            })
            && let Some(u) = page["webSocketDebuggerUrl"].as_str()
        {
            ws_debugger_url = u.to_string();
            break;
        }
        sleep(Duration::from_millis(150)).await;
    }
    assert!(
        !ws_debugger_url.is_empty(),
        "Chromium CDP debugger endpoint must be discoverable"
    );

    let (mut ws_stream, _) = connect_async(&ws_debugger_url)
        .await
        .unwrap_or_else(|e| panic!("Failed to connect to Chromium CDP: {e}"));

    // Enable Page and Runtime domains
    let _ = cdp_call(&mut ws_stream, 1, "Page.enable", json!({})).await;
    let _ = cdp_call(&mut ws_stream, 2, "Runtime.enable", json!({})).await;

    // Wait until instances are loaded into frontend state
    let mut instances_count = 0;
    for i in 0..30 {
        let val = cdp_eval(
            &mut ws_stream,
            10 + i,
            "document.querySelectorAll('#inventory-tbody tr, #instances-tbody tr').length",
        )
        .await;
        instances_count = val.as_i64().unwrap_or(0);
        if instances_count >= 4 {
            break;
        }
        sleep(Duration::from_millis(250)).await;
    }
    assert_eq!(
        instances_count, 4,
        "Expected 4 default CLI instances rendered in table"
    );

    let artifact_dir = PathBuf::from(
        "/home/fallrik/.gemini/antigravity/brain/cf10db04-b547-431a-9701-b53154ddc4ff",
    );

    // =========================================================================
    // TEST 1: Connect to Antigravity CLI instance running in agy_session tmux
    // =========================================================================
    let _ = cdp_eval(&mut ws_stream, 99, "window.switchTab('console'); true").await;

    let agy_opened = cdp_eval(
        &mut ws_stream,
        100,
        r#"(() => {
            const agy = window.state.instances.find(i => i.kind === 'antigravity');
            if (agy) {
                window.openTerminalModal(agy.id);
                return { ok: true, id: agy.id, name: agy.name };
            }
            return { ok: false, err: "antigravity instance not found" };
        })()"#,
    )
    .await;
    assert_eq!(
        agy_opened["ok"],
        json!(true),
        "Should open terminal modal for Antigravity"
    );

    // Wait for WebSocket terminal to connect and display live tmux output
    let mut agy_term_text = String::new();
    let mut agy_ws_badge = String::new();
    for i in 0..40 {
        let t = cdp_eval(
            &mut ws_stream,
            110 + i * 2,
            "document.getElementById('terminal-viewport')?.innerText || ''",
        )
        .await;
        let b = cdp_eval(
            &mut ws_stream,
            111 + i * 2,
            "document.getElementById('terminal-ws-status')?.innerText || ''",
        )
        .await;
        agy_term_text = t.as_str().unwrap_or("").to_string();
        agy_ws_badge = b.as_str().unwrap_or("").to_string();
        if agy_term_text.contains("ANTIGRAVITY AGY CLI TMUX SESSION") {
            break;
        }
        sleep(Duration::from_millis(250)).await;
    }
    assert!(
        agy_ws_badge.contains("Connected") || agy_ws_badge.contains("Connecté"),
        "Antigravity terminal WebSocket badge must be connected, got: {agy_ws_badge}"
    );
    assert!(
        agy_term_text.contains("ANTIGRAVITY AGY CLI TMUX SESSION"),
        "Antigravity terminal should display live agy_session content, got: {agy_term_text}"
    );

    // Send input into agy tmux session via terminal-cmd-input
    let _ = cdp_eval(
        &mut ws_stream,
        120,
        r#"(() => {
            const inp = document.getElementById("terminal-cmd-input");
            if (inp) {
                inp.value = "echo 'TEST_AGY_LIVE_OK'";
                window.sendTerminalLine();
                return true;
            }
            return false;
        })()"#,
    )
    .await;

    // Verify terminal viewport updates with live response
    for i in 0..30 {
        let t = cdp_eval(
            &mut ws_stream,
            130 + i,
            "document.getElementById('terminal-viewport')?.innerText || ''",
        )
        .await;
        agy_term_text = t.as_str().unwrap_or("").to_string();
        if agy_term_text.contains("TEST_AGY_LIVE_OK") {
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }
    assert!(
        agy_term_text.contains("TEST_AGY_LIVE_OK"),
        "Antigravity terminal must reflect sent input"
    );

    // Save screenshot of Antigravity terminal session taking 100% of workspace
    save_screenshot(
        &mut ws_stream,
        140,
        &artifact_dir.join("chromium_14_agy_tmux_connected.png"),
    )
    .await;

    // Test terminal maximize across the full viewport
    let _ = cdp_eval(
        &mut ws_stream,
        142,
        "document.getElementById('btn-term-maximize')?.click(); true",
    )
    .await;
    sleep(Duration::from_millis(300)).await;

    save_screenshot(
        &mut ws_stream,
        143,
        &artifact_dir.join("chromium_16_terminal_fullscreen_maximized.png"),
    )
    .await;

    let _ = cdp_eval(
        &mut ws_stream,
        144,
        "document.getElementById('btn-term-maximize')?.click(); true",
    )
    .await;
    sleep(Duration::from_millis(300)).await;

    // Close terminal modal
    let _ = cdp_eval(&mut ws_stream, 141, "window.closeModal('terminal'); true").await;
    sleep(Duration::from_millis(500)).await;

    // =========================================================================
    // TEST 2: Connect to Codex CLI instance running in codex_session tmux
    // =========================================================================
    let codex_opened = cdp_eval(
        &mut ws_stream,
        200,
        r#"(() => {
            const codex = window.state.instances.find(i => i.kind === 'codex');
            if (codex) {
                window.openTerminalModal(codex.id);
                return { ok: true, id: codex.id, name: codex.name };
            }
            return { ok: false, err: "codex instance not found" };
        })()"#,
    )
    .await;
    assert_eq!(
        codex_opened["ok"],
        json!(true),
        "Should open terminal modal for Codex"
    );

    // Wait for WebSocket terminal to connect and display live tmux output
    let mut codex_term_text = String::new();
    let mut codex_ws_badge = String::new();
    for i in 0..40 {
        let t = cdp_eval(
            &mut ws_stream,
            210 + i * 2,
            "document.getElementById('terminal-viewport')?.innerText || ''",
        )
        .await;
        let b = cdp_eval(
            &mut ws_stream,
            211 + i * 2,
            "document.getElementById('terminal-ws-status')?.innerText || ''",
        )
        .await;
        codex_term_text = t.as_str().unwrap_or("").to_string();
        codex_ws_badge = b.as_str().unwrap_or("").to_string();
        if codex_term_text.contains("OPENAI CODEX CLI TMUX SESSION")
            || codex_term_text.contains("codex ready")
        {
            break;
        }
        sleep(Duration::from_millis(250)).await;
    }
    assert!(
        codex_ws_badge.contains("Connected") || codex_ws_badge.contains("Connecté"),
        "Codex terminal WebSocket badge must be connected, got: {codex_ws_badge}"
    );
    assert!(
        codex_term_text.contains("OPENAI CODEX CLI TMUX SESSION")
            || codex_term_text.contains("codex ready"),
        "Codex terminal should display live codex_session content, got: {codex_term_text}"
    );

    // Send input into codex tmux session via terminal-cmd-input
    let _ = cdp_eval(
        &mut ws_stream,
        220,
        r#"(() => {
            const inp = document.getElementById("terminal-cmd-input");
            if (inp) {
                inp.value = "echo 'TEST_CODEX_LIVE_OK'";
                window.sendTerminalLine();
                return true;
            }
            return false;
        })()"#,
    )
    .await;

    // Verify terminal viewport updates with live response
    for i in 0..30 {
        let t = cdp_eval(
            &mut ws_stream,
            230 + i,
            "document.getElementById('terminal-viewport')?.innerText || ''",
        )
        .await;
        codex_term_text = t.as_str().unwrap_or("").to_string();
        if codex_term_text.contains("TEST_CODEX_LIVE_OK") {
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }
    assert!(
        codex_term_text.contains("TEST_CODEX_LIVE_OK"),
        "Codex terminal must reflect sent input"
    );

    // Save screenshot of Codex terminal session
    save_screenshot(
        &mut ws_stream,
        240,
        &artifact_dir.join("chromium_15_codex_tmux_connected.png"),
    )
    .await;

    // Close terminal modal
    let _ = cdp_eval(&mut ws_stream, 241, "window.closeModal('terminal'); true").await;
    sleep(Duration::from_millis(300)).await;

    // 13. Test Brand Home button and Single-Line Header alignment
    // First, switch away from console tab to orchestration tab
    let _ = cdp_eval(
        &mut ws_stream,
        250,
        "window.switchTab('orchestration'); true",
    )
    .await;
    sleep(Duration::from_millis(300)).await;

    let orch_active = cdp_eval(
        &mut ws_stream,
        251,
        "document.getElementById('tab-orchestration')?.classList.contains('active')",
    )
    .await;
    assert_eq!(
        orch_active,
        json!(true),
        "Orchestration panel should be active after switch"
    );

    // Click brand home button ('Simply Agent Hub')
    let _ = cdp_eval(
        &mut ws_stream,
        252,
        "document.getElementById('brand-home')?.click(); true",
    )
    .await;
    sleep(Duration::from_millis(300)).await;

    // Verify console panel is restored to active
    let console_active = cdp_eval(
        &mut ws_stream,
        253,
        "document.getElementById('tab-console')?.classList.contains('active')",
    )
    .await;
    assert_eq!(
        console_active,
        json!(true),
        "Clicking brand-home ('Simply Agent Hub') must return to console home tab"
    );

    // Verify that brand logo and tab navigation are rendered on the same row
    let logo_top = cdp_eval(
        &mut ws_stream,
        254,
        "Math.round(document.getElementById('brand-home')?.getBoundingClientRect().top || 0)",
    )
    .await;
    let tab_top = cdp_eval(
        &mut ws_stream,
        255,
        "Math.round(document.querySelector('.tab-nav')?.getBoundingClientRect().top || 0)",
    )
    .await;
    let logo_top_val = logo_top.as_i64().unwrap_or(0);
    let tab_top_val = tab_top.as_i64().unwrap_or(0);
    assert!(
        (logo_top_val - tab_top_val).abs() <= 10,
        "Brand home and tab navigation must be horizontally aligned on the same single line: logo={logo_top_val}, tab={tab_top_val}"
    );

    // Save screenshot of the single-line header with brand home at the same level as tabs
    save_screenshot(
        &mut ws_stream,
        260,
        &artifact_dir.join("chromium_17_single_line_header_home.png"),
    )
    .await;

    // Clean up test sessions
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", "agy_session"])
        .status()
        .await;
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", "codex_session"])
        .status()
        .await;
}
