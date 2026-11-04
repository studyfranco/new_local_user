# new_local_user — Layer 1 : Base Généraliste (`master`)

Image Docker de base généraliste (`ghcr.io/studyfranco/new_local_user:master`), conçue pour être **100 % rootless** (`USER thenewuser` `1000:1000`, sans `init.sh` ni `gosu`/`chown`) et **read-only ready** (`read_only: true`).

## Architecture en 3 Layers (18 branches)

```text
Layer 1 (Base généraliste)
└── master (contient uniquement run.sh)
    ├── Layer 2 (Agents CLI & Web UI — ajoutent uniquement leur start-cli.sh)
    │   ├── claude        (Anthropic Claude Code CLI)
    │   │   ├── claude-audio
    │   │   ├── claude-video
    │   │   └── claude-dev
    │   ├── antigravity   (Google Antigravity CLI - agy)
    │   │   ├── antigravity-audio
    │   │   ├── antigravity-video
    │   │   └── antigravity-dev
    │   ├── goose         (AAIF / Block Goose CLI + goose serve ACP)
    │   │   ├── goose-audio
    │   │   ├── goose-video
    │   │   └── goose-dev
    │   ├── codex         (OpenAI Codex CLI + codex app-server)
    │   │   ├── codex-audio
    │   │   ├── codex-video
    │   │   └── codex-dev
    │   └── web           (Interfaces Web ACP-UI & Codex-Web pour se connecter à goose serve / codex app-server)
```

## Outils inclus dans `master`
- **Base** : `ghcr.io/studyfranco/docker-baseimages-debian:testing-video`
- **Système & Réseau** : `build-essential`, `netcat-openbsd`, `lsof`, `procps`, `jq`, `rsync`, `zip`, `7zip`, `pkg-config`, `openssl`, `curl`, `git`, `strace`, `tmux`
- **Python** : `python3-venv`, `python3-pip`, `py-spy`
- **Utilisateur non-root** : `thenewuser` (`UID=1000`, `GID=1000`)

## Conception Rootless & Read-Only (`read_only: true`)
- **Aucun `init.sh` root** : L'`ENTRYPOINT` est directement `/usr/local/bin/run.sh` exécuté sous `USER thenewuser` (`1000:1000`). Compatible avec Docker Rootless, Podman Rootless, `user: "1000:1000"`, `cap_drop: [ALL]` et `no-new-privileges:true`.
- **Héritage propre des scripts** : `run.sh` dans `master` lance `/usr/local/bin/start-cli.sh` s'il est fourni par un Layer 2 (sinon `/bin/bash`). Les branches filles ne redéfinissent jamais les scripts des couches parentes.

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
