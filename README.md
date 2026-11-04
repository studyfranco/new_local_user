# new_local_user — Layer 3 : OpenAI Codex CLI + Video (`codex-video`)

Image Docker **OpenAI Codex CLI (`codex`) + gestion de médias vidéo** (`ghcr.io/studyfranco/new_local_user:codex-video`), basée sur `ghcr.io/studyfranco/new_local_user:codex`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Outils Vidéo & Serveur `codex app-server`
- **Outils Vidéo** : `mnamer`, `atomicparsley` (ainsi que les outils vidéo de l'image de base `testing-video`)
- **Agent CLI** : OpenAI Codex CLI (`codex`) + `codex app-server` (`/run/codex/app.sock` et `ws://0.0.0.0:4500`, hérités du layer 2 `codex`)
- **Tokens d'authentification headless / remote** : `OPENAI_API_KEY` ou `CODEX_API_KEY` (ou `--device-auth`).

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  codex-video:
    image: ghcr.io/studyfranco/new_local_user:codex-video
    container_name: codex_video
    restart: unless-stopped
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    ports:
      - "4500:4500"
    environment:
      - CODEX_HOME=/home/thenewuser/.codex
      - OPENAI_API_KEY=${OPENAI_API_KEY:-}
      # - CODEX_API_KEY=${CODEX_API_KEY:-}
    tmpfs:
      - /tmp:exec,mode=1777
    volumes:
      - ./data/codex:/home/thenewuser/.codex
      - ./data/codex-run:/run/codex
      - ./data/config:/home/thenewuser/.config
      - /srv/videos:/srv/videos
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` de Codex :
```bash
docker exec -it codex_video tmux attach -t the_new_user
```

## Last Update
2026-11-04
