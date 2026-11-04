# new_local_user — Branche `web` (Layer 2 : Agent Hub — Rust API + Native JS Web UI)

Cette image héberge **Agent Hub** (`/usr/local/bin/agent-hub`), une application Web unifiée développée sur mesure :
- **Backend en Rust** (`axum`, `tokio`, `tokio-tungstenite`, `reqwest`) servant à la fois l'interface Web statique (embarquée dans le binaire via `include_str!`) et le proxy/pont API & WebSocket temps réel.
- **Frontend en JavaScript Natif** (sans dépendance lourde externe) permettant de piloter depuis une seule interface :
  1. **Codex (`codex app-server`)** en JSON-RPC 2.0 via **WebSocket** (`ws://codex:4500`) ou **Socket Unix** partagé (`/run/codex/app.sock`).
  2. **Goose (`goose serve`)** via le protocole **ACP (Agent Client Protocol)** sur **WebSocket** (`ws://goose:3284/acp`) ou **HTTP JSON-RPC** (`http://goose:3284/acp`) avec injection côté serveur de la clé `X-Secret-Key` (`GOOSE_SERVER__SECRET_KEY`).
- **Conçu pour Traefik** : écoute en HTTP pur sur `0.0.0.0:8080` (`PORT`), la terminaison TLS/HTTPS étant déléguée au reverse-proxy Traefik.
- **100% Rootless (`1000:1000`) & Read-Only Ready** : le binaire Rust autonome ne nécessite aucune écriture sur le système de fichiers racine.

---

## Endpoints du serveur Rust (`agent-hub`)

| Route | Description |
| :--- | :--- |
| `GET /`, `/app.js`, `/style.css` | Interface Web unifiée en JavaScript Natif |
| `GET /healthz` | Healthcheck JSON (`{"status":"ok"}`) pour Docker / Traefik |
| `GET /api/config` | Configuration par défaut détectée depuis les variables d'environnement |
| `GET /api/status` | Sonde TCP / Socket Unix en temps réel de la disponibilité de `goose` et `codex` |
| `GET /ws/codex` | Pont WebSocket bidirectionnel vers `codex app-server` (`?mode=ws` ou `?mode=unix`) |
| `GET /ws/goose` | Pont WebSocket ACP vers `goose serve` (`/acp`) avec injection de `X-Secret-Key` |
| `ANY /api/goose/*path` | Reverse-proxy HTTP vers `http://goose:3284/*path` avec authentification `X-Secret-Key` |

---

## Variables d'environnement

| Variable | Défaut | Description |
| :--- | :--- | :--- |
| `PORT` | `8080` | Port HTTP d'écoute du binaire Rust |
| `GOOSE_HTTP_URL` | `http://goose:3284` | URL HTTP interne du conteneur Goose |
| `GOOSE_WS_URL` | `ws://goose:3284/acp` | URL WebSocket ACP interne du conteneur Goose |
| `GOOSE_SERVER__SECRET_KEY` | *(vide)* | Clé secrète partagée avec `goose serve` (injectée par le backend Rust sans l'exposer) |
| `CODEX_WS_URL` | `ws://codex:4500` | URL WebSocket interne du conteneur Codex (`codex app-server`) |
| `CODEX_UNIX_SOCKET` | `/run/codex/app.sock` | Chemin du socket Unix Codex (si volume partagé entre `codex` et `web`) |
| `DEFAULT_CWD` | `/srv` | Répertoire de travail par défaut transmis lors de la création de sessions/threads |

---

## Exemple `docker-compose.yml` complet (Derrière Traefik + Rootless + Read-Only)

```yaml
services:
  web:
    image: ghcr.io/studyfranco/new_local_user:web
    container_name: agent-hub-web
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    environment:
      - PORT=8080
      - GOOSE_HTTP_URL=http://goose:3284
      - GOOSE_WS_URL=ws://goose:3284/acp
      - GOOSE_SERVER__SECRET_KEY=${GOOSE_SECRET_KEY:-change_me_secret}
      - CODEX_WS_URL=ws://codex:4500
      - CODEX_UNIX_SOCKET=/run/codex/app.sock
      - DEFAULT_CWD=/srv
    tmpfs:
      - /tmp:rw,exec,nosuid,nodev,uid=1000,gid=1000,size=128m
    volumes:
      - codex-sock:/run/codex
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.agent-hub.rule=Host(`agents.example.com`)"
      - "traefik.http.routers.agent-hub.entrypoints=websecure"
      - "traefik.http.routers.agent-hub.tls.certresolver=letsencrypt"
      - "traefik.http.services.agent-hub.loadbalancer.server.port=8080"
    depends_on:
      - goose
      - codex
    networks:
      - traefik-public
      - agents-internal
    restart: unless-stopped

  goose:
    image: ghcr.io/studyfranco/new_local_user:goose
    container_name: goose-agent
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    environment:
      - GOOSE_PROVIDER=anthropic
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - GOOSE_SERVER__SECRET_KEY=${GOOSE_SECRET_KEY:-change_me_secret}
    tmpfs:
      - /tmp:rw,exec,nosuid,nodev,uid=1000,gid=1000,size=512m
    volumes:
      - ./goose-config:/home/thenewuser/.config/goose
      - ./goose-data:/home/thenewuser/.local/share/goose
      - ./workspace:/srv
    networks:
      - agents-internal
    restart: unless-stopped

  codex:
    image: ghcr.io/studyfranco/new_local_user:codex
    container_name: codex-agent
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    environment:
      - OPENAI_API_KEY=${OPENAI_API_KEY}
    tmpfs:
      - /tmp:rw,exec,nosuid,nodev,uid=1000,gid=1000,size=512m
    volumes:
      - ./codex-home:/home/thenewuser/.codex
      - codex-sock:/run/codex
      - ./workspace:/srv
    networks:
      - agents-internal
    restart: unless-stopped

volumes:
  codex-sock:

networks:
  traefik-public:
    external: true
  agents-internal:
    driver: bridge
```

## Last Update
2026-11-04
