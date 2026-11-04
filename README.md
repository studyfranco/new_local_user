# new_local_user — Layer 1 : Base Généraliste (`master`)

Image Docker de base généraliste (`ghcr.io/studyfranco/new_local_user:master`), conçue pour être **rootless** (`USER thenewuser` `1000:1000`) et **read-only ready** (`read_only: true`).

## Architecture en 3 Layers

```text
Layer 1 (Base généraliste)
└── master
    ├── Layer 2 (Agents CLI)
    │   ├── claude        (Anthropic Claude Code CLI)
    │   │   ├── claude-audio
    │   │   └── claude-video
    │   ├── antigravity   (Google Antigravity CLI - agy)
    │   │   ├── antigravity-audio
    │   │   └── antigravity-video
    │   ├── goose         (AAIF / Block Goose CLI)
    │   │   ├── goose-audio
    │   │   └── goose-video
    │   └── codex         (OpenAI Codex CLI)
    │       ├── codex-audio
    │       └── codex-video
```

## Outils inclus dans `master`
- **Base** : `ghcr.io/studyfranco/docker-baseimages-debian:testing-video`
- **Système & Réseau** : `build-essential`, `netcat-openbsd`, `lsof`, `procps`, `jq`, `rsync`, `zip`, `7zip`, `pkg-config`, `openssl`, `curl`, `strace`, `tmux`
- **Python** : `python3-venv`, `python3-pip`, `py-spy`
- **Utilisateur non-root** : `thenewuser` (`UID=1000`, `GID=1000`)

## Compatibilité Rootless & Read-Only (`read_only: true`)
- Le conteneur s'exécute par défaut sous l'utilisateur non-root `thenewuser` (`1000:1000`).
- Tous les scripts d'initialisation (`/init.sh`, `/usr/local/bin/run.sh`) sont installés dans le système de fichiers racine en lecture seule.
- Les caches et sockets d'exécution (`TMUX_TMPDIR`, `XDG_RUNTIME_DIR`, `XDG_CACHE_HOME`, `CARGO_HOME`, `PYTHONPYCACHEPREFIX`, `PIP_CACHE_DIR`, `NPM_CONFIG_CACHE`) pointent vers `/tmp` (à monter en `tmpfs`).

## Exemple `docker-compose.yml`

```yaml
services:
  base-shell:
    image: ghcr.io/studyfranco/new_local_user:master
    container_name: new_local_user_base
    restart: unless-stopped
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    tmpfs:
      - /tmp:exec,mode=1777
    volumes:
      - ./data/home:/home/thenewuser
      - /srv:/srv
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` active :
```bash
docker exec -it new_local_user_base tmux attach -t the_new_user
```

## Last Update
2026-11-04
