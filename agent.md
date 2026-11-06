# Simply Agent Hub (`new_local_user`) — AI Agent Context & Directives

Hello AI Agent! You are assisting with the development and maintenance of the `new_local_user` project and its integrated **Simply Agent Hub** (`web-ui/`). Read and strictly adhere to these guidelines before inspecting, authoring, or modifying any code.

---

## 📄 Document & Context Hierarchy

To maintain complete architectural clarity and modularity, responsibilities are split across dedicated files:

1. **`agent.md`** *(This file, symlinked as `AGENT.MD`)*: Core architectural rules, security boundaries, multi-branch topology, quality constraints, and testing directives.
2. **`AGENT_NOTES.MD`**: Living agent worklog. You **MUST** update this file with architectural decisions, audit results, and completed tasks after any significant modification.
3. **`web-ui/api.yml`**: Official OpenAPI 3.1.0 specification for all REST endpoints, WebSocket streaming channels, and JSON data schemas.
4. **`Dockerfile`**: Multi-stage, multi-tier container definition based on Debian Trixie (13) with rootless user execution and developer toolchains.
5. **`web-ui/`**: Complete Rust (Axum 0.8 + SeaORM 2.0) and Vanilla JS application directory.

---

## 🔄 Peer Ecosystem & Convergence (`simply_` Family)

`new_local_user` is not an isolated experiment. It is the **runtime execution environment and multi-agent control plane** for the entire `simply_` ecosystem:

| Peer Service | Domain & Role in Ecosystem |
| :--- | :--- |
| `simply_ip_vault` | Central Firewall and ACL management API with HMAC-SHA256 zero-trust security and RBAC. |
| `simply_hook_executor` | Signed execution of server-side webhook actions and secure process execution. |
| `simply_ip_sync` | Feed ingestion and multi-vault database synchronization worker. |
| `simply_ip_exporter` | High-performance filtered IP feed distributor for edge network appliances. |
| **`new_local_user` / `Simply Agent Hub`** | **The developer workstation & autonomous agent orchestrator**: houses CLI agents (`claude`, `antigravity`, `codex`, `goose`), provides live tmux terminals, orchestrates collective workflows, manages Git commits, and tests peer services. |

Because `Simply Agent Hub` shares lineage with the `simply_` family:
- It adheres to the same **KDE Plasma Breeze Dark** visual specification.
- It enforces the same **strict zero-panic, zero-clippy, zero-warning** Rust engineering standards.
- It enforces the same **SQL-agnostic SeaORM multi-database** portability (SQLite, PostgreSQL, MariaDB/MySQL).
- It enforces the same **zero-external-dependency Vanilla JS** frontend architecture.

---

## 🎯 Project Overview & Mission

`new_local_user` provides two tightly integrated operational layers:

### Layer 1: The Rootless Developer Container Environment
- **Base OS**: Debian GNU/Linux 13 (Trixie).
- **Unprivileged User**: `thenewuser` / `fallrik` (UID/GID 1000). Zero root execution in runtime.
- **Desktop & Workspace**: KDE Plasma X11, XRDP server, tmux, zsh, bash.
- **Developer Toolchain**: Rust (`rustup`, `cargo`, `rust-clippy`, `rust-src`, `rustfmt`), Python 3, debugging and profiling tools (`strace`, `py-spy`), Git, `jq`, `curl`.

### Layer 2: Simply Agent Hub (`web-ui/`)
- **Backend**: Rust 2024, Axum 0.8+, Tokio 1.x, SeaORM 2.0+ with `tikv-jemallocator`.
- **Frontend**: Vanilla ES2022 JavaScript, HTML5, CSS3 matching KDE Plasma Breeze Dark.
- **CLI Agents Supported**: Claude Code CLI, Google Antigravity CLI (AGY), OpenAI Codex CLI, Goose CLI (Agent Communication Protocol).
- **Native Terminal**: Bi-directional WebSocket streaming live ANSI tmux screen frames and raw keystrokes without external xterm.js dependencies.
- **Orchestration**: Real-time multi-agent communication channels with `@AgentName` auto-dispatch, role-based system prompt injection, and configurable message retention.
- **Git Dashboard**: Direct status inspection, unified diffs, atomic staging/committing, and reversion.
- **Internationalization (i18n)**: English by default with dynamic French language switcher and `localStorage` persistence.

