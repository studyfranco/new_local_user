# new_local_user — Branche `web` (Layer 2 : Simply Agent Hub — Rust API + Native JS UI)

Cette image héberge **Simply Agent Hub** (`/usr/local/bin/agent-hub`), une application Web unifiée développée sur mesure reprenant la charte graphique **KDE Plasma Breeze Dark** de `simply_ip_vault` / `simply_ip_sync` :
- **Backend en Rust** (`axum`, `tokio`, `tokio-tungstenite`, `reqwest`) servant à la fois l'interface Web statique (embarquée dans le binaire via `include_str!`) et le routeur d'orchestration inter-agents.
- **Frontend en JavaScript Natif** (palette `#232629` / `#31363b`, accents Breeze Green `#1cdc9a` et Breeze Blue `#3daee9`) permettant de piloter et de faire dialoguer :
  1. **Codex (`codex app-server`)** en JSON-RPC 2.0 via **WebSocket** (`ws://codex:4500`) ou **Socket Unix** (`/run/codex/app.sock`).
  2. **Goose (`goose serve`)** via le protocole **ACP (Agent Client Protocol)** sur **WebSocket** (`ws://goose:3284/acp`) ou **HTTP JSON-RPC** (`http://goose:3284/acp`) avec injection côté serveur de la clé `X-Secret-Key` (`GOOSE_SERVER__SECRET_KEY`).
  3. **Claude Code (`claude`)** via le pont JSON-RPC/Tmux intégré (`http://claude:3285/rpc`).
  4. **Antigravity (`agy`)** via le pont JSON-RPC/Tmux intégré (`http://antigravity:3286/rpc`).

---

## Communication Inter-Agents (Codex $\leftrightarrow$ Goose $\leftrightarrow$ Claude $\leftrightarrow$ Antigravity)

Trois mécanismes complémentaires permettent aux 4 agents de collaborer :

1. **Salon Multi-Agents & Pipeline Séquentiel (Onglet « Pont Inter-Agents » de la Web UI)** :
   - Enchaîne automatiquement jusqu'à 3 agents sur une même mission (ex. *Étape 1 : Claude (Architecture)* $\rightarrow$ *Étape 2 : Codex (Implémentation)* $\rightarrow$ *Étape 3 : Goose (Vérification)*), chaque agent recevant le contexte et les réponses des étapes précédentes.
2. **Routage automatique des mentions `@codex`, `@goose`, `@claude`, `@antigravity`** :
   - Lorsqu'un agent mentionne un pair dans sa réponse (ex. `@codex Peux-tu vérifier ce test ?`), le serveur Rust intercepte la mention, interroge automatiquement l'agent ciblé via `/api/agents/ask`, et injecte sa réponse dans la conversation.
3. **Commande CLI interne `agent-ask` disponible dans tous les conteneurs** :
   - Depuis son propre terminal interne, n'importe quel agent (`claude`, `antigravity`, `goose`, `codex`) peut interroger un autre conteneur en exécutant :
     ```bash
     agent-ask codex "Analyse le fichier /srv/main.rs et propose un correctif"
     agent-ask goose "Exécute la suite de tests dans /srv"
     agent-ask claude "Vérifie la sécurité de ce patch"
     agent-ask antigravity "Génère le plan de refactoring"
     ```

---

## Endpoints du serveur Rust (`agent-hub`)

| Route | Description |
| :--- | :--- |
| `GET /`, `/app.js`, `/style.css` | Interface Web Simply Agent Hub (thème KDE Breeze Dark) |
| `GET /ready`, `GET /healthz` | Sondes de disponibilité JSON (`{"status":"ok"}`) pour Docker `HEALTHCHECK` et Traefik |
| `GET /api/config` | Configuration par défaut détectée depuis les variables d'environnement |
| `GET /api/status` | Sonde TCP / Socket Unix en temps réel des 4 conteneurs (`codex`, `goose`, `claude`, `antigravity`) |
| `POST /api/agents/ask` | Routeur unifié inter-agents (`{"agent":"codex\|goose\|claude\|antigravity","prompt":"..."}`) |
| `POST /api/agents/tmux` | Capture en direct du terminal `tmux` (`the_new_user`) de `claude` ou `antigravity` |
| `GET /ws/codex` | Pont WebSocket bidirectionnel vers `codex app-server` (`?mode=ws` ou `?mode=unix`) |
| `GET /ws/goose` | Pont WebSocket ACP vers `goose serve` (`/acp`) avec injection de `X-Secret-Key` |
| `ANY /api/goose/*path` | Reverse-proxy HTTP vers `http://goose:3284/*path` avec authentification `X-Secret-Key` |

---

## Exemple `docker-compose.yml` complet (Derrière Traefik + 4 Agents + Rootless + Read-Only)

```yaml
services:
  web:
    image: ghcr.io/studyfranco/new_local_user:web
    container_name: simply-agent-hub
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
      - CLAUDE_RPC_URL=http://claude:3285/rpc
      - ANTIGRAVITY_RPC_URL=http://antigravity:3286/rpc
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
    networks:
      - traefik-public
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
      - AGENT_HUB_URL=http://web:8080
    tmpfs:
      - /tmp:rw,exec,nosuid,nodev,uid=1000,gid=1000,size=512m
    volumes:
      - ./codex-home:/home/thenewuser/.codex
      - codex-sock:/run/codex
      - ./workspace:/srv
    networks:
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
      - AGENT_HUB_URL=http://web:8080
    tmpfs:
      - /tmp:rw,exec,nosuid,nodev,uid=1000,gid=1000,size=512m
    volumes:
      - ./goose-config:/home/thenewuser/.config/goose
      - ./goose-data:/home/thenewuser/.local/share/goose
      - ./workspace:/srv
    networks:
      - agents-internal
    restart: unless-stopped

  claude:
    image: ghcr.io/studyfranco/new_local_user:claude
    container_name: claude-agent
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    environment:
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - AGENT_BRIDGE_PORT=3285
      - AGENT_HUB_URL=http://web:8080
    tmpfs:
      - /tmp:rw,exec,nosuid,nodev,uid=1000,gid=1000,size=512m
    volumes:
      - ./claude-config:/home/thenewuser/.claude
      - ./workspace:/srv
    networks:
      - agents-internal
    restart: unless-stopped

  antigravity:
    image: ghcr.io/studyfranco/new_local_user:antigravity
    container_name: antigravity-agent
    user: "1000:1000"
    read_only: true
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    environment:
      - GEMINI_API_KEY=${GEMINI_API_KEY}
      - AGENT_BRIDGE_PORT=3286
      - AGENT_HUB_URL=http://web:8080
    tmpfs:
      - /tmp:rw,exec,nosuid,nodev,uid=1000,gid=1000,size=512m
    volumes:
      - ./antigravity-config:/home/thenewuser/.gemini
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
2026-11-05
