# new_local_user & Simply Agent Hub — AI Agent Context & Directives

Hello AI Agent! You are assisting with the development and maintenance of the `new_local_user` repository and its integrated **Simply Agent Hub** (`web-ui/`). Read and strictly adhere to these directives before inspecting, modifying, or adding code.

---

## 📄 Document & Repository Hierarchy

1. **`agent.md`** *(This file)*: Core architectural rules, branch topology, quality constraints, security directives, and testing workflows.
2. **`web-ui/api.yml`**: Official OpenAPI 3.1.0 specification for Simply Agent Hub REST & WebSocket endpoints.
3. **`README.md`**: User-facing overview of the project and current layer capabilities.
4. **`Dockerfile`**: Multi-stage Debian Trixie (13) rootless container image definition with KDE Plasma X11/xrdp and developer toolchains.
5. **`web-ui/`**: Complete Rust (Axum 0.8 + SeaORM 2.0) and Vanilla JS application directory.

---

## 🎯 Project Overview & Mission

`new_local_user` is a dual-layer ecosystem designed to host, orchestrate, and observe autonomous AI CLI agents in a secure, rootless Linux container environment:

1. **The Container Environment (Layer 1)**:
   - Base system: Debian GNU/Linux 13 (Trixie).
   - Dedicated non-root developer user: `thenewuser` (UID/GID 1000).
   - Desktop & Terminal: KDE Plasma X11, XRDP, tmux, zsh, bash.
   - Developer tooling: `rustup`, `cargo`, `rust-clippy`, `rust-src`, `rustfmt`, `python3`, `strace`, `py-spy`, `git`, `curl`, `jq`.
   - Security: Zero-root in runtime (`USER thenewuser`), rootless podman/docker friendly.

2. **Simply Agent Hub (Layer 2 — `web-ui/`)**:
   - Enterprise-grade Web UI and API written in **Rust (Edition 2024, Axum 0.8, SeaORM 2.0)**.
   - High-fidelity **KDE Plasma Breeze Dark** frontend with **Vanilla HTML5/CSS3/ES2022** (Strictly zero npm/React/Vue/Tailwind/CDN dependencies).
   - Unified multi-agent management for **Claude Code**, **Antigravity CLI (AGY)**, **OpenAI Codex CLI**, and **Goose CLI (ACP)**.
   - Real-time native **ANSI Tmux Terminal** streamed over bi-directional WebSockets without heavy external xterm.js dependencies.
   - Multi-agent orchestration channels with automated role-based dispatch (`@AgentName`), configurable message retention, and Markdown export.
   - Integrated Git Diff, Status, Commit, and Revert dashboard.
   - Native Internationalization (**i18n**) with English by default and real-time French switcher.

---

## 🌿 Multi-Branch Architecture & Git Workflow

The repository is organized across specialized branches. Agents must preserve this separation:

| Branch Name | Purpose & Contents |
| :--- | :--- |
| `master` | Base Debian rootless developer container image and foundational tooling. |
| `web` | **Simply Agent Hub** (`web-ui/`), Axum server, agent-bridge, SeaORM schema, and Web UI. |
| `claude`, `claude-dev`, `claude-audio`, `claude-video` | Specialized container variants preconfigured for Claude Code CLI. |
| `antigravity`, `antigravity-dev`, `antigravity-audio`, `antigravity-video` | Specialized container variants for Google Antigravity CLI. |
| `codex`, `codex-dev`, `codex-audio`, `codex-video` | Specialized container variants for OpenAI Codex CLI. |
| `goose`, `goose-dev`, `goose-audio`, `goose-video` | Specialized container variants for Goose Agent Communication Protocol (ACP). |

### Git Rules for AI Agents:
- **Never commit build artifacts**: `target/`, `*.db`, `chrome_profile/`, or node-like artifacts must never be staged. Check `git status` before committing.
- **Maintain branch scope**: Work intended for Simply Agent Hub belongs on `web`. Base container modifications belong on `master`.
- **Atomic, Conventional Commits**: Use `feat(...)`, `fix(...)`, `test(...)`, `refactor(...)`, or `docs(...)` adhering to existing commit history.

---

## 🛠️ Tech Stack & Hard Constraints

### 1. Backend (Rust Edition 2024)
- **Framework:** Axum 0.8+ with Tokio 1.x asynchronous runtime.
- **Allocator:** `tikv-jemallocator = "0.7"` for high-throughput, low-fragmentation memory management.
- **ORM (SQL-Agnostic):** `sea-orm = "2"` & `sea-orm-migration = "2"`.
  - Supports **SQLite** (`sqlite://...` or `sqlite::memory:`), **PostgreSQL** (`postgres://...`), and **MariaDB/MySQL** (`mysql://...`) without recompilation.
  - **Zero Raw SQL for DML**: Never use `execute_unprepared` or raw SQL strings for data manipulation. Always use typed SeaORM Entity models and ActiveModels (`Entity::find()`, `ActiveModel`).
  - Automatic migration runner executed upon startup in `db::connect_and_migrate`.
