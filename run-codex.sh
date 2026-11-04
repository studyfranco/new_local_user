#!/bin/bash

mkdir -p "${CODEX_HOME:-/home/thenewuser/.codex}" 2>/dev/null || true
cd /tmp

# Authenticate automatically via stdin token if OPENAI_API_KEY or CODEX_API_KEY is set,
# or fall back to remote device-code auth if not yet logged in
if [[ -n "${OPENAI_API_KEY:-}" ]]; then
    printenv OPENAI_API_KEY | codex login --with-api-key
elif [[ -n "${CODEX_API_KEY:-}" ]]; then
    printenv CODEX_API_KEY | codex login --with-api-key
elif ! codex login status >/dev/null 2>&1; then
    codex login --device-auth || true
fi

# Start codex app-server on Unix socket (for codex-web container) if /run/codex is writable
if [[ -w "/run/codex" ]]; then
    rm -f /run/codex/app.sock
    codex app-server --listen unix:///run/codex/app.sock >/tmp/codex-app-server-unix.log 2>&1 &
fi

# Start codex app-server on WebSocket if CODEX_PORT is set
if [[ -n "${CODEX_PORT:-}" ]]; then
    codex app-server --listen "ws://${CODEX_HOST:-0.0.0.0}:${CODEX_PORT}" >/tmp/codex-app-server-ws.log 2>&1 &
fi

FLAGS=(--dangerously-bypass-approvals-and-sandbox
       --add-dir "/srv" --add-dir "/tmp" --add-dir "/mnt")
codex resume --last "${FLAGS[@]}" || codex "${FLAGS[@]}"
