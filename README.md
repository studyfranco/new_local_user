# new_local_user — Layer 2 : Hub Web UI (`web`) pour `goose serve` & `codex app-server`

Image Docker **Web UI Hub** (`ghcr.io/studyfranco/new_local_user:web`), basée sur `ghcr.io/studyfranco/new_local_user:master`. Conçue pour être **100 % rootless** (`1000:1000`) et **read-only ready** (`read_only: true`).

Contrairement à Claude (`claude --remote-control`) et Antigravity (`agy --remote-control`) qui disposent déjà de leur propre tableau de bord cloud officiel, **Goose** (`goose serve`) et **Codex** (`codex app-server`) exposent des serveurs locaux/réseau. Ce conteneur fournit les deux interfaces Web open-source permettant de s'y connecter :

1. **ACP-UI (`formulahendry/acp-ui`) — Port `3000`** :
   - Client Web pour le protocole **ACP (Agent Client Protocol)**.
   - Permet de se connecter en WebSocket à **`goose serve`** (`ws://<ip-ou-host>:3284/acp?token=<GOOSE_SERVER__SECRET_KEY>`) ou tout autre agent compatible ACP.
2. **Codex-Web (`sjtrny/codex-web`) — Port `8000`** :
   - Interface Web dédiée à **`codex app-server`**.
   - Se connecte au socket partagé `/run/codex/app.sock` du conteneur `codex` (et partage les conversations en temps réel avec le terminal `tmux` de `codex`).

## Exemple `docker-compose.yml` complet (`web` + `goose` + `codex` en Rootless & Read-Only)

```yaml
services:
  web:
    image: ghcr.io/studyfranco/new_local_user:web
    container_name: agents_web_ui
    restart: unless-stopped
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    ports:
      - "3000:3000" # ACP-UI (pour Goose serve / ACP)
      - "8000:8000" # Codex-Web (pour Codex app-server)
    environment:
      - ACP_UI_PORT=3000
      - CODEX_RUNTIME_DIR=/run/codex
      - CODEX_WORKSPACE_ROOT=/srv
      - CODEX_DEFAULT_CWD=/srv
    tmpfs:
      - /tmp:exec,mode=1777
    volumes:
      - ./data/codex-run:/run/codex:ro
      - /srv:/srv:ro

  goose:
    image: ghcr.io/studyfranco/new_local_user:goose
    container_name: goose_cli
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
      - ./data/goose/config:/home/thenewuser/.config/goose
      - ./data/goose/share:/home/thenewuser/.local/share/goose
      - ./data/goose/state:/home/thenewuser/.local/state/goose
      - /srv:/srv

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
    tmpfs:
      - /tmp:exec,mode=1777
    volumes:
      - ./data/codex:/home/thenewuser/.codex
      - ./data/codex-run:/run/codex
      - /srv:/srv
```

## Utilisation
- **Interface Web Goose / ACP** : ouvrir `http://localhost:3000` et renseigner `ws://<host>:3284/acp?token=<GOOSE_SERVER__SECRET_KEY>`
- **Interface Web Codex** : ouvrir `http://localhost:8000`

## Last Update
2026-11-04
