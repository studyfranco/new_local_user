# new_local_user — Layer 3 : Claude Code CLI + Audio (`claude-audio`)

Image Docker **Claude Code CLI + gestion de bibliothèque musicale** (`ghcr.io/studyfranco/new_local_user:claude-audio`), basée sur `ghcr.io/studyfranco/new_local_user:claude`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Outils Audio & Authentification à distance
- **Outils Audio** : `beets`, `picard` (`xvfb`), `shntool`, `flac`, `wavpack`, `fdupes`, `dos2unix`, `recode`, `coreutils`
- **Agent CLI** : Anthropic Claude Code (`claude`) avec `--remote-control` (hérité du layer 2 `claude`)
- **Tokens d'authentification headless / remote** : `CLAUDE_CODE_OAUTH_TOKEN` (via `claude setup-token`) ou `ANTHROPIC_API_KEY`

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  claude-audio:
    image: ghcr.io/studyfranco/new_local_user:claude-audio
    container_name: claude_audio
    restart: unless-stopped
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    environment:
      - remote_name=claude_audio
      - CLAUDE_CODE_OAUTH_TOKEN=${CLAUDE_CODE_OAUTH_TOKEN:-}
      # - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY:-}
    tmpfs:
      - /tmp:exec,mode=1777
    volumes:
      - ./data/claude:/home/thenewuser/.claude
      - ./data/config:/home/thenewuser/.config
      - /srv/music:/srv/music
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` de Claude :
```bash
docker exec -it claude_audio tmux attach -t the_new_user
```

## Last Update
2026-11-04
