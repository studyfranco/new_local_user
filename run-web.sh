#!/bin/bash
set -e

export PORT="${PORT:-8080}"
export GOOSE_HTTP_URL="${GOOSE_HTTP_URL:-http://goose:3284}"
export GOOSE_WS_URL="${GOOSE_WS_URL:-ws://goose:3284/acp}"
export CODEX_WS_URL="${CODEX_WS_URL:-ws://codex:4500}"
export CODEX_UNIX_SOCKET="${CODEX_UNIX_SOCKET:-/run/codex/app.sock}"
export DEFAULT_CWD="${DEFAULT_CWD:-/srv}"

echo "=================================================================="
echo " Agent Hub (Rust API + Native JS UI) démarré sur 0.0.0.0:${PORT}"
echo " -> Prêt derrière un reverse-proxy Traefik (HTTP)"
echo " -> Cible Goose : ${GOOSE_WS_URL} (${GOOSE_HTTP_URL})"
echo " -> Cible Codex : ${CODEX_WS_URL} (ou socket ${CODEX_UNIX_SOCKET})"
echo "=================================================================="

exec /usr/local/bin/agent-hub
