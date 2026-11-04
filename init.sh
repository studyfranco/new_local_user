#!/bin/bash

set -e

CURRENTUID=$(id -u)
NUMCHECK='^[0-9]+$'
USER="thenewuser"

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

# Rootless mode: running directly as non-root user
if [[ "$CURRENTUID" -ne "0" ]]; then
    if [[ -x "/usr/local/bin/run.sh" ]]; then
        exec "/usr/local/bin/run.sh"
    else
        exec "/home/thenewuser/run.sh"
    fi
fi

# Root mode: optional PUID/PGID mapping when started as UID 0
if ! [[ "$PGID" =~ $NUMCHECK ]] ; then
    printf "Invalid group id given: %s\n" "$PGID"
    PGID="1000"
elif [[ "$PGID" -eq 0 ]]; then
    printf "PGID/group cannot be 0 (root)\n"
    exit 1
fi

if ! [[ "$PUID" =~ $NUMCHECK ]] ; then
    printf "Invalid user id given: %s\n" "$PUID"
    PUID="1000"
elif [[ "$PUID" -eq 0 ]]; then
    printf "PUID/user cannot be 0 (root)\n"
    exit 1
fi

if [[ -w /etc/group ]]; then
    if [[ $(getent group "$PGID" | cut -d: -f1) ]]; then
        usermod -a -G "$PGID" "$USER" || true
    else
        groupmod -g "$PGID" "$USER" || true
    fi
fi

if [[ -w /etc/passwd ]]; then
    if [[ $(getent passwd "${PUID}" | cut -d: -f1) ]]; then
        USER=$(getent passwd "$PUID" | cut -d: -f1)
    else
        usermod -u "$PUID" "$USER" || true
    fi
fi

chown -R "$PUID":"$PGID" /config /home/thenewuser /tmp 2>/dev/null || true

if [[ -x "/usr/local/bin/run.sh" ]]; then
    exec gosu "${PUID}:${PGID}" "/usr/local/bin/run.sh"
else
    exec gosu "${PUID}:${PGID}" "/home/thenewuser/run.sh"
fi
