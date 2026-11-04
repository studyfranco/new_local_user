#!/bin/bash

mkdir -p "${CODEX_UPLOAD_DIR:-/tmp/uploads}" /tmp/codex-run 2>/dev/null || true

# Optional: bridge a remote TCP/WS codex container to a local Unix socket if CODEX_REMOTE_TCP is set (e.g. codex_cli:4500)
if [[ -n "${CODEX_REMOTE_TCP:-}" ]]; then
    export CODEX_RUNTIME_DIR="/tmp/codex-run"
    rm -f /tmp/codex-run/app.sock
    socat UNIX-LISTEN:/tmp/codex-run/app.sock,fork TCP:"${CODEX_REMOTE_TCP}" >/tmp/socat-codex.log 2>&1 &
fi

# 1. Start ACP-UI Web Server (connects to Goose `goose serve` ACP & ACP WebSocket agents) on port 3000
python3 -m http.server "${ACP_UI_PORT:-3000}" --bind 0.0.0.0 --directory /opt/acp-ui/dist-web >/tmp/acp-ui.log 2>&1 &

# 2. Start Codex-Web UI (connects to `codex app-server` via /run/codex/app.sock) on port 8000
cd /opt/codex-web
exec python3 /opt/codex-web/app.py
