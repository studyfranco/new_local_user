# new_local_user — Layer 3 : Antigravity CLI + Video (`antigravity-video`)

Image Docker **Google Antigravity CLI (`agy`) + gestion de médias vidéo** (`ghcr.io/studyfranco/new_local_user:antigravity-video`), basée sur `ghcr.io/studyfranco/new_local_user:antigravity`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Outils Vidéo & Authentification à distance
- **Outils Vidéo** : `mnamer`, `atomicparsley` (ainsi que les outils vidéo de l'image de base `testing-video`)
- **Agent CLI** : Google Antigravity CLI (`agy`) avec `--remote-control` (hérité du layer 2 `antigravity`)
- **Tokens d'authentification headless / remote** : `GEMINI_API_KEY` (configure automatiquement `"modelProvider": "gemini"`) ou flux OAuth distant (`SSH_TTY`).

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  antigravity-video:
    image: ghcr.io/studyfranco/new_local_user:antigravity-video
    container_name: antigravity_video
    restart: unless-stopped
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    environment:
      - GEMINI_API_KEY=${GEMINI_API_KEY:-}
    tmpfs:
      - /tmp:exec,mode=1777
    volumes:
      - ./data/gemini:/home/thenewuser/.gemini
      - ./data/config:/home/thenewuser/.config
      - /srv/videos:/srv/videos
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` d'Antigravity :
```bash
docker exec -it antigravity_video tmux attach -t the_new_user
```

## Last Update
2026-11-04
