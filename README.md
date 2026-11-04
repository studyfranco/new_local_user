# new_local_user — Layer 2 : Claude Code CLI (`claude`)

Image Docker pour **Claude Code CLI** (`ghcr.io/studyfranco/new_local_user:claude`), basée sur `ghcr.io/studyfranco/new_local_user:master`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Fonctionnalités & Authentification à distance
- **Agent CLI** : Anthropic Claude Code (`claude`)
- **Mode autonome & Remote Control** : Lancé automatiquement par `/usr/local/bin/run.sh` (hérité de `master`) via `/usr/local/bin/start-cli.sh` (`run-claude.sh`) dans une session `tmux` (`the_new_user`) avec `--dangerously-skip-permissions --remote-control "${remote_name}"` et reprise automatique (`--continue`).
- **Tokens d'authentification headless / remote** :
  - `CLAUDE_CODE_OAUTH_TOKEN` : Token OAuth d'abonnement Claude (généré via `claude setup-token`).
  - `ANTHROPIC_API_KEY` (ou `ANTHROPIC_AUTH_TOKEN`) : Clé API Anthropic directe.
- **Configuration persistante** : Stockée dans `CLAUDE_CONFIG_DIR=/home/thenewuser/.claude`

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  claude:
    image: ghcr.io/studyfranco/new_local_user:claude
    container_name: claude_cli
    restart: unless-stopped
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    environment:
      - remote_name=claude_code
      - CLAUDE_CODE_OAUTH_TOKEN=${CLAUDE_CODE_OAUTH_TOKEN:-}
      # - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY:-}
    tmpfs:
      - /tmp:exec,mode=1777
    volumes:
      - ./data/claude:/home/thenewuser/.claude
      - /srv:/srv
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` de Claude :
```bash
docker exec -it claude_cli tmux attach -t the_new_user
```

## Last Update
2026-11-04
