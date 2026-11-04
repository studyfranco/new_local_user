# new_local_user — Layer 2 : Google Antigravity CLI (`antigravity`)

Image Docker pour **Google Antigravity CLI (`agy`)** (`ghcr.io/studyfranco/new_local_user:antigravity`), basée sur `ghcr.io/studyfranco/new_local_user:master`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Fonctionnalités & Authentification à distance
- **Agent CLI** : Google Antigravity CLI (`agy`) installé dans `/usr/local/bin/agy`
- **Mode autonome & Remote Control** : Lancé automatiquement par `/usr/local/bin/run.sh` (hérité de `master`) via `/usr/local/bin/start-cli.sh` (`run-antigravity.sh`) dans une session `tmux` (`the_new_user`) avec `--dangerously-skip-permissions --remote-control` et reprise automatique (`--continue`).
- **Tokens d'authentification headless / remote** :
  - `GEMINI_API_KEY` : Si définie, configure automatiquement `"modelProvider": "gemini"` dans `~/.gemini/antigravity-cli/settings.json` pour une authentification directe par token sans navigateur.
  - **Remote OAuth Flow** : Si aucune clé API n'est fournie, `SSH_TTY` active automatiquement le flux OAuth distant par URL/code à copier-coller.
- **Configuration persistante** : Stockée dans `/home/thenewuser/.gemini`

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  antigravity:
    image: ghcr.io/studyfranco/new_local_user:antigravity
    container_name: antigravity_cli
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
      - /srv:/srv
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` d'Antigravity :
```bash
docker exec -it antigravity_cli tmux attach -t the_new_user
```

## Last Update
2026-11-04
