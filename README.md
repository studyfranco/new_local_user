# new_local_user — Branche `web` (Layer 2 : Simply Agent Hub v0.2 — SeaORM + Multi-CLI + Orchestration)

Cette image héberge **Simply Agent Hub** (`/usr/local/bin/agent-hub`), une application Web et API en **Rust (Edition 2024)** + **JavaScript Natif** alignée sur l'architecture et la charte **KDE Plasma Breeze Dark** de `simply_ip_vault` / `simply_ip_sync` :

- **Frameworks alignés en versions maximales (`Cargo.toml`)** :
  - `axum = "0.8"` (HTTP & WebSocket)
  - `sea-orm = "2"` & `sea-orm-migration = "2"` (`sqlx-sqlite`, `sqlx-postgres`, `sqlx-mysql` / MariaDB, `runtime-tokio-rustls`)
  - `reqwest = "0.13"` (`rustls`), `tokio-tungstenite = "0.28"`, `tokio = "1"`, `tikv-jemallocator = "0.7"`
  - `oxc_parser = "=0.152.0"` pour la validation syntaxique ECMAScript de `static/app.js` lors de `cargo test`.
- **Gestion multi-instances CLI & Renommage en base de données (`SeaORM`)** :
  - Menu déroulant à gauche listant toutes vos instances CLI (ex. *3 Codex*, *2 Antigravity*, *1 Claude*, *1 Goose*) avec possibilité d'**ajouter**, **renommer** (`PUT /api/instances/{id}`) et **supprimer** chaque instance.
  - Persistance SQL agnostique via `DATABASE_URL` : **SQLite** par défaut (`sqlite:///tmp/agent_hub.db?mode=rwc` ou volume `/data`), commutable sans recompilation vers **PostgreSQL** (`postgres://user:pass@db:5432/agent_hub`) ou **MariaDB / MySQL** (`mysql://user:pass@db:3306/agent_hub`).
- **Authentification des conteneurs depuis l'interface Web (`POST /api/instances/{id}/auth`)** :
  - Vérification du statut d'authentification (`status`), génération du **code Device Auth / lien OAuth** (`device_start`), injection directe d'une **clé API** (`api_key`), ou validation d'une **URL de callback OAuth** (`oauth_callback`).
- **Canaux d'Orchestration Temps Réel (`/api/orchestration/channels` & `/ws/orchestration/{id}`)** :
  - Créez des canaux dédiés où les conteneurs CLI sélectionnés se connectent et échangent des messages en temps réel (persistés dans `channel_messages` et diffusés sur le bus WebSocket).

---

## Endpoints de l'API Rust (`agent-hub`)

| Méthode & Route | Description |
| :--- | :--- |
| `GET /`, `/style.css`, `/app.js` | Interface Web Simply Agent Hub (KDE Breeze Dark) |
| `GET /healthz`, `GET /ready` | Sondes de vie et de disponibilité de la base SeaORM |
| `GET /api/config`, `GET /api/status` | Configuration par défaut et sonde des ports par défaut |
| `GET /api/instances` | Liste toutes les instances CLI enregistrées en base SeaORM avec leur état en ligne |
| `POST /api/instances` | Ajoute une nouvelle instance CLI (`codex`, `antigravity`, `claude`, `goose`) |
| `GET / PUT / DELETE /api/instances/{id}` | Consulte, renomme/modifie ou supprime une instance CLI |
| `POST /api/instances/{id}/auth` | Authentifie le conteneur depuis la Web UI (`status`, `device_start`, `api_key`, `oauth_callback`) |
| `POST /api/agents/ask` | Interroge directement une instance par `instance_id`, `name` ou `kind` |
| `POST /api/agents/tmux` | Capture la sortie `tmux` en direct du conteneur ciblé |
| `GET / POST /api/orchestration/channels` | Liste ou crée un canal d'orchestration multi-CLIs |
| `GET / PUT / DELETE /api/orchestration/channels/{id}` | Consulte, met à jour les participants ou supprime un canal |
| `GET /api/orchestration/channels/{id}/messages` | Historique persisté des messages échangés sur le canal |
| `POST /api/orchestration/channels/{id}/publish` | Publie un message sur le canal et déclenche les réponses des CLIs participants |
| `GET /ws/orchestration/{id}` | Bus WebSocket temps réel du canal d'orchestration `{id}` |
| `GET /ws/codex`, `GET /ws/goose` | Ponts WebSocket directs (`?instance_id=<uuid>`) |

---

## Tests automatisés (`cargo test` & `./scripts/test_e2e.sh`)

```bash
# Validation Rust + validation syntaxique ECMAScript de static/app.js (oxc_parser)
cargo test

# Suite de tests E2E complète en curl (CRUD SeaORM, renommage, auth Web, canaux d'orchestration)
./scripts/test_e2e.sh
```

## Last Update
2026-11-06
