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
    let resp = cdp_call(
        ws,
        id,
        "Page.captureScreenshot",
        json!({ "format": "png" }),
    )
    .await;
    if let Some(b64) = resp["result"]["data"].as_str()
        && let Ok(bytes) = BASE64.decode(b64)
    {
        let _ = std::fs::write(out_path, bytes);
    }
}

#[tokio::test]
async fn full_scale_chromium_and_local_instances_e2e() {
    if !Path::new("/usr/bin/chromium").exists() {
        return;
    }

    // 1. Prepare a real local tmux session ("the_new_user") with ANSI colors
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", "the_new_user"])
        .status()
        .await;
    let _ = Command::new("tmux")
        .args(["new-session", "-d", "-s", "the_new_user"])
        .status()
        .await;
    let _ = Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            "the_new_user",
            "printf '\\033[1;32m[SIMPLY-LOCAL-INSTANCE]\\033[0m Ready in tmux session the_new_user\\n'",
            "Enter",
        ])
        .status()
        .await;

    let agent_hub_bin = PathBuf::from(env!("CARGO_BIN_EXE_agent-hub"));
    let agent_bridge_bin = PathBuf::from(env!("CARGO_BIN_EXE_agent-bridge"));

    // 2. Spawn local agent-bridge (Claude on 13285, Antigravity on 13286)
    let bridge_claude = Command::new(&agent_bridge_bin)
        .env("BRIDGE_PORT", "13285")
        .env("BRIDGE_AGENT", "claude")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn claude agent-bridge: {e}"));
    let _guard_bridge_claude = ChildGuard(bridge_claude);

    let bridge_agy = Command::new(&agent_bridge_bin)
        .env("BRIDGE_PORT", "13286")
        .env("BRIDGE_AGENT", "antigravity")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn agy agent-bridge: {e}"));
    let _guard_bridge_agy = ChildGuard(bridge_agy);

    // 3. Spawn agent-hub on 18089 with a temporary SQLite DB and CWD pointing to repo root
    let tmp_dir = tempfile::tempdir().unwrap_or_else(|e| panic!("tempdir failed: {e}"));
    let db_path = tmp_dir.path().join("chromium_e2e.db");
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")));

    let hub_child = Command::new(&agent_hub_bin)
        .env("PORT", "18089")
        .env(
            "DATABASE_URL",
            format!("sqlite://{}?mode=rwc", db_path.display()),
        )
        .env("CLAUDE_RPC_URL", "http://127.0.0.1:13285/rpc")
        .env("ANTIGRAVITY_RPC_URL", "http://127.0.0.1:13286/rpc")
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

    // Wait for agent-hub and agent-bridge to be ready
    for _ in 0..40 {
        if let Ok(r) = client.get("http://127.0.0.1:18089/healthz").send().await
            && r.status().is_success()
        {
            break;
        }
        sleep(Duration::from_millis(150)).await;
    }

    // 4. Launch real headless Chromium with remote debugging port 19223
    let chrome_profile = tmp_dir.path().join("chrome_profile");
    let chromium_child = Command::new("/usr/bin/chromium")
        .args([
            "--headless=new",
            "--no-sandbox",
            "--disable-gpu",
            "--disable-dev-shm-usage",
            "--remote-debugging-port=19223",
            "--window-size=1440,960",
            &format!("--user-data-dir={}", chrome_profile.display()),
            "http://127.0.0.1:18089/",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn chromium: {e}"));
    let _guard_chromium = ChildGuard(chromium_child);

    // Discover CDP page WebSocket URL
    let mut ws_debugger_url = String::new();
    for _ in 0..50 {
        if let Ok(resp) = client.get("http://127.0.0.1:19223/json/list").send().await
            && let Ok(targets) = resp.json::<Vec<Value>>().await
            && let Some(page) = targets.iter().find(|t| {
                t["type"] == "page"
                    && t["url"]
                        .as_str()
                        .unwrap_or("")
                        .contains("127.0.0.1:18089")
            })
            && let Some(u) = page["webSocketDebuggerUrl"].as_str()
        {
            ws_debugger_url = u.to_string();
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }
    assert!(
        !ws_debugger_url.is_empty(),
        "Chromium DevTools page target did not become ready"
    );

    let (mut cdp_ws, _) = connect_async(&ws_debugger_url)
        .await
        .unwrap_or_else(|e| panic!("failed to connect to Chromium CDP WebSocket: {e}"));

    let _ = cdp_call(&mut cdp_ws, 1, "Page.enable", json!({})).await;
    let _ = cdp_call(&mut cdp_ws, 2, "Runtime.enable", json!({})).await;

    // Switch to Container Fleet (Inventory) Tab to observe all registered instances
    let _ = cdp_eval(&mut cdp_ws, 3, "window.switchTab('inventory'); true").await;

    // Wait until instances are loaded into the frontend state and rendered in the DOM
    let mut instances_count = 0;
    for i in 0..30 {
        let val = cdp_eval(
            &mut cdp_ws,
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
        "Expected 4 default CLI instances rendered in Chromium table"
    );

    let artifact_dir = PathBuf::from(
        "/home/fallrik/.gemini/antigravity/brain/cf10db04-b547-431a-9701-b53154ddc4ff",
    );
    let screenshot_dir = if artifact_dir.is_dir() {
        artifact_dir
    } else {
        tmp_dir.path().to_path_buf()
    };

    // Screenshot 1: Container Fleet Dashboard with roles & status
    save_screenshot(
        &mut cdp_ws,
        100,
        &screenshot_dir.join("chromium_01_instances.png"),
    )
    .await;

    // Verify local Claude instance is online (via our real local agent-bridge on 13285)
    let claude_online = cdp_eval(
        &mut cdp_ws,
        101,
        "(() => { const c = window.state.instances.find(i => i.kind === 'claude'); return c ? c.online : false; })()",
    )
    .await;
    assert_eq!(
        claude_online,
        json!(true),
        "Local Claude instance should be detected as online via local agent-bridge"
    );

    // Switch back to Console Tab and open the Native Interactive Terminal on the local Claude instance
    let _ = cdp_eval(&mut cdp_ws, 102, "window.switchTab('console'); true").await;
    let _ = cdp_eval(
        &mut cdp_ws,
        103,
        "(() => { const c = window.state.instances.find(i => i.kind === 'claude'); if (c) window.openTerminalModal(c.id); return c ? c.id : null; })()",
    )
    .await;

    // Wait for WebSocket terminal to connect and render the local tmux ANSI screen
    let mut term_text = String::new();
    let mut ws_badge = String::new();
    for i in 0..30 {
        let t = cdp_eval(
            &mut cdp_ws,
            110 + i * 2,
            "document.getElementById('terminal-viewport')?.innerText || ''",
        )
        .await;
        let b = cdp_eval(
            &mut cdp_ws,
            111 + i * 2,
            "document.getElementById('terminal-ws-status')?.innerText || ''",
        )
        .await;
        term_text = t.as_str().unwrap_or("").to_string();
        ws_badge = b.as_str().unwrap_or("").to_string();
        if term_text.contains("[SIMPLY-LOCAL-INSTANCE]") {
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }
    assert!(
        ws_badge.contains("Connected") || ws_badge.contains("Connecté"),
        "Terminal WebSocket badge should be connected, got: {ws_badge}"
    );
    assert!(
        term_text.contains("[SIMPLY-LOCAL-INSTANCE]"),
        "Native terminal in Chromium should display live tmux output, got: {term_text}"
    );

    // Send an interactive command from Chromium into the local tmux session
    let _ = cdp_eval(
        &mut cdp_ws,
        180,
        "(() => { const inp = document.getElementById('terminal-cmd-input'); inp.value = 'echo CHROMIUM_TMUX_OK'; window.sendTerminalLine(); return true; })()",
    )
    .await;

    let mut updated_term = String::new();
    for i in 0..25 {
        let t = cdp_eval(
            &mut cdp_ws,
            181 + i,
            "document.getElementById('terminal-viewport')?.innerText || ''",
        )
        .await;
        updated_term = t.as_str().unwrap_or("").to_string();
        if updated_term.contains("CHROMIUM_TMUX_OK") {
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }
    assert!(
        updated_term.contains("CHROMIUM_TMUX_OK"),
        "Command typed in Chromium terminal should execute in local tmux and stream back, got: {updated_term}"
    );

    // Screenshot 2: Native Interactive Terminal Drawer with live ANSI output
    save_screenshot(
        &mut cdp_ws,
        220,
        &screenshot_dir.join("chromium_02_terminal_natif.png"),
    )
    .await;

    // Close terminal drawer & switch to Multi-CLI Orchestration view
    let _ = cdp_eval(
        &mut cdp_ws,
        221,
        "window.closeModal('terminal-modal'); window.switchTab('orchestration'); true",
    )
    .await;
    sleep(Duration::from_millis(600)).await;

    // Publish an orchestration message in Chromium
    let _ = cdp_eval(
        &mut cdp_ws,
        222,
        "(() => { document.getElementById('channel-message-input').value = '@Claude Code Local validation from Chromium headless E2E'; window.publishOrchestrationMessage(); return true; })()",
    )
    .await;

    let mut orch_html = String::new();
    for i in 0..20 {
        let h = cdp_eval(
            &mut cdp_ws,
            230 + i,
            "document.getElementById('channel-messages')?.innerText || ''",
        )
        .await;
        orch_html = h.as_str().unwrap_or("").to_string();
        if orch_html.contains("validation from Chromium headless E2E") {
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }
    assert!(
        orch_html.contains("validation from Chromium headless E2E"),
        "Published orchestration message should appear in Chromium timeline"
    );

    // Screenshot 3: Multi-Agent Orchestration view
    save_screenshot(
        &mut cdp_ws,
        260,
        &screenshot_dir.join("chromium_03_orchestration.png"),
    )
    .await;

    // 6. Switch to Git Diff & Commit view in Chromium
    let _ = cdp_eval(&mut cdp_ws, 261, "window.switchTab('git'); true").await;
    let mut git_branch = String::new();
    for i in 0..25 {
        let b = cdp_eval(
            &mut cdp_ws,
            270 + i,
            "document.getElementById('git-branch-badge')?.innerText || ''",
        )
        .await;
        git_branch = b.as_str().unwrap_or("").to_string();
        if !git_branch.is_empty() && git_branch != "-" {
            break;
        }
        sleep(Duration::from_millis(200)).await;
    }
    assert!(
        !git_branch.is_empty() && git_branch != "-",
        "Git view in Chromium should display active git branch, got: {git_branch}"
    );

    // Screenshot 4: Git Diff & Commit view
    save_screenshot(
        &mut cdp_ws,
        300,
        &screenshot_dir.join("chromium_04_git_viewer.png"),
    )
    .await;

    // 7. Dynamic Internationalization (i18n), Browser Detection & Extensible Language Packs
    let fleet_tab_en = cdp_eval(
        &mut cdp_ws,
        301,
        "document.querySelector('[data-tab=\"inventory\"]')?.innerText || ''",
    )
    .await;
    assert!(
        fleet_tab_en.as_str().unwrap_or("").contains("Fleet"),
        "Default language should be English, got: {fleet_tab_en}"
    );

    // 7a. Test Browser Language Detection logic
    let detected_de = cdp_eval(
        &mut cdp_ws,
        302,
        "(() => { const oldLangs = navigator.languages; try { Object.defineProperty(navigator, 'languages', { value: ['de-DE', 'de', 'en'], configurable: true }); localStorage.removeItem('agent_hub_lang'); const res = window.detectBrowserLanguage(); return res; } finally { Object.defineProperty(navigator, 'languages', { value: oldLangs, configurable: true }); } })()",
    )
    .await;
    assert_eq!(
        detected_de,
        json!("de"),
        "Browser detection should match German 'de' from navigator.languages"
    );

    // 7b. Switch to German (DE) by clicking the DE button in Chromium
    let _ = cdp_eval(
        &mut cdp_ws,
        303,
        "document.getElementById('lang-btn-de')?.click(); true",
    )
    .await;
    sleep(Duration::from_millis(500)).await;

    let fleet_tab_de = cdp_eval(
        &mut cdp_ws,
        304,
        "document.querySelector('[data-tab=\"inventory\"]')?.innerText || ''",
    )
    .await;
    assert!(
        fleet_tab_de
            .as_str()
            .unwrap_or("")
            .contains("Container-Flotte"),
        "UI should dynamically translate to German 'Container-Flotte', got: {fleet_tab_de}"
    );

    let git_btn_de = cdp_eval(
        &mut cdp_ws,
        305,
        "document.getElementById('btn-git-refresh')?.innerText || ''",
    )
    .await;
    assert!(
        git_btn_de.as_str().unwrap_or("").contains("Analysieren"),
        "Git button should translate to German '↻ Analysieren', got: {git_btn_de}"
    );

    // Screenshot 8: German UI in Chromium
    save_screenshot(
        &mut cdp_ws,
        306,
        &screenshot_dir.join("chromium_08_i18n_german.png"),
    )
    .await;

    // 7c. Test Dynamic Runtime Language Pack Installation (e.g. Spanish 'es')
    let _ = cdp_eval(
        &mut cdp_ws,
        307,
        "(() => { window.registerLanguagePack('es', { tabInventory: 'Flota de Contenedores (SeaORM)', btnRefreshAll: '↻ Actualizar Estado' }); return true; })()",
    )
    .await;
    sleep(Duration::from_millis(300)).await;

    let has_es_btn = cdp_eval(
        &mut cdp_ws,
        308,
        "Boolean(document.getElementById('lang-btn-es'))",
    )
    .await;
    assert_eq!(
        has_es_btn,
        json!(true),
        "Dynamic language pack registration should create a new switcher button #lang-btn-es"
    );

    // Switch to newly installed Spanish language pack
    let _ = cdp_eval(
        &mut cdp_ws,
        309,
        "document.getElementById('lang-btn-es')?.click(); true",
    )
    .await;
    sleep(Duration::from_millis(400)).await;

    let fleet_tab_es = cdp_eval(
        &mut cdp_ws,
        310,
        "document.querySelector('[data-tab=\"inventory\"]')?.innerText || ''",
    )
    .await;
    assert!(
        fleet_tab_es
            .as_str()
            .unwrap_or("")
            .contains("Flota de Contenedores"),
        "UI should translate via installed language pack, got: {fleet_tab_es}"
    );

    // Screenshot 9: Dynamically installed language pack in Chromium
    save_screenshot(
        &mut cdp_ws,
        311,
        &screenshot_dir.join("chromium_09_i18n_dynamic_plugin.png"),
    )
    .await;

    // 7d. Switch to French (FR)
    let _ = cdp_eval(
        &mut cdp_ws,
        312,
        "document.getElementById('lang-btn-fr')?.click(); true",
    )
    .await;
    sleep(Duration::from_millis(500)).await;

    let fleet_tab_fr = cdp_eval(
        &mut cdp_ws,
        313,
        "document.querySelector('[data-tab=\"inventory\"]')?.innerText || ''",
    )
    .await;
    assert!(
        fleet_tab_fr
            .as_str()
            .unwrap_or("")
            .contains("Parc de Conteneurs"),
        "UI should dynamically translate to French, got: {fleet_tab_fr}"
    );

    let git_btn_fr = cdp_eval(
        &mut cdp_ws,
        314,
        "document.getElementById('btn-git-refresh')?.innerText || ''",
    )
    .await;
    assert!(
        git_btn_fr.as_str().unwrap_or("").contains("Analyser"),
        "Git button should translate to French '↻ Analyser', got: {git_btn_fr}"
    );

    // Screenshot 5: French UI in Chromium
    save_screenshot(
        &mut cdp_ws,
        315,
        &screenshot_dir.join("chromium_05_i18n_french.png"),
    )
    .await;

    // 7e. Switch back to English (EN)
    let _ = cdp_eval(
        &mut cdp_ws,
        316,
        "document.getElementById('lang-btn-en')?.click(); true",
    )
    .await;
    sleep(Duration::from_millis(500)).await;

    let fleet_tab_back = cdp_eval(
        &mut cdp_ws,
        317,
        "document.querySelector('[data-tab=\"inventory\"]')?.innerText || ''",
    )
    .await;
    assert!(
        fleet_tab_back.as_str().unwrap_or("").contains("Fleet"),
        "UI should dynamically switch back to English, got: {fleet_tab_back}"
    );

    // 8. Test Agent Role Selection and Onboarding in Chromium
    let _ = cdp_eval(&mut cdp_ws, 400, "window.switchTab('console'); true").await;
    sleep(Duration::from_millis(300)).await;

    // Select the Claude instance in the UI dropdown
    let _ = cdp_eval(
        &mut cdp_ws,
        401,
        "(() => { const sel = document.getElementById('cli-selector'); const inst = window.state.instances.find(i => i.kind === 'claude'); if (inst) { sel.value = inst.id; sel.dispatchEvent(new Event('change')); return inst.name; } return ''; })()",
    )
    .await;
    sleep(Duration::from_millis(300)).await;

    // Change role in the dropdown to "architect"
    let _ = cdp_eval(
        &mut cdp_ws,
        402,
        "(() => { const rsel = document.getElementById('cli-role-selector'); rsel.value = 'architect'; rsel.dispatchEvent(new Event('change')); return rsel.value; })()",
    )
    .await;
    sleep(Duration::from_millis(300)).await;

    let prompt_val = cdp_eval(
        &mut cdp_ws,
        403,
        "document.getElementById('cli-role-prompt')?.value || ''",
    )
    .await;
    assert!(
        prompt_val.as_str().unwrap_or("").contains("Architect"),
        "Role prompt should auto-populate with the Architect preset instructions"
    );

    // Click "Initialize Agent" in Chromium
    let _ = cdp_eval(
        &mut cdp_ws,
        404,
        "document.getElementById('btn-onboard-cli')?.click(); true",
    )
    .await;
    sleep(Duration::from_millis(1000)).await;

    // Screenshot 6: Agent Role & Mission Onboarding in Chromium
    save_screenshot(
        &mut cdp_ws,
        405,
        &screenshot_dir.join("chromium_06_role_onboarding.png"),
    )
    .await;

    // 9. Test Configurable Retention in Chromium
    let _ = cdp_eval(
        &mut cdp_ws,
        500,
        "window.switchTab('orchestration'); true",
    )
    .await;
    sleep(Duration::from_millis(300)).await;

    let _ = cdp_eval(
        &mut cdp_ws,
        501,
        "(() => { document.getElementById('channel-retention-days').value = '14'; document.getElementById('channel-max-messages').value = '25'; document.getElementById('btn-save-retention')?.click(); return true; })()",
    )
    .await;
    sleep(Duration::from_millis(1000)).await;

    // Screenshot 7: Retention Policy updated in Chromium
    save_screenshot(
        &mut cdp_ws,
        502,
        &screenshot_dir.join("chromium_07_retention_modal.png"),
    )
    .await;
}
