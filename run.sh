#!/bin/bash

set -euo pipefail

# Prepare ephemeral runtime and cache directories in /tmp (compatible with read_only: true)
mkdir -p "${XDG_RUNTIME_DIR:-/tmp/runtime-thenewuser}" \
         "${XDG_CACHE_HOME:-/tmp/.cache}" \
         "${CARGO_HOME:-/tmp/cargo}" \
         "${PYTHONPYCACHEPREFIX:-/tmp/pycache}" \
         "${PIP_CACHE_DIR:-/tmp/.cache/pip}" \
         "${NPM_CONFIG_CACHE:-/tmp/.cache/npm}" \
         "/tmp/bin" 2>/dev/null || true
chmod 700 "${XDG_RUNTIME_DIR:-/tmp/runtime-thenewuser}" 2>/dev/null || true
export PATH="/tmp/bin:${PATH}"

# Prepare XDG user directories if HOME is mounted writable
if [[ -w "${HOME:-/home/thenewuser}" ]]; then
    mkdir -p "${XDG_CONFIG_HOME:-/home/thenewuser/.config}" \
             "${XDG_DATA_HOME:-/home/thenewuser/.local/share}" \
             "${XDG_STATE_HOME:-/home/thenewuser/.local/state}" \
             "${HOME:-/home/thenewuser}/.local/bin" 2>/dev/null || true
fi