---

## 🛠️ Tech Stack & Hard Constraints

### 1. Backend Architecture (Rust Edition 2024)
- **Framework:** Axum 0.8+ with Tokio 1.x.
- **Memory Allocator:** `tikv-jemallocator = "0.7"` for high-performance memory allocation and low fragmentation.
- **ORM (SQL-Agnostic):** `sea-orm = "2"` & `sea-orm-migration = "2"`.
  - Compatible with **SQLite** (`sqlite://...` or `sqlite::memory:`), **PostgreSQL** (`postgres://...`), and **MariaDB / MySQL** (`mysql://...`).
  - **Zero Raw SQL for DML**: Never use `execute_unprepared`, raw string queries, or string interpolations for `SELECT`, `INSERT`, `UPDATE`, or `DELETE`. Use typed SeaORM Entity models (`Entity::find()`, `ActiveModel`) exclusively.
  - **Automated Startup Migration**: Migrations run automatically on application startup via `db::connect_and_migrate`.
- **Quality Standard (Zero Tolerance)**:
  - **Zero compiler warnings**.
  - **Zero clippy warnings**: Enforced via `cargo clippy --all-targets -- -D warnings`.
  - **Zero `.unwrap()` or `.expect()` in production code**: Every error must map to Axum `StatusCode` and structured JSON responses (`{"error": "..."}`).
  - **Mandatory rustdoc**: Document all public structs, enums, functions, and Axum handlers with `///`.

### 2. Frontend Architecture (Vanilla Web)
- **Strictly Zero External Dependencies**:
  - **NO** React, Vue, Svelte, or Angular.
  - **NO** Tailwind, Bootstrap, or CSS frameworks.
  - **NO** external CDN `<script>` or `<link>` tags.
  - **NO** NPM, webpack, Vite, or node_modules runtime requirement.
  - Native browser `fetch()`, DOM APIs, and WebSocket APIs only.
- **Visual Design Identity**:
  - Authentic **KDE Plasma Breeze Dark** color scheme:
    - Base backgrounds: `#161b24`, `#1e2430`, `#232a36`.
    - Accent & Borders: Cyan `#3daee9`, highlight `#2ecc71`, danger `#e74c3c`.
    - Typography: System sans-serif with `JetBrains Mono` / `Consolas` monospaced fonts for terminal, code, and diffs.
- **Internationalization (i18n) & Extensible Language Packs**:
  - Default language is **English** (`en`), with out-of-the-box **German** (`de`) and **French** (`fr`) support.
  - Automatic browser language detection via `detectBrowserLanguage()` (`navigator.languages` / `navigator.language`), overridable at will by the user with persistence in `localStorage`.
  - Extensible runtime language pack registration via `window.registerLanguagePack(code, dictionary)`.
  - Dynamic switching at will via `window.setAppLanguage('en' | 'fr' | 'de' | ...)` with immediate DOM translation.
- **ECMAScript Syntax Verification**:
  - All JavaScript changes are statically parsed and validated via the `oxc_parser` test in `web-ui/tests/frontend_syntax_test.rs`.

---

## 🌿 Multi-Branch Topology & Git Workflow

The `new_local_user` repository employs a structured multi-branch architecture:

| Branch | Purpose & Focus |
| :--- | :--- |
| `master` | Base Debian rootless developer container image, system packages, and base toolchain. |
| `web` | **Simply Agent Hub** (`web-ui/`), Axum server, agent-bridge, SeaORM entities, and web assets. |
| `claude`, `claude-dev`, `claude-audio`, `claude-video` | Specialized container variants for Claude Code CLI and dedicated media tiers. |
| `antigravity`, `antigravity-dev`, `antigravity-audio`, `antigravity-video` | Specialized container variants for Google Antigravity CLI and media tiers. |
| `codex`, `codex-dev`, `codex-audio`, `codex-video` | Specialized container variants for OpenAI Codex CLI and media tiers. |
| `goose`, `goose-dev`, `goose-audio`, `goose-video` | Specialized container variants for Goose Agent Communication Protocol. |

