#!/bin/bash

mkdir -p "${XDG_CONFIG_HOME:-/home/thenewuser/.config}/goose" \
         "${XDG_DATA_HOME:-/home/thenewuser/.local/share}/goose" \
         "${XDG_STATE_HOME:-/home/thenewuser/.local/state}/goose" 2>/dev/null || true
cd /tmp

export GOOSE_MODE="${GOOSE_MODE:-auto}"

# Start remote ACP server (goose serve) in the background when GOOSE_SERVER__SECRET_KEY is provided
if [[ -n "${GOOSE_SERVER__SECRET_KEY:-}" ]]; then
    goose serve --host "${GOOSE_HOST:-0.0.0.0}" --port "${GOOSE_PORT:-3284}" >/tmp/goose-serve.log 2>&1 &
fi

goose session --resume || goose session
