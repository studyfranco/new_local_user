# new_local_user — Layer 2 : OpenAI Codex CLI (`codex`)

Image Docker pour **OpenAI Codex CLI (`codex`)** (`ghcr.io/studyfranco/new_local_user:codex`), basée sur `ghcr.io/studyfranco/new_local_user:master`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

## Fonctionnalités & Serveur `codex app-server`
- **Agent CLI** : OpenAI Codex CLI (`codex`) installé via le script officiel (`https://chatgpt.com/codex/install.sh`) dans `/opt/codex` et `/usr/local/bin/codex`
- **Mode autonome** : Lancé automatiquement par `/usr/local/bin/run.sh` (hérité de `master`) via `/usr/local/bin/start-cli.sh` (`run-codex.sh`) dans une session `tmux` (`the_new_user`) avec `--dangerously-bypass-approvals-and-sandbox` et reprise automatique (`codex resume --last`).
- **Serveur distant `codex app-server`** :
  - Démarre `codex app-server --listen unix:///run/codex/app.sock` (pour partager les sessions avec l'interface Web de la branche `web`) et `codex app-server --listen ws://0.0.0.0:4500` (pour `codex --remote ws://...`).
- **Tokens d'authentification headless / remote** :
  - `OPENAI_API_KEY` ou `CODEX_API_KEY` : Importé automatiquement via `printenv ... | codex login --with-api-key`.
  - **Device Auth (`--device-auth`)** : Si aucun token n'est défini, exécute `codex login --device-auth`.

## Exemple `docker-compose.yml` (Rootless & Read-Only)

```yaml
services:
  codex:
    image: ghcr.io/studyfranco/new_local_user:codex
    container_name: codex_cli
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
      - /srv:/srv
      - /mnt:/mnt
```

## Utilisation
Rejoindre la session `tmux` de Codex :
```bash
docker exec -it codex_cli tmux attach -t the_new_user
```

## Last Update
2026-11-04
