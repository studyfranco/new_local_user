#!/bin/bash

set -euo pipefail

# Prepare ephemeral runtime and cache directories in /tmp (compatible with read_only: true)
mkdir -p "${XDG_RUNTIME_DIR:-/tmp/runtime-thenewuser}" \
         "${XDG_CACHE_HOME:-/tmp/.cache}" \
         "${CARGO_HOME:-/tmp/cargo}" \
         "${PYTHONPYCACHEPREFIX:-/tmp/pycache}" \
         "${PIP_CACHE_DIR:-/tmp/.cache/pip}" \
         "${NPM_CONFIG_CACHE:-/tmp/.cache/npm}" 2>/dev/null || true
chmod 700 "${XDG_RUNTIME_DIR:-/tmp/runtime-thenewuser}" 2>/dev/null || true

# Prepare XDG user directories if HOME is mounted writable
if [[ -w "${HOME:-/home/thenewuser}" ]]; then
    mkdir -p "${XDG_CONFIG_HOME:-/home/thenewuser/.config}" \
             "${XDG_DATA_HOME:-/home/thenewuser/.local/share}" \
             "${XDG_STATE_HOME:-/home/thenewuser/.local/state}" \
             "${HOME:-/home/thenewuser}/.local/bin" 2>/dev/null || true
fi

cd /tmp
if [[ -x "/usr/local/bin/start-cli.sh" ]]; then
    tmux new-session -d -s the_new_user /usr/local/bin/start-cli.sh
else
    tmux new-session -d -s the_new_user /bin/bash
fi
tail -f /dev/null

exit 0
