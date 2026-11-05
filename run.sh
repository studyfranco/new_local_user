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

# Create inter-agent CLI helper (/tmp/bin/agent-ask) so any CLI agent can query peers via Agent Hub
cat << 'EOF' > /tmp/bin/agent-ask
#!/bin/bash
set -euo pipefail
if [[ $# -lt 2 ]]; then
    echo "Usage: agent-ask <codex|goose|claude|antigravity> \"<prompt>\"" >&2
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

# Auto-start lightweight JSON-RPC bridge for Claude (3285) or Antigravity (3286) if applicable
BRIDGE_PORT="${AGENT_BRIDGE_PORT:-}"
BRIDGE_AGENT="${AGENT_BRIDGE_NAME:-}"
if [[ -z "$BRIDGE_PORT" ]]; then
    if command -v claude >/dev/null 2>&1; then
        BRIDGE_PORT="3285"
        BRIDGE_AGENT="claude"
    elif command -v agy >/dev/null 2>&1; then
        BRIDGE_PORT="3286"
        BRIDGE_AGENT="antigravity"
    fi
fi

if [[ -n "$BRIDGE_PORT" ]] && command -v python3 >/dev/null 2>&1; then
    cat << 'PYBRIDGE' > /tmp/runtime-thenewuser/agent_bridge.py
import json, os, subprocess, time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

AGENT_NAME = os.environ.get("BRIDGE_AGENT", "cli")
PORT = int(os.environ.get("BRIDGE_PORT", "3285"))
SECRET = os.environ.get("AGENT_BRIDGE_SECRET", "")

def capture_tmux(lines=120):
    res = subprocess.run(
        ["tmux", "capture-pane", "-pt", "the_new_user", "-S", f"-{lines}"],
        capture_output=True, text=True
    )
    return res.stdout.strip() if res.returncode == 0 else res.stderr.strip()

def send_tmux(text):
    subprocess.run(["tmux", "send-keys", "-t", "the_new_user", "-l", text], check=False)
    subprocess.run(["tmux", "send-keys", "-t", "the_new_user", "Enter"], check=False)
    time.sleep(1.2)
    return capture_tmux(120)

def run_cli_prompt(prompt, cwd="/srv", mode="auto"):
    if mode == "tmux":
        return send_tmux(prompt)
    cmd = None
    if AGENT_NAME == "claude":
        cmd = ["claude", "-p", prompt, "--dangerously-skip-permissions"]
    elif AGENT_NAME == "antigravity":
        cmd = ["agy", "-p", prompt, "--dangerously-skip-permissions"]
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
        if self.path.startswith("/healthz"):
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
        elif method == "tmux/capture":
            result = {"output": capture_tmux(int(params.get("lines", 120)))}
        elif method == "tmux/send_keys":
            result = {"output": send_tmux(str(params.get("text", "")))}
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