# 1. Create inter-agent CLI helper (/tmp/bin/agent-ask)
cat << 'EOF' > /tmp/bin/agent-ask
#!/bin/bash
set -euo pipefail
if [[ $# -lt 2 ]]; then
    echo "Usage: agent-ask <instance_name_or_kind> \"<prompt>\"" >&2
    exit 1
fi
TARGET_AGENT="$1"
shift
PROMPT_TEXT="$*"
HUB_URL="${AGENT_HUB_URL:-http://web:8080}"
python3 - "$HUB_URL" "$TARGET_AGENT" "$PROMPT_TEXT" << 'PYEOF'
import json, sys, urllib.request
hub_url, agent, prompt = sys.argv[1], sys.argv[2], sys.argv[3]
req = urllib.request.Request(
    f"{hub_url.rstrip('/')}/api/agents/ask",
    data=json.dumps({"agent": agent, "prompt": prompt, "cwd": "/srv"}).encode("utf-8"),
    headers={"Content-Type": "application/json"},
    method="POST",
)
with urllib.request.urlopen(req, timeout=300) as resp:
    data = json.loads(resp.read().decode("utf-8"))
    if data.get("error"):
        print(f"Error: {data['error']}", file=sys.stderr)
        sys.exit(1)
    print(data.get("reply", ""))
PYEOF
EOF
chmod +x /tmp/bin/agent-ask 2>/dev/null || true

# 2. Create orchestration channel CLI helper (/tmp/bin/agent-channel)
cat << 'EOF' > /tmp/bin/agent-channel
#!/bin/bash
set -euo pipefail
if [[ $# -lt 2 ]]; then
    echo "Usage: agent-channel <channel_id> \"<message>\" [recipient]" >&2
    exit 1
fi
CHANNEL_ID="$1"
MESSAGE="$2"
RECIPIENT="${3:-@all}"
SENDER="${AGENT_INSTANCE_NAME:-$(hostname)}"
HUB_URL="${AGENT_HUB_URL:-http://web:8080}"
python3 - "$HUB_URL" "$CHANNEL_ID" "$SENDER" "$RECIPIENT" "$MESSAGE" << 'PYEOF'
import json, sys, urllib.request
hub_url, ch_id, sender, recipient, content = sys.argv[1:6]
req = urllib.request.Request(
    f"{hub_url.rstrip('/')}/api/orchestration/channels/{ch_id}/publish",
    data=json.dumps({
        "sender_name": sender,
        "recipient_name": recipient,
        "content": content,
        "auto_dispatch": True
    }).encode("utf-8"),
    headers={"Content-Type": "application/json"},
    method="POST",
)
with urllib.request.urlopen(req, timeout=300) as resp:
    print(resp.read().decode("utf-8"))
PYEOF
EOF
chmod +x /tmp/bin/agent-channel 2>/dev/null || true

# 3. Auto-start lightweight JSON-RPC & Web-Auth bridge for any CLI agent
BRIDGE_PORT="${AGENT_BRIDGE_PORT:-}"
BRIDGE_AGENT="${AGENT_BRIDGE_NAME:-}"
if [[ -z "$BRIDGE_PORT" ]]; then
    if command -v claude >/dev/null 2>&1; then
        BRIDGE_PORT="3285"
        BRIDGE_AGENT="claude"
    elif command -v agy >/dev/null 2>&1; then
        BRIDGE_PORT="3286"
        BRIDGE_AGENT="antigravity"
    elif command -v codex >/dev/null 2>&1; then
        BRIDGE_PORT="3287"
        BRIDGE_AGENT="codex"
    elif command -v goose >/dev/null 2>&1; then
        BRIDGE_PORT="3288"
        BRIDGE_AGENT="goose"
    fi
fi

if [[ -n "$BRIDGE_PORT" ]] && command -v python3 >/dev/null 2>&1; then
    cat << 'PYBRIDGE' > /tmp/runtime-thenewuser/agent_bridge.py
import json, os, re, subprocess, time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

AGENT_NAME = os.environ.get("BRIDGE_AGENT", "cli")
PORT = int(os.environ.get("BRIDGE_PORT", "3285"))
SECRET = os.environ.get("AGENT_BRIDGE_SECRET", "")

def capture_tmux(session="the_new_user", lines=120):
    res = subprocess.run(
        ["tmux", "capture-pane", "-pt", session, "-S", f"-{lines}"],
        capture_output=True, text=True
    )
    return res.stdout.strip() if res.returncode == 0 else res.stderr.strip()

def send_tmux(text, session="the_new_user"):
    subprocess.run(["tmux", "send-keys", "-t", session, "-l", text], check=False)
    subprocess.run(["tmux", "send-keys", "-t", session, "Enter"], check=False)
    time.sleep(1.2)
    return capture_tmux(session, 120)

def check_auth_status():
    if AGENT_NAME == "codex":
        res = subprocess.run(["codex", "login", "status"], capture_output=True, text=True)
        out = (res.stdout + "\n" + res.stderr).strip()
        authenticated = "Logged in" in out and "Not logged in" not in out
        return {"authenticated": authenticated, "detail": out or "Unknown"}
    elif AGENT_NAME == "antigravity":
        pane = capture_tmux("the_new_user", 80)
        authenticated = "Authenticated as" in pane or "Antigravity server is running" in pane
        return {"authenticated": authenticated, "detail": pane[-400:] if pane else "Check tmux"}
    elif AGENT_NAME == "claude":
        pane = capture_tmux("the_new_user", 80)
        authenticated = bool(os.environ.get("ANTHROPIC_API_KEY")) or ("Welcome to Claude" in pane)
        return {"authenticated": authenticated, "detail": pane[-400:] if pane else "Ready"}
    else:
        return {"authenticated": True, "detail": f"{AGENT_NAME} ready"}

def start_device_auth():
    if AGENT_NAME == "codex":
        subprocess.run(["tmux", "kill-session", "-t", "web_auth"], capture_output=True)
        subprocess.run(
            ["tmux", "new-session", "-d", "-s", "web_auth", "codex login --device-auth; sleep 300"],
            check=False
        )
        time.sleep(2.5)
        out = capture_tmux("web_auth", 60)
        url_match = re.search(r"(https://[^\s]+)", out)
        code_match = re.search(r"\b([A-Z0-9]{4,5}-[A-Z0-9]{4,6})\b", out)
        return {
            "status": "pending",
            "url": url_match.group(1) if url_match else "https://auth.openai.com/codex/device",
            "code": code_match.group(1) if code_match else "",
            "raw": out
        }
    else:
        out = capture_tmux("the_new_user", 80)
        url_match = re.search(r"(https://[^\s]+)", out)
        return {
            "status": "pending",
            "url": url_match.group(1) if url_match else "",
            "code": "",
            "raw": out
        }

def apply_api_key(api_key):
    if AGENT_NAME == "codex":
        p = subprocess.run(
            ["codex", "login", "--with-api-key"],
            input=api_key, text=True, capture_output=True
        )
        return {"authenticated": p.returncode == 0, "detail": (p.stdout + p.stderr).strip()}
    elif AGENT_NAME == "antigravity":
        cfg_dir = os.path.expanduser("~/.gemini/antigravity-cli")
        os.makedirs(cfg_dir, exist_ok=True)
        with open(os.path.join(cfg_dir, "settings.json"), "w") as f:
            json.dump({"modelProvider": "gemini", "apiKey": api_key}, f)
        os.environ["GEMINI_API_KEY"] = api_key
        return {"authenticated": True, "detail": "GEMINI_API_KEY configured"}
    elif AGENT_NAME == "claude":
        os.environ["ANTHROPIC_API_KEY"] = api_key
        return {"authenticated": True, "detail": "ANTHROPIC_API_KEY configured"}
    return {"authenticated": True, "detail": "Key stored"}

def run_cli_prompt(prompt, cwd="/srv", mode="auto"):
    if mode == "tmux":
        return send_tmux(prompt)
    cmd = None
    if AGENT_NAME == "claude":
        cmd = ["claude", "-p", prompt, "--dangerously-skip-permissions"]
    elif AGENT_NAME == "antigravity":
        cmd = ["agy", "-p", prompt, "--dangerously-skip-permissions"]
    elif AGENT_NAME == "codex":
        cmd = ["codex", "exec", "--skip-git-repo-check", "--dangerously-bypass-approvals-and-sandbox", prompt]
    elif AGENT_NAME == "goose":
        cmd = ["goose", "run", "-t", prompt]
    if cmd:
        try:
            res = subprocess.run(
                cmd, cwd=cwd if os.path.isdir(cwd) else "/tmp",
                capture_output=True, text=True, timeout=240
            )
            out = (res.stdout or "").strip()
            if res.returncode == 0 and out:
                return out
        except Exception:
            pass
    return send_tmux(prompt)

class Handler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        return

    def _check_auth(self):
        if not SECRET:
            return True
        return self.headers.get("X-Secret-Key", "") == SECRET

    def do_GET(self):
        if self.path.startswith("/healthz") or self.path.startswith("/ready"):
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"status": "ok", "agent": AGENT_NAME}).encode())
            return
        self.send_response(404)
        self.end_headers()

    def do_POST(self):
        if not self._check_auth():
            self.send_response(401)
            self.end_headers()
            return
        length = int(self.headers.get("Content-Length", "0"))
        raw = self.rfile.read(length).decode("utf-8") if length > 0 else "{}"
        try:
            req = json.loads(raw)
        except Exception:
            req = {}
        rpc_id = req.get("id", 1)
        method = req.get("method", "")
        params = req.get("params") or {}

        if method == "initialize":
            result = {"agent": AGENT_NAME, "protocolVersion": 1, "tmuxSession": "the_new_user"}
        elif method == "auth/status":
            result = check_auth_status()
        elif method == "auth/device_start":
            result = start_device_auth()
        elif method == "auth/api_key":
            result = apply_api_key(str(params.get("apiKey", "")))
        elif method == "auth/oauth_callback":
            cb_url = str(params.get("callbackUrl", "")).strip()
            if cb_url.startswith("http://localhost") or cb_url.startswith("http://127.0.0.1"):
                r = subprocess.run(["curl", "-sL", cb_url], capture_output=True, text=True, timeout=15)
                result = {"authenticated": True, "detail": r.stdout[:300]}
            else:
                result = {"authenticated": True, "detail": send_tmux(cb_url)}
        elif method == "tmux/capture":
            result = {"output": capture_tmux(params.get("session", "the_new_user"), int(params.get("lines", 120)))}
        elif method == "tmux/send_keys":
            result = {"output": send_tmux(str(params.get("text", "")), params.get("session", "the_new_user"))}
        elif method in ("prompt/send", "session/prompt", "turn/start"):
            text = params.get("prompt") or params.get("text") or ""
            if isinstance(text, list) and len(text) > 0 and isinstance(text[0], dict):
                text = text[0].get("text", "")
            cwd = params.get("cwd", "/srv")
            mode = params.get("mode", "auto")
            reply_text = run_cli_prompt(str(text), cwd=cwd, mode=mode)
            result = {"reply": reply_text, "agent": AGENT_NAME}
        else:
            result = {"status": "ok", "agent": AGENT_NAME}

        body = json.dumps({"jsonrpc": "2.0", "id": rpc_id, "result": result}).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

if __name__ == "__main__":
    ThreadingHTTPServer(("0.0.0.0", PORT), Handler).serve_forever()
PYBRIDGE
    BRIDGE_AGENT="$BRIDGE_AGENT" BRIDGE_PORT="$BRIDGE_PORT" python3 /tmp/runtime-thenewuser/agent_bridge.py >/tmp/runtime-thenewuser/bridge.log 2>&1 &
fi

cd /tmp
if [[ -x "/usr/local/bin/start-cli.sh" ]]; then
    tmux new-session -d -s the_new_user /usr/local/bin/start-cli.sh
else
    tmux new-session -d -s the_new_user /bin/bash
fi
tail -f /dev/null

exit 0
