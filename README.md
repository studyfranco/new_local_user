# new_local_user — Layer 3 : Goose CLI + Video (`goose-video`)

Image Docker **Goose CLI (`goose`) + gestion de médias vidéo** (`ghcr.io/studyfranco/new_local_user:goose-video`), basée sur `ghcr.io/studyfranco/new_local_user:goose`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Outils Vidéo & Serveur ACP distant
- **Outils Vidéo** : `mnamer`, `atomicparsley` (ainsi que les outils vidéo de l'image de base `testing-video`)
- **Agent CLI** : Goose CLI (`goose`) + `goose serve` (port `3284`, hérité du layer 2 `goose`)
- **Tokens d'authentification headless / remote** : `GOOSE_PROVIDER`, `GOOSE_MODEL`, `GOOSE_PROVIDER__API_KEY` et `GOOSE_SERVER__SECRET_KEY`.

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  goose-video:
    image: ghcr.io/studyfranco/new_local_user:goose-video
    container_name: goose_video
    restart: unless-stopped
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    ports:
      - "3284:3284"
    environment:
      - GOOSE_MODE=auto
      - GOOSE_PROVIDER=${GOOSE_PROVIDER:-openai}
      - GOOSE_MODEL=${GOOSE_MODEL:-gpt-4o}
      - GOOSE_PROVIDER__API_KEY=${GOOSE_PROVIDER__API_KEY:-}
      - GOOSE_SERVER__SECRET_KEY=${GOOSE_SERVER__SECRET_KEY:-change_me_secret_token}
    tmpfs:
      - /tmp:exec,mode=1777
    volumes:
      - ./data/goose/config:/home/thenewuser/.config
      - ./data/goose/share:/home/thenewuser/.local/share/goose
      - ./data/goose/state:/home/thenewuser/.local/state/goose
      - /srv/videos:/srv/videos
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` de Goose :
```bash
docker exec -it goose_video tmux attach -t the_new_user
```

## Last Update
2026-11-04
