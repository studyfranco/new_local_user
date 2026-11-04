#!/bin/bash

GEMINI_DIR="${HOME:-/home/thenewuser}/.gemini/antigravity-cli"
SETTINGS_FILE="${GEMINI_DIR}/settings.json"
mkdir -p "${GEMINI_DIR}" 2>/dev/null || true

# If GEMINI_API_KEY is provided, configure modelProvider="gemini" automatically as required by Antigravity CLI docs
if [[ -n "${GEMINI_API_KEY:-}" ]] && [[ -w "${GEMINI_DIR}" ]]; then
    if [[ -f "${SETTINGS_FILE}" ]]; then
        jq '.modelProvider = "gemini"' "${SETTINGS_FILE}" > "${SETTINGS_FILE}.tmp" && mv "${SETTINGS_FILE}.tmp" "${SETTINGS_FILE}"
    else
        printf '{\n  "modelProvider": "gemini"\n}\n' > "${SETTINGS_FILE}"
    fi
fi

# Force remote/headless OAuth URL flow when no local browser is available
export SSH_TTY="${SSH_TTY:-/dev/pts/0}"

cd /tmp

FLAGS=(--dangerously-skip-permissions --remote-control
       --add-dir "/srv" --add-dir "/tmp" --add-dir "/mnt")
agy "${FLAGS[@]}" --continue || agy "${FLAGS[@]}"
