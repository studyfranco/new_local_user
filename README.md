# new_local_user — Layer 3 : Claude Code CLI + Video (`claude-video`)

Image Docker **Claude Code CLI + gestion de médias vidéo** (`ghcr.io/studyfranco/new_local_user:claude-video`), basée sur `ghcr.io/studyfranco/new_local_user:claude`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Outils Vidéo & Authentification à distance
- **Outils Vidéo** : `mnamer`, `atomicparsley` (ainsi que les outils vidéo de l'image de base `testing-video`)
- **Agent CLI** : Anthropic Claude Code (`claude`) avec `--remote-control` (hérité du layer 2 `claude`)
- **Tokens d'authentification headless / remote** : `CLAUDE_CODE_OAUTH_TOKEN` (via `claude setup-token`) ou `ANTHROPIC_API_KEY`

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  claude-video:
    image: ghcr.io/studyfranco/new_local_user:claude-video
    container_name: claude_video
    restart: unless-stopped
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    environment:
      - remote_name=claude_video
      - CLAUDE_CODE_OAUTH_TOKEN=${CLAUDE_CODE_OAUTH_TOKEN:-}
      # - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY:-}
    tmpfs:
      - /tmp:exec,mode=1777
    volumes:
      - ./data/claude:/home/thenewuser/.claude
      - ./data/config:/home/thenewuser/.config
      - /srv/videos:/srv/videos
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` de Claude :
```bash
docker exec -it claude_video tmux attach -t the_new_user
```

## Last Update
2026-11-04
