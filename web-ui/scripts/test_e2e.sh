#!/usr/bin/env bash
#
# End-to-end curl API test suite for Simply Agent Hub (web branch).
#
# Boots a fresh instance backed by a throwaway SQLite database (SeaORM) alongside mock CLI
# containers, and exercises every HTTP endpoint with curl:
# - Static assets & Readiness (/ , /style.css, /app.js, /healthz, /ready)
# - Configuration & Live probes (/api/config, /api/status)
# - Multi-CLI instance lifecycle (/api/instances POST, GET, PUT rename, DELETE)
# - Web UI Container Authentication (/api/instances/{id}/auth: status, device_start, api_key)
# - Direct CLI Ask & Tmux capture (/api/agents/ask, /api/agents/tmux)
# - Multi-Agent Orchestration Channels (/api/orchestration/channels POST, GET, PUT, publish, messages, DELETE)

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/agent_hub_e2e.XXXXXX")"
DB_PATH="$WORK_DIR/e2e.db"
HUB_PORT="${HUB_PORT:-18080}"
MOCK_PORT="${MOCK_PORT:-18286}"
BASE_URL="http://127.0.0.1:${HUB_PORT}"

HUB_PID=""
MOCK_PID=""
PASS_COUNT=0
FAIL_COUNT=0

cleanup() {
    [[ -n "$HUB_PID" ]] && kill "$HUB_PID" 2>/dev/null || true
    [[ -n "$MOCK_PID" ]] && kill "$MOCK_PID" 2>/dev/null || true
    rm -rf "$WORK_DIR"
}
trap cleanup EXIT

assert_contains() {
    local label="$1"
    local haystack="$2"
    local needle="$3"
    if [[ "$haystack" == *"$needle"* ]]; then
        echo "  [PASS] $label"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        echo "  [FAIL] $label (expected '$needle', got: $haystack)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
}

echo "==> 1. Starting mock CLI container RPC server on 127.0.0.1:${MOCK_PORT}"
python3 - "$MOCK_PORT" << 'PYEOF' &
import json, sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

port = int(sys.argv[1])

class MockCliHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        return
    def do_GET(self):
        body = b"<html><body>Native CLI Web UI Ready (Port 4400)</body></html>"
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)
    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        req = json.loads(self.rfile.read(length).decode("utf-8")) if length > 0 else {}
        rpc_id = req.get("id", 1)
        method = req.get("method", "")
        params = req.get("params") or {}
        if method == "auth/status":
            res = {"authenticated": True, "detail": "Mock CLI authenticated"}
        elif method == "auth/device_start":
            res = {"status": "pending", "url": "https://auth.openai.com/codex/device", "code": "TEST-12345"}
        elif method == "auth/api_key":
            res = {"authenticated": True, "detail": "API key applied"}
        elif method == "tmux/capture":
            res = {"output": "thenewuser@container:/srv$ mock-tmux-ready"}
        else:
            prompt = params.get("prompt", "")
            res = {"reply": f"MockAgentReply({prompt[:40]})"}
        body = json.dumps({"jsonrpc": "2.0", "id": rpc_id, "result": res}).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

ThreadingHTTPServer(("127.0.0.1", port), MockCliHandler).serve_forever()
PYEOF
MOCK_PID=$!

echo "==> 2. Building and starting Simply Agent Hub on ${BASE_URL}"
cd "$PROJECT_ROOT"
cargo build --quiet || exit 1

PORT="$HUB_PORT" \
DATABASE_URL="sqlite://${DB_PATH}?mode=rwc" \
ANTIGRAVITY_RPC_URL="http://127.0.0.1:${MOCK_PORT}/rpc" \
CLAUDE_RPC_URL="http://127.0.0.1:${MOCK_PORT}/rpc" \
./target/debug/agent-hub >"$WORK_DIR/hub.log" 2>&1 &
HUB_PID=$!

for _ in $(seq 1 30); do
    if curl -fsS "${BASE_URL}/ready" >/dev/null 2>&1; then
        break
    fi
    sleep 0.2
done

echo "==> 3. Running curl API checks against ${BASE_URL}"

# 3.1 Static & Health
HTML_OUT="$(curl -fsS "${BASE_URL}/")"
assert_contains "GET / serves Simply Agent Hub HTML" "$HTML_OUT" "Simply Agent Hub"

CSS_OUT="$(curl -fsS "${BASE_URL}/style.css")"
assert_contains "GET /style.css serves KDE Breeze Dark theme" "$CSS_OUT" "--accent: #1cdc9a"