- **Quality Standard (Zero Tolerance)**:
  - **Zero compiler warnings**.
  - **Zero clippy warnings**: Code must pass `cargo clippy --all-targets -- -D warnings`.
  - **Zero `.unwrap()` or `.expect()` in production code**: Use structured `Result<T, E>`, Axum's `StatusCode`, and JSON error responses.

### 2. Frontend (Vanilla Web)
- **Zero NPM / Zero External CDNs**: Strictly Vanilla ES2022 JavaScript, semantic HTML5, and pure CSS3.
- **Visual Design**: Strict adherence to the **KDE Plasma Breeze Dark** aesthetic (slate dark backgrounds `#1e2430` / `#161b24`, cyan borders `#3daee9`, neon accents `#2ecc71`, `#e74c3c`, monospaced typography).
- **Internationalization (i18n)**:
  - English is the default language.
  - All translatable strings belong in the `I18N = { en: {...}, fr: {...} }` dictionary in `web-ui/static/app.js`.
  - HTML elements use `data-i18n="key"` attributes for automated DOM text binding.
  - Preference is persisted in `localStorage.getItem("agent_hub_lang")`.
- **AST Syntax Verification**: Any modification to `static/app.js` is automatically parsed and validated via the `oxc_parser` test in `web-ui/tests/frontend_syntax_test.rs`.

---

## 🏛️ System Architecture & Components

```
                          ┌──────────────────────────────────────────────┐
                          │     Web Browser / Headless Chromium CDP      │
                          │   (KDE Breeze Dark UI - EN / FR Switcher)    │
                          └──────────────┬───────────────────────────────┘
                                         │ HTTP REST & WebSocket (ANSI / Bus)
                                         ▼
┌────────────────────────────────────────────────────────────────────────┐
│               Simply Agent Hub (`agent-hub` - Port 8080)               │
├────────────────────────────────────────────────────────────────────────┤
│ • Axum Router & Web Static Server                                      │
│ • SeaORM 2.0 Multi-Backend Persistence (SQLite / Postgres / MySQL)     │
│ • Native Tmux Terminal WebSocket Proxy (`/ws/instances/{id}/terminal`) │
│ • Multi-Agent Orchestration Engine & Auto-Dispatch (`@AgentName`)      │
│ • Retention Manager (Days / Max Messages Purging)                      │
│ • Git Status, Commit, Diff & Revert Controller                         │
└──────┬────────────────────┬────────────────────┬─────────────────┬─────┘
       │ HTTP RPC           │ HTTP RPC           │ WebSocket       │ WebSocket / HTTP
       ▼                    ▼                    ▼                 ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐  ┌──────────────┐
│ agent-bridge │     │ agent-bridge │     │  Codex CLI   │  │  Goose CLI   │
│   (Claude)   │     │ (Antigravity)│     │  (Port 14500)│  │ (ACP Server) │
│ (Port 13285) │     │ (Port 13286) │     └──────────────┘  └──────────────┘
└──────┬───────┘     └──────┬───────┘
       │ tmux capture/keys  │ tmux capture/keys
       ▼                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                  Local Tmux Sessions in Rootless Linux                 │
│              (the_new_user@container:~$ running CLI agents)             │
└────────────────────────────────────────────────────────────────────────┘
```

### 1. Unified Agent Bridge (`agent-bridge`)
Located at `web-ui/src/bin/agent_bridge.rs`:
- Lightweight, standalone JSON-RPC 2.0 daemon running alongside CLI tools in the container.
- Translates JSON-RPC calls into tmux operations:
  - `prompt/send`: Submits prompt text to the target agent and captures response.
  - `tmux/capture`: Grabs live terminal output with full ANSI color escapes (`tmux capture-pane -e -p`).
  - `tmux/send_raw`: Injects interactive raw keystrokes (Enter, Backspace, arrows, Ctrl+C).
  - `tmux/send_keys`: Injects single command lines.

### 2. Multi-Agent Orchestration & Role Presets
- Pre-packaged roles with customized instructions in both English and French:
  - `architect`: Software Architect (`🏗️ Software Architect` / `🏗️ Architecte Logiciel`).
  - `developer`: Lead Developer (`💻 Lead Developer` / `💻 Développeur Principal`).
  - `reviewer`: Code Reviewer & QA (`🔍 Code Reviewer & QA` / `🔍 Code Reviewer & Qualité`).
  - `tester`: QA & E2E Test Engineer (`🧪 QA & E2E Test Engineer` / `🧪 Ingénieur QA & Tests E2E`).
  - `security`: Security & DevSecOps Auditor (`🛡️ Security & DevSecOps Auditor` / `🛡️ Auditeur Sécurité & DevSecOps`).
  - `general`: Versatile Agent (`🤖 Versatile Agent` / `🤖 Agent Polyvalent`).
