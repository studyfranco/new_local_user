#!/bin/bash
set -e

export PORT="${PORT:-8080}"
export DATABASE_URL="${DATABASE_URL:-sqlite:///tmp/agent_hub.db?mode=rwc}"
export DEFAULT_CWD="${DEFAULT_CWD:-/srv}"

echo "=================================================================="
echo " Simply Agent Hub (Rust Axum 0.8 + SeaORM 2) sur 0.0.0.0:${PORT}"
echo " -> Base de données SeaORM : ${DATABASE_URL}"
echo "=================================================================="

exec /usr/local/bin/agent-hub