JS_OUT="$(curl -fsS "${BASE_URL}/app.js")"
assert_contains "GET /app.js serves frontend script" "$JS_OUT" "/api/orchestration/channels"

HEALTH_OUT="$(curl -fsS "${BASE_URL}/healthz")"
assert_contains "GET /healthz returns status ok" "$HEALTH_OUT" '"status":"ok"'

READY_OUT="$(curl -fsS "${BASE_URL}/ready")"
assert_contains "GET /ready confirms SeaORM DB migration & seed" "$READY_OUT" '"status":"ready"'

# 3.2 Config & Status
CFG_OUT="$(curl -fsS "${BASE_URL}/api/config")"
assert_contains "GET /api/config returns default_cwd" "$CFG_OUT" '"default_cwd":"/srv"'

STATUS_OUT="$(curl -fsS "${BASE_URL}/api/status")"
assert_contains "GET /api/status probes containers" "$STATUS_OUT" '"online":true'

# 3.3 CLI Instances CRUD & Renaming
LIST_OUT="$(curl -fsS "${BASE_URL}/api/instances")"
assert_contains "GET /api/instances lists seeded instances" "$LIST_OUT" "Codex #1 (Principal)"

CREATE_OUT="$(curl -fsS -X POST "${BASE_URL}/api/instances" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"Antigravity #2 - Audio\",\"kind\":\"antigravity\",\"transport\":\"rpc\",\"endpoint_url\":\"http://127.0.0.1:${MOCK_PORT}/rpc\",\"default_cwd\":\"/srv/audio\"}")"
assert_contains "POST /api/instances creates new CLI instance" "$CREATE_OUT" "Antigravity #2 - Audio"

NEW_ID="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["id"])' "$CREATE_OUT")"

RENAME_OUT="$(curl -fsS -X PUT "${BASE_URL}/api/instances/${NEW_ID}" \
    -H "Content-Type: application/json" \
    -d '{"name":"Antigravity #2 - Bibliothèque Musicale"}')"
assert_contains "PUT /api/instances/{id} renames CLI instance in DB" "$RENAME_OUT" "Antigravity #2 - Bibliothèque Musicale"

GET_ONE_OUT="$(curl -fsS "${BASE_URL}/api/instances/${NEW_ID}")"
assert_contains "GET /api/instances/{id} returns renamed instance" "$GET_ONE_OUT" "Antigravity #2 - Bibliothèque Musicale"

# 3.4 Web UI Container Authentication Endpoints
AUTH_STATUS_OUT="$(curl -fsS -X POST "${BASE_URL}/api/instances/${NEW_ID}/auth" \
    -H "Content-Type: application/json" \
    -d '{"action":"status"}')"
assert_contains "POST /api/instances/{id}/auth (status)" "$AUTH_STATUS_OUT" '"authenticated":true'

AUTH_DEVICE_OUT="$(curl -fsS -X POST "${BASE_URL}/api/instances/${NEW_ID}/auth" \
    -H "Content-Type: application/json" \
    -d '{"action":"device_start"}')"
assert_contains "POST /api/instances/{id}/auth (device_start returns code)" "$AUTH_DEVICE_OUT" "TEST-12345"

AUTH_KEY_OUT="$(curl -fsS -X POST "${BASE_URL}/api/instances/${NEW_ID}/auth" \
    -H "Content-Type: application/json" \
    -d '{"action":"api_key","value":"sk-test-key-999"}')"
assert_contains "POST /api/instances/{id}/auth (api_key)" "$AUTH_KEY_OUT" "API key applied"

# 3.5 Direct Ask & Tmux Capture by Instance ID
TMUX_OUT="$(curl -fsS -X POST "${BASE_URL}/api/agents/tmux" \
    -H "Content-Type: application/json" \
    -d "{\"instance_id\":\"${NEW_ID}\",\"lines\":50}")"
assert_contains "POST /api/agents/tmux captures container tmux" "$TMUX_OUT" "mock-tmux-ready"

ASK_OUT="$(curl -fsS -X POST "${BASE_URL}/api/agents/ask" \
    -H "Content-Type: application/json" \
    -d "{\"instance_id\":\"${NEW_ID}\",\"prompt\":\"Range les fichiers FLAC\"}")"
assert_contains "POST /api/agents/ask queries specific CLI instance" "$ASK_OUT" "MockAgentReply(Range les fichiers FLAC)"