- **Onboarding Workflow**:
  - `POST /api/instances/{id}/onboard` sends a structured identity and role prompt into the target CLI session.
- **Context-Aware Auto-Dispatch**:
  - When a message in an orchestration channel mentions `@AgentName`, Simply Agent Hub extracts the recipient, prefixes the message with channel context and role identity, dispatches it to the agent, and streams the reply back to the channel bus.

### 3. Configurable Retention Engine
- Channels enforce two retention limits stored in SeaORM:
  - `retention_days`: Messages older than `N` days are automatically pruned.
  - `max_messages`: Hard cap keeping only the `N` most recent messages.
- Automatic pruning occurs on channel updates and message publishing; manual pruning is exposed via `POST /api/orchestration/channels/{id}/prune`.
- Complete chat logs can be exported as clean GitHub Flavored Markdown via `GET /api/orchestration/channels/{id}/export`.

---

## 🧪 Testing Directives & Verification

All contributions must pass the three-tiered test suite before submission:

```bash
cd web-ui

# 1. Clippy Zero-Warning Gate
cargo clippy --all-targets -- -D warnings

# 2. Rust Unit Tests & Integration Tests
cargo test --bin agent-bridge --bin agent-hub

# 3. JavaScript Syntax & i18n Static Validation (oxc_parser)
cargo test --test frontend_syntax_test

# 4. Real-world Headless Chromium E2E Test
cargo test --test chromium_full_scale_test
```

### What `chromium_full_scale_test.rs` validates:
- Real local tmux session creation (`the_new_user`).
- Live `agent-bridge` daemon spawning for Claude and Antigravity.
- Axum `agent-hub` spawning on temporary SQLite database.
- Headless Chromium `/usr/bin/chromium` connecting over Chrome DevTools Protocol (CDP WebSocket).
- Instances discovery and status polling.
- Live bi-directional ANSI terminal streaming and command execution (`echo CHROMIUM_TMUX_OK`).
- Orchestration channel publishing and message rendering.
- Git workspace status detection.
- Dynamic i18n switching (`EN` ➔ `FR` ➔ `EN`) with DOM text assertions.
- Role onboarding submission.
- Retention policy modification.
- Automated PNG screenshot generation stored in the brain artifacts folder.

---

## ⚙️ Environment Variables Reference

| Variable | Default Value | Description |
| :--- | :--- | :--- |
| `PORT` | `8080` | Port on which Simply Agent Hub listens. |
| `DATABASE_URL` | `sqlite:///data/agent_hub.db?mode=rwc` | SeaORM connection string (SQLite, PostgreSQL, or MySQL). |
| `DEFAULT_CWD` | `/srv` | Default working directory for CLI instances and Git operations. |
| `CLAUDE_RPC_URL` | `http://127.0.0.1:13285/rpc` | JSON-RPC endpoint for Claude Code agent bridge. |
| `ANTIGRAVITY_RPC_URL` | `http://127.0.0.1:13286/rpc` | JSON-RPC endpoint for Antigravity agent bridge. |
| `CODEX_WS_URL` | `ws://127.0.0.1:14500` | Direct WebSocket endpoint for OpenAI Codex CLI. |
| `CODEX_UNIX_SOCKET` | `/tmp/codex.sock` | Optional Unix Domain Socket for Codex communication. |
| `GOOSE_WS_URL` | `ws://127.0.0.1:13284/acp` | Agent Communication Protocol (ACP) WebSocket endpoint for Goose. |
| `GOOSE_HTTP_URL` | `http://127.0.0.1:13284` | HTTP proxy endpoint for Goose web interface. |
| `BRIDGE_SECRET` | *(None)* | Optional Bearer token for authenticating `agent-bridge` RPC calls. |
| `RUST_LOG` | `info` | Logging level (`debug`, `info`, `warn`, `error`). |

---

## 🚀 Golden Rules for Future Agents

1. **Keep it Rootless**: Never run production services as `root`. All development and container execution must remain unprivileged under `thenewuser` / `fallrik`.
2. **Never Break i18n Parity**: Any new UI label added to `index.html` or `app.js` must exist in both `en` and `fr` dictionaries.
3. **Preserve Zero-Unwrap**: Every error must be mapped to a clean HTTP response or logged via `tracing::error!`.
4. **Preserve Documentation Integrity**: Update `agent.md` and `web-ui/api.yml` whenever adding, modifying, or removing endpoints or core architectural behaviors.
