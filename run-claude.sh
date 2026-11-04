#!/bin/bash

mkdir -p "${CLAUDE_CONFIG_DIR:-/home/thenewuser/.claude}" 2>/dev/null || true
cd /tmp

FLAGS=(--dangerously-skip-permissions --remote-control "${remote_name}"
       --add-dir "/srv" "/tmp" "/mnt")
claude "${FLAGS[@]}" --continue || claude "${FLAGS[@]}"