WEB_PROXY_OUT="$(curl -fsS "${BASE_URL}/api/instances/${NEW_ID}/web")"
assert_contains "GET /api/instances/{id}/web proxies native CLI web server" "$WEB_PROXY_OUT" "Native CLI Web UI Ready"

# 3.5b Roles Presets, Role Update, Agent Onboarding & Git Status
ROLES_OUT="$(curl -fsS "${BASE_URL}/api/roles/presets")"
assert_contains "GET /api/roles/presets returns role catalog" "$ROLES_OUT" '"id":"architect"'

ROLE_UPD_OUT="$(curl -fsS -X PUT "${BASE_URL}/api/instances/${NEW_ID}" \
    -H "Content-Type: application/json" \
    -d '{"role":"architect","role_prompt":"Tu es l Architecte Audio."}')"
assert_contains "PUT /api/instances/{id} updates role and role_prompt" "$ROLE_UPD_OUT" '"role":"architect"'

ONBOARD_OUT="$(curl -fsS -X POST "${BASE_URL}/api/instances/${NEW_ID}/onboard" \
    -H "Content-Type: application/json" \
    -d '{"channel_name":"audio-pipeline"}')"
assert_contains "POST /api/instances/{id}/onboard sends role prompt to CLI" "$ONBOARD_OUT" "MockAgentReply"

GIT_STATUS_OUT="$(curl -fsS "${BASE_URL}/api/git/status?cwd=${PROJECT_ROOT}")"
assert_contains "GET /api/git/status inspects workspace git repository" "$GIT_STATUS_OUT" '"branch":'

# 3.6 Multi-Agent Orchestration Channels
CHAN_CREATE_OUT="$(curl -fsS -X POST "${BASE_URL}/api/orchestration/channels" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"audio-pipeline\",\"description\":\"Canal Audio\",\"participant_ids\":[\"${NEW_ID}\"]}")"
assert_contains "POST /api/orchestration/channels creates channel" "$CHAN_CREATE_OUT" "audio-pipeline"

CHAN_ID="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1])["id"])' "$CHAN_CREATE_OUT")"

PUB_OUT="$(curl -fsS -X POST "${BASE_URL}/api/orchestration/channels/${CHAN_ID}/publish" \
    -H "Content-Type: application/json" \
    -d '{"sender_name":"Utilisateur","recipient_name":"@all","content":"Vérifie les tags ID3","auto_dispatch":true}')"
assert_contains "POST /api/orchestration/channels/{id}/publish dispatches to channel CLIs" "$PUB_OUT" "MockAgentReply"

MSGS_OUT="$(curl -fsS "${BASE_URL}/api/orchestration/channels/${CHAN_ID}/messages")"
assert_contains "GET /api/orchestration/channels/{id}/messages returns persisted history" "$MSGS_OUT" "Antigravity #2 - Bibliothèque Musicale"

RETENTION_OUT="$(curl -fsS -X PUT "${BASE_URL}/api/orchestration/channels/${CHAN_ID}" \
    -H "Content-Type: application/json" \
    -d '{"retention_days":7,"max_messages":50}')"
assert_contains "PUT /api/orchestration/channels/{id} updates configurable retention" "$RETENTION_OUT" '"retention_days":7'

PRUNE_OUT="$(curl -fsS -X POST "${BASE_URL}/api/orchestration/channels/${CHAN_ID}/prune")"
assert_contains "POST /api/orchestration/channels/{id}/prune enforces retention policy" "$PRUNE_OUT" '"pruned_messages":'

EXPORT_OUT="$(curl -fsS "${BASE_URL}/api/orchestration/channels/${CHAN_ID}/export")"
assert_contains "GET /api/orchestration/channels/{id}/export exports channel as Markdown" "$EXPORT_OUT" "# Canal d'orchestration : #audio-pipeline"

DEL_CHAN_OUT="$(curl -fsS -X DELETE "${BASE_URL}/api/orchestration/channels/${CHAN_ID}")"
assert_contains "DELETE /api/orchestration/channels/{id} removes channel" "$DEL_CHAN_OUT" '"deleted":true'

DEL_INST_OUT="$(curl -fsS -X DELETE "${BASE_URL}/api/instances/${NEW_ID}")"
assert_contains "DELETE /api/instances/{id} removes CLI instance" "$DEL_INST_OUT" '"deleted":true'

echo "=================================================================="
echo " E2E Summary: ${PASS_COUNT} passed, ${FAIL_COUNT} failed"
echo "=================================================================="
[[ "$FAIL_COUNT" -eq 0 ]]