### Git Rules for AI Agents:
1. **Never commit build directories**: `target/`, temporary SQLite databases (`*.db`), or browser profiles (`chrome_profile/`) must never be tracked by Git. Always verify with `git status`.
2. **Branch Hygiene**: Work for Simply Agent Hub belongs on `web`. Base container modifications belong on `master`.
3. **Conventional Commits**: Every commit message must follow conventional commit standards (`feat(...)`, `fix(...)`, `test(...)`, `docs(...)`, `refactor(...)`).

---

## 🏛️ Core Architectural Rules

### 1. Security & Rootless Execution Boundary
- **Zero-Root Runtime**: Services run strictly under the unprivileged user (`thenewuser`, UID 1000). Never elevate privileges or execute daemon processes as `root`.
- **Local Bridge Isolation**: Agent bridges (`agent-bridge`) bind to loopback (`127.0.0.1`) only. When `BRIDGE_SECRET` is defined, requests must carry `Authorization: Bearer <token>`.
- **WebSocket Terminal Isolation**: Terminal access (`/ws/instances/{id}/terminal`) validates instance existence and active tmux sessions before upgrading protocols.

### 2. Multi-Agent Collective Orchestration & Auto-Dispatch
- **Database Schema**:
  - `cli_instances`: Persists registered agent CLIs, endpoints, assigned roles, and custom prompts.
  - `orchestration_channels`: Persists shared rooms, retention rules, and metadata.
  - `channel_messages`: Persists message history, sender/recipient metadata, and timestamps.
  - `channel_participants`: M:N link table tracking instances participating in a channel.
- **Multilingual Role Presets**:
  - Six standard roles: `architect`, `developer`, `reviewer`, `tester`, `security`, `general`.
  - Roles provide pre-configured system instructions in both English and French via `db::default_role_prompt_lang(role, lang)`.
- **Onboarding Pipeline (`POST /api/instances/{id}/onboard`)**:
  - Transmits a structured identity, role, and channel protocol briefing into the agent's active CLI session.
- **Context-Aware Auto-Dispatch**:
  - When a message contains `@AgentName`, Simply Agent Hub extracts the target instance, wraps the prompt with contextual channel metadata and role instructions, invokes the agent via HTTP-RPC or WebSocket, and automatically publishes the agent's reply back to the channel.

### 3. Configurable Retention Engine & Markdown Export
- **Retention Rules**:
  - `retention_days`: Messages older than `N` days are automatically purged.
  - `max_messages`: Strict cap retaining only the `N` most recent messages.
- **Purge Execution**:
  - Automatic pruning triggers during channel updates (`PUT /api/orchestration/channels/{id}`) and upon new message publication.
  - Manual pruning is exposed via `POST /api/orchestration/channels/{id}/prune`.
- **Exporting Conversations**:
  - `GET /api/orchestration/channels/{id}/export` formats the channel transcript into clean GitHub Flavored Markdown with timestamps, sender badges, and retention headers.

### 4. Native Tmux ANSI Terminal & WebSocket Streaming
- **Zero-Dependency Terminal**:
  - Terminal output is captured directly from tmux with color escapes: `tmux capture-pane -e -p -t <session>`.
  - The ANSI stream is piped over WebSocket to the browser client, which renders colors and text using a lightweight CSS ANSI parser.
- **Raw Keystroke Protocol**:
  - Keystrokes, special keys (Enter, Backspace, arrows), and signals (Ctrl+C) are sent as JSON payloads (`{"type": "raw", "data": "...", "special": bool}`) and forwarded to tmux via `tmux send-keys`.

### 5. Git Workspace Operations
- **Status & Diff Tracking (`GET /api/git/status`)**:
  - Reads active Git branch, uncommitted modified files, recent commits, and runs `git diff HEAD` to return unified diffs.
- **Atomic Commits (`POST /api/git/commit`)**:
  - Executes `git add -A` and `git commit -m "{message}"` cleanly.
- **Reversion (`POST /api/git/revert`)**:
  - Allows reverting an individual modified file (`git checkout -- <file>`) or the entire workspace (`git checkout -- . && git clean -fd`).

---

## 🧪 Testing Directives & Quality Gates

Every code contribution must pass the full test suite before staging:

```bash
cd web-ui

# 1. Clippy Gate (Zero Warnings)
cargo clippy --all-targets -- -D warnings

# 2. Rust Unit & Integration Tests
cargo test --bin agent-bridge --bin agent-hub

# 3. JavaScript Syntax & i18n Static Verification (oxc_parser)
cargo test --test frontend_syntax_test

# 4. Real-World Headless Chromium E2E Test Suite
cargo test --test chromium_full_scale_test
```

### Full-Scale Chromium E2E Test Scope:
The test [`web-ui/tests/chromium_full_scale_test.rs`](file:///home/fallrik/Documents/workspaces/new_local_user/web-ui/tests/chromium_full_scale_test.rs) launches:
1. A real local tmux session `the_new_user`.
2. Real local `agent-bridge` daemons for Claude (13285) and Antigravity (13286).
3. The Axum `agent-hub` server on port 18089 with a fresh SQLite database.
4. Headless Chromium (`/usr/bin/chromium`) connected via Chrome DevTools Protocol (CDP WebSocket).
5. Validates:
   - Container Fleet table rendering (4 default CLI instances).
   - Local Claude online status detection.
   - Interactive native tmux terminal connection, streaming, and command input (`echo CHROMIUM_TMUX_OK`).
   - Multi-agent orchestration message publishing.
   - Git branch badge and diff viewer.
   - Dynamic i18n switching (`EN` ➔ `FR` ➔ `EN`) with live DOM translation assertions.
   - Role onboarding workflow.
   - Channel retention policy updates.
   - Automated generation of 7 PNG screenshot artifacts in the brain directory.

---

## ⚙️ Environment Variables Reference

| Variable | Default Value | Description |
| :--- | :--- | :--- |
| `PORT` | `8080` | Port on which Simply Agent Hub listens. |
| `DATABASE_URL` | `sqlite:///data/agent_hub.db?mode=rwc` | SeaORM connection string (SQLite, PostgreSQL, or MariaDB/MySQL). |
| `DEFAULT_CWD` | `/srv` | Default working directory for CLI agents and Git repository operations. |
| `CLAUDE_RPC_URL` | `http://127.0.0.1:13285/rpc` | JSON-RPC 2.0 endpoint for Claude Code agent bridge. |
| `ANTIGRAVITY_RPC_URL` | `http://127.0.0.1:13286/rpc` | JSON-RPC 2.0 endpoint for Google Antigravity agent bridge. |
| `CODEX_WS_URL` | `ws://127.0.0.1:14500` | Direct WebSocket bridge endpoint for OpenAI Codex CLI. |
| `CODEX_UNIX_SOCKET` | `/tmp/codex.sock` | Unix Domain Socket for Codex communication. |
| `GOOSE_WS_URL` | `ws://127.0.0.1:13284/acp` | Agent Communication Protocol (ACP) WebSocket endpoint for Goose. |
| `GOOSE_HTTP_URL` | `http://127.0.0.1:13284` | HTTP proxy endpoint for Goose web interface. |
| `BRIDGE_SECRET` | *(None)* | Optional Bearer token for authenticating `agent-bridge` RPC calls. |
| `RUST_LOG` | `info` | Tracing and log filter level (`debug`, `info`, `warn`, `error`). |

---

## 🚀 Golden Rules for Future AI Agents

1. **Keep it Rootless**: Never execute application or CLI tasks as `root`. The non-root sandbox is a fundamental security guarantee.
2. **Preserve i18n Symmetry**: Whenever you add a new UI element, form field, button, or message, provide the corresponding translations in both `en` and `fr` dictionaries in `static/app.js`.
3. **Preserve Zero-Unwrap**: Never introduce `.unwrap()` or `.expect()` into production code. Use Axum's `IntoResponse` and structured error returns.
4. **Preserve Documentation Integrity**: Update `agent.md`, `AGENT_NOTES.MD`, and `web-ui/api.yml` whenever modifying database schemas, endpoints, or system behaviors.
