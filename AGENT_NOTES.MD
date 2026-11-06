# AGENT NOTES — Simply Agent Hub (`new_local_user`)

> This file is the living worklog and audit record for the `new_local_user` project and its integrated
> **Simply Agent Hub** (`web-ui/`), adhering to the documentation standards of the `simply_` ecosystem.
> AI Agents working on this repository **MUST** record their architectural decisions, audit findings,
> and completed modification sweeps here.

---

## 2026-11-06 — Layer 2 Simply Agent Hub Implementation & Full-Scale Verification

### Scope & Architectural Goals
1. Establish Simply Agent Hub (`web-ui/`) as the central multi-agent orchestrator for `new_local_user`.
2. Align visual and engineering standards with the `simply_ip_vault` / `simply_ip_sync` ecosystem:
   - SeaORM 2.0 multi-database engine (SQLite default, PostgreSQL & MariaDB/MySQL compatible).
   - Zero-panic (`no .unwrap()` / `no .expect()` in production).
   - Zero clippy warnings (`cargo clippy --all-targets -- -D warnings`).
   - Pure Vanilla JS / CSS3 / HTML5 interface strictly adhering to the KDE Plasma Breeze Dark specification.
3. Deliver native interactive terminal streaming directly from tmux over WebSockets without external xterm.js dependencies.
4. Establish full English default with dynamic multi-language (i18n) support (EN / FR).
5. Unify and update Rust dependencies in `web-ui/Cargo.toml`.
6. Author comprehensive test suites including a full-scale real-world test driving headless Chromium via Chrome DevTools Protocol (CDP).
7. Create `agent.md` / `AGENT.MD` and `web-ui/api.yml` following the high standards of `simply_ip_vault`.

---

### Audit Findings & Resolutions

#### 1. Rust Dependencies Deduplication & Modernization
* **Finding**: `web-ui/Cargo.toml` used `tokio-tungstenite = "0.28"` while Axum 0.8.9 pulled in `tokio-tungstenite = "0.29"`, resulting in duplicated crate builds and potential WebSocket type incompatibilities. `base64` was also split across versions 0.22 and 0.23.
* **Resolution**:
  - Updated `tokio-tungstenite` to `"0.29"` in `web-ui/Cargo.toml`.
  - Updated `base64` to `"0.23"`.
  - Updated `oxc_allocator`, `oxc_parser`, `oxc_span` to `"=0.157.0"`.
  - Executed `cargo update`, eliminating duplicate crates and cleaning compilation times.

#### 2. Code Quality & Zero-Panic Mandate
* **Finding**: Several production handlers had raw `.unwrap()` calls, and clippy surfaced warnings regarding unneeded borrows and format strings.
* **Resolution**:
  - Replaced all production `.unwrap()` / `.expect()` calls with structured `Result`, Axum `StatusCode`, and JSON error responses.
  - Resolved all clippy lints across production and test targets.
  - Verified with `cargo clippy --all-targets -- -D warnings` (0 warnings, 0 errors).

#### 3. Internationalization (i18n), Browser Detection & Extensible Language Packs
* **Finding**: Prompts and UI labels were originally authored in French, lacking a structured multi-language dictionary, automatic browser detection, and modular extensibility for additional languages (such as German).
* **Resolution**:
  - Implemented automatic browser language detection via `detectBrowserLanguage()` inspecting `navigator.languages` / `navigator.language` while prioritizing explicit user preference in `localStorage`.
  - Added extensible language architecture with `registerLanguagePack(code, dictionary)` and dynamic switcher rendering (`renderLanguageSwitcher()`).
  - Added full German (`de`) localization across UI labels, role presets, and agent onboarding briefings (`[AGENTEN-ONBOARDING — SIMPLY AGENT HUB]`).
  - Updated `web-ui/src/db.rs` and `web-ui/src/main.rs` to support German presets and onboarding prompts (`/api/roles/presets?lang=de`).
  - Validated dynamic switching at will across EN, FR, DE, and dynamically installed language packs (e.g. ES) in the real-world Chromium CDP test suite, generating screenshot artifacts `chromium_08_i18n_german.png` and `chromium_09_i18n_dynamic_plugin.png`.

#### 4. Full-Scale Real-World Chromium CDP Test Suite
* **Finding**: Previous tests were limited to synthetic mock requests and lacked end-to-end browser verification.
* **Resolution**:
  - Authored [`web-ui/tests/chromium_full_scale_test.rs`](web-ui/tests/chromium_full_scale_test.rs) launching real local processes:
    - tmux session `the_new_user` with live ANSI text.
    - `agent-bridge` daemons for Claude (13285) and Antigravity (13286).
    - `agent-hub` on port 18089 with temporary SQLite database.
    - Headless Chromium (`/usr/bin/chromium --headless=new`) connected via CDP WebSocket.
  - Verified:
    - Container Fleet table rendering and local instance reachability.
    - Native terminal WebSocket streaming and raw command execution (`echo CHROMIUM_TMUX_OK`).
    - Multi-agent orchestration channel publishing.
    - Git branch and diff viewer.
    - Real-time dynamic i18n switching (`EN` ➔ `FR` ➔ `EN`).
    - Role onboarding and prompt auto-filling.
    - Retention policy updates.
  - Generated and verified 7 PNG screenshots stored in the brain artifacts directory:
    - `chromium_01_instances.png`
    - `chromium_02_terminal_natif.png`
    - `chromium_03_orchestration.png`
    - `chromium_04_git_viewer.png`
    - `chromium_05_i18n_french.png`
    - `chromium_06_role_onboarding.png`
    - `chromium_07_retention_modal.png`

#### 5. Documentation & API Contracts
* **Finding**: The repository lacked agent context directives (`agent.md`) and formal OpenAPI specifications (`web-ui/api.yml`).
* **Resolution**:
  - Authored [`agent.md`](agent.md) (and symlink `AGENT.MD`) modeled on the gold-standard `simply_ip_vault/AGENT.MD`.
  - Authored [`web-ui/api.yml`](web-ui/api.yml) specifying the complete OpenAPI 3.1.0 contract.
  - Added automated test in `frontend_syntax_test.rs` ensuring OpenAPI coverage.

#### 6. Ergonomics: Compact Header, Retractable Sidebar & High-Screen Terminal
* **Finding**: The top header and static sidebar occupied significant screen real estate, reducing the visible canvas for code, git diffs, and terminal execution.
* **Resolution**:
  - **Minimized Top Menu (`compact-header`)**: Added `#btn-toggle-compact-header` to collapse header padding, slim the tab bar, and reclaim vertical screen space, with persistence in `localStorage`.
  - **Retractable Left Sidebar (`sidebar-collapsed`)**: Added `#btn-toggle-sidebar` and floating `#btn-expand-sidebar-pill` allowing the left menu to retract completely, expanding the main workspace across 100% of the screen width.
  - **High-Screen & Maximized Native Terminal**: Increased default terminal drawer height to 60vh (min 440px) and added `#btn-term-maximize` allowing the terminal to occupy 98% of viewport width and height (`fixed` overlay with Escape key restoration).
  - Validated via automated Chromium CDP test scenarios, generating screenshots `chromium_10_compact_header.png`, `chromium_11_sidebar_retracted.png`, and `chromium_12_terminal_maximized.png`.

#### 7. Settings Gear Modal (⚙) & Technical UI Label Cleansing (Removal of "SeaORM")
* **Finding**: Raw language buttons (`EN`, `FR`, `DE`) cluttered the top header. In addition, the internal technical library name `(SeaORM)` had bled into the user interface (e.g. `Container Fleet (SeaORM)`), confusing end users who should see clean, domain-specific labels.
* **Resolution**:
  - **Settings Gear Modal (`#settings-modal`)**: Replaced raw header buttons with a discrete gear icon (`⚙`, `#btn-open-settings`) opening a KDE Plasma dark modal overlay. The modal houses the dynamic language selector (`#lang-selector`) and ergonomics checkboxes (compact header, retractable sidebar) with Escape key and backdrop dismissal.
  - **Technical UI Label Cleansing**: Completely removed all `(SeaORM)` suffixes from user-facing tabs, card titles, and subtitles across `index.html` and `app.js` (`I18N.en`, `I18N.fr`, `I18N.de`):
    - EN: `Container Fleet`, `Configurable Retention`, `Registered CLI Container Fleet`
    - FR: `Parc de Conteneurs`, `Rétention Paramétrable`, `Parc de Conteneurs CLI enregistrés`
    - DE: `Container-Flotte`, `Konfigurierbare Aufbewahrung`, `Registrierte CLI-Container-Flotte`
  - **Automated Verification**: Updated `frontend_syntax_test.rs` and `chromium_full_scale_test.rs` to open the modal via `#btn-open-settings`, assert that labels translate dynamically without any remaining "SeaORM" substrings, and captured screenshot artifact `chromium_13_settings_modal.png`.

#### 8. Database Persistence of Role Prompts & Language Packs with Startup Auto-Healing & English Fallback
* **Finding**: Default role prompts and UI dictionaries were originally embedded as hardcoded Rust match expressions in application code (`default_role_prompt_lang`, `default_lang_dict_*`). If the database was partially initialized or custom roles/languages were added dynamically, the system could not persist modifications or heal missing defaults without code recompilation.
* **Resolution**:
  - **SeaORM Entities & Migration**:
    - Created `web-ui/src/entities/role_prompt.rs` (table `role_prompts` with columns `id`, `role`, `lang`, `label`, `prompt`, `created_at`, `updated_at`, and a unique compound index on `(role, lang)`).
    - Created `web-ui/src/entities/language_pack.rs` (table `language_packs` with columns `id`, `lang`, `name`, `dictionary_json`, `created_at`, `updated_at`, and a unique index on `lang`).
    - Added migration `m20261106_000002_role_prompts_and_languages.rs` creating both tables idempotently.
  - **Startup Reconciliation & Self-Healing (`defaults.rs`)**:
    - Authored dedicated module [`web-ui/src/defaults.rs`](web-ui/src/defaults.rs) holding canonical role prompts and full JSON dictionaries for `en`, `fr`, `de`.
    - Implemented `ensure_database_defaults(&db)` called at application launch inside `db::connect_and_migrate`. It inspects all canonical definitions and automatically inserts any missing records without overwriting customized user data.
    - Implemented `get_or_create_role_prompt(&db, role, lang)` and `get_or_create_role_label(&db, role, lang)` providing runtime self-healing: if an entry is missing from DB, it automatically writes the canonical default to the database.
  - **REST API Endpoints**:
    - `GET /api/roles/presets` (queries `role_prompts` in DB, with automatic English fallback when an unknown language is requested).
    - `POST /api/roles/presets`, `PUT /api/roles/presets/{role}`, `DELETE /api/roles/presets/{role}` for runtime database updates.
    - `GET /api/languages`, `POST /api/languages`, `GET /api/languages/{lang}`, `PUT /api/languages/{lang}`, `DELETE /api/languages/{lang}` for database-backed dynamic language packs.
  - **Frontend UI & English Fallback**:
    - Added `#btn-save-role-template` ("💾 Save to DB" / "💾 Enregistrer en base" / "💾 In DB speichern") in the Agent Role subcard allowing users to persist customized system prompts directly to the database.
    - Added `loadLanguagePacksFromDb()` on startup to dynamically register all language packs present in the database.
    - Updated `t(key, ...args)` translation helper in `app.js` to strictly fall back to English (`I18N.en[key]`) whenever the active language has no translation defined for a given key (`undefined`, `null`, or `""`).
  - **Automated Verification**:
    - Added comprehensive unit test `test_role_prompts_and_language_packs_database_persistence_and_autohealing` verifying database pre-seeding, deletion recovery (self-healing), custom preset upsertion, and Spanish language pack persistence.
    - Enhanced `chromium_full_scale_test.rs` to click `#btn-save-role-template` and assert the updated prompt template in the SeaORM database.

#### 9. Foreign Key Constraints, Indexes on All Joins & SeaORM Relations
* **Finding**: While primary keys and unique keys were indexed, foreign keys (`channel_id`, `sender_instance_id`) and join columns (`role`, `lang`) lacked dedicated B-Tree indexes, and `sender_instance_id` lacked an explicit foreign key definition to `cli_instances`. Without indexes on both sides of each join, relational database engines (PostgreSQL, SQLite, MySQL) are forced to perform full table scans during joins and cascade operations.
* **Resolution**:
  - **Foreign Key Constraints & Relations**:
    - Added `fk_channel_messages_sender_instance` referencing `cli_instances(id)` with `on_delete: SetNull` and `on_update: Cascade`.
    - Declared bidirectional SeaORM relations between `channel_message` and `cli_instance` (`Relation::CliInstance` and `Relation::ChannelMessage`).
  - **Comprehensive Indexing on All Joins**:
    - `channel_messages.channel_id` (`idx_channel_messages_channel_id`): accelerates joins with `orchestration_channels`.
    - `channel_messages.sender_instance_id` (`idx_channel_messages_sender_instance_id`): accelerates joins with `cli_instances`.
    - `channel_messages.(channel_id, created_at)` (`idx_channel_messages_channel_created`): composite index optimizing channel message pagination and timeline ordering.
    - `cli_instances.role` (`idx_cli_instances_role`): indexes agent role lookups and joins with `role_prompts`.
    - `role_prompts.role` (`idx_role_prompts_role`): accelerates joins on role definitions.
    - `role_prompts.lang` (`idx_role_prompts_lang`): accelerates language-filtered lookups and joins with `language_packs`.
  - **Migration & Upgrade Path**:
    - Created migration `m20261106_000003_foreign_key_indexes.rs` with `.if_not_exists()` ensuring existing database files automatically apply all indexes on launch.
    - Updated initial schema migration `m20261105_000001_initial_schema.rs` and `m20261106_000002_role_prompts_and_languages.rs` for fresh database builds.
  - **Automated Verification**:
    - Added unit test `test_database_foreign_key_and_join_indexes` asserting the catalog contains all 7 indexes, testing SeaORM `.find_related()` navigation, verifying `CASCADE` message deletion on channel removal, and `SET NULL` on instance removal.

#### 10. Dedicated Master Table for Available Languages (`available_languages`)
* **Finding**: Languages were previously derived implicitly from the `language_packs` table containing large JSON dictionary strings. The system lacked a dedicated, lightweight master table defining the available/supported languages, default language flags (`is_default`), and active statuses without requiring full dictionary table scans.
* **Resolution**:
  - **SeaORM Entity (`available_language.rs`)**:
    - Created table `available_languages` with columns `id` (UUID PK), `code` (ISO 639-1 unique), `name`, `native_name`, `is_default` (boolean), `is_active` (boolean), `created_at`, `updated_at`.
    - Declared SeaORM relations to `language_pack` (`has_one`) and `role_prompt` (`has_many`).
  - **Migration (`m20261106_000004_available_languages.rs`)**:
    - Authored migration creating `available_languages` and B-Tree indexes: `idx_available_languages_code`, `idx_available_languages_is_default`, and `idx_available_languages_is_active`.
  - **Startup Reconciliation & Self-Healing (`defaults.rs`)**:
    - Defined `CANONICAL_AVAILABLE_LANGUAGES` pre-seeding `en` (default), `fr`, and `de`.
    - Implemented `ensure_available_languages_seeded(&db)` executed at startup in `ensure_database_defaults`.
    - Removed hardcoded `let packs = [...]` in `ensure_language_packs_seeded`, replacing it with dynamic iteration over `available_language::Entity` query from the database.
    - Added `canonical_lang_dict(code)` mapping canonical dictionaries.
    - Implemented `list_available_languages(&db)` and `get_or_create_available_language(&db, code, name, native_name)`.
  - **REST API Endpoints**:
    - Added `GET /api/languages/available` returning lightweight language records.
    - Added `POST /api/languages/available` to register new available languages.
    - Updated `GET /api/languages` to query the `available_languages` master table.
    - Synchronized `POST /api/languages` (pack upsertion) to automatically register the language in `available_languages`.
  - **OpenAPI & Automated Verification**:
    - Documented `/api/languages/available` and `AvailableLanguage` schemas in [`web-ui/api.yml`](web-ui/api.yml).
    - Added test assertions in `test_role_prompts_and_language_packs_database_persistence_and_autohealing` and `test_database_foreign_key_and_join_indexes`.

#### 11. Root Cargo Workspace Configuration & IDE Language Server (rust-analyzer) Integration
* **Finding**: The Git repository root (`/home/fallrik/Documents/workspaces/new_local_user`) lacked a top-level `Cargo.toml`. When opened in an IDE or editor (VS Code, Antigravity IDE, RustRover), the default `rust-analyzer` language server attempted to locate `Cargo.toml` at the workspace root and failed (`error: could not find 'Cargo.toml' in workspace root`). This caused editors to analyze Rust files as detached standalone files, generating spurious syntax and unresolved import warnings (e.g. `crate::entities`).
* **Resolution**:
  - Created root virtual workspace manifest [`Cargo.toml`](Cargo.toml):
    ```toml
    [workspace]
    resolver = "2"
    members = [
        "web-ui",
    ]
    ```
  - Executed full workspace formatting sweep with `cargo fmt`.
  - Verified that `cargo check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` execute directly from the repository root with zero errors.

#### 12. Externalization of Multilingual Defaults to `default_translate.json`, TTL Server Cache & Browser Cache
* **Finding**: Hardcoding canonical role prompts, available languages, and large JSON dictionaries inside Rust variables bloated the binary and necessitated recompilations. Storing defaults in `.js` required fragile parsing, whereas standard JSON provides direct deserialization. High-frequency queries required query-specific indexing and a thread-safe cache with a time-to-live (TTL) on-demand population. Furthermore, redundant HTTP requests on client view changes warranted a browser-side cache (`localStorage`) invalidated only on language switch or explicit modifications.
* **Resolution**:
  - **Canonical JSON Seed File (`web-ui/static/default_translate.json`)**:
    - Created pure JSON seed file `default_translate.json` containing `available_languages` (`en`, `fr`, `de`), canonical `role_prompts` (6 roles x 3 languages), and complete UI translation dictionaries (`language_packs`).
    - Removed obsolete `defaults.js` and replaced `/defaults.js` route with `/default_translate.json`.
  - **Zero Embedded Language Constants in Rust Code (`defaults.rs`)**:
    - Purged all hardcoded multilingual constants from Rust code. The codebase only retains lookup keys: `DEFAULT_LANG = "en"` and `DEFAULT_ROLE = "developer"`.
    - Implemented `load_defaults_from_json()` with compile-time embedded fallback `include_str!("../static/default_translate.json")` strictly for database self-healing when tables or rows are missing.
  - **B-Tree Join & Query Indexes (`migration/m20261106_000004_available_languages.rs`)**:
    - Added dedicated indexes for high-frequency queries:
      - `idx_available_languages_active_code` on `available_languages(is_active, code)`
      - `idx_role_prompts_lang_role` on `role_prompts(lang, role)`
      - `idx_language_packs_name` on `language_packs(name)`
      - Plus existing `idx_role_prompts_role_lang`, `idx_language_packs_lang`, `idx_available_languages_code`, `idx_available_languages_is_default`, `idx_available_languages_is_active`.
  - **On-Demand Server-Side Cache with Monotonic TTL (`cache.rs`)**:
    - Implemented `LanguageCache` where items are populated on-demand ("au fur et à mesure des appels") with monotonic time-to-live (`expires_at: Instant`).
    - Configured default TTL to 5 minutes (`DEFAULT_CACHE_TTL = 300s`).
    - Integrated with `list_available_languages_cached`, `get_language_pack`, and `list_role_presets`.
    - Automated cache invalidation on any write or delete operation (`POST /api/languages`, `POST /api/languages/available`, `POST/PUT/DELETE /api/roles/presets`, prompt edits).
  - **Browser-Side Client Cache (`app.js`)**:
    - Stored language dictionaries and role presets in `localStorage` (`agent_hub_dict_${lang}`, `agent_hub_role_presets_${lang}`).
    - Avoids repetitive network requests when re-rendering fields or navigating views.
    - Cache is invalidated only when the user switches language (`setAppLanguage`) or explicitly saves a new role template (`#btn-save-role-template`).
  - **Automated Verification**:
    - Added `default_translate_json_is_valid_json` in `frontend_syntax_test.rs`.
    - Tested index existence and cache invalidation in `main.rs`.
    - Verified all 14 tests passing across workspace.

#### 13. Multi-CLI Tmux Session Isolation (`agy` & `codex`) & Chromium CDP E2E Live Terminal Verification
* **Finding**: `agent_bridge` previously hardcoded the tmux session name to `"the_new_user"`, preventing multiple CLI agents (such as Antigravity `agy` and OpenAI `codex`) from running in distinct dedicated tmux sessions simultaneously. Furthermore, rapid switching between terminal instances in `app.js` could cause stale `onclose` events from previous sockets to overwrite the status of newly opened WebSockets.
* **Resolution**:
  - **Configurable `TMUX_SESSION` in `agent_bridge` (`src/bin/agent_bridge.rs`)**:
    - Added `tmux_session` to `BridgeState`, configured via `env::var("TMUX_SESSION").or_else(|_| env::var("AGENT_TMUX_SESSION"))` (falling back to `"the_new_user"`).
    - `initialize` returns `"tmuxSession": state.tmux_session`.
    - `tmux/capture`, `tmux/send_keys`, `tmux/send_raw`, and `run_cli_prompt` target the configured session by default.
  - **Configurable Codex Bridge URL in Hub (`db.rs` & `main.rs`)**:
    - Supported `CODEX_RPC_URL` / `CODEX_BRIDGE_URL` in `connect_and_migrate` to allow pointing Codex directly to local bridge instances.
  - **Terminal WebSocket Race Condition Guard in Frontend (`app.js`)**:
    - Detached `onclose`, `onerror`, and `onmessage` handlers before closing prior sockets in `connectNativeTerminal()` and `closeModal()`.
    - Guarded status updates with `if (state.terminalWs === ws)` to guarantee socket isolation during instance switching.
  - **Automated Chromium CDP Verification (`tests/chromium_tmux_cli_test.rs`)**:
    - Created dedicated end-to-end integration test creating live `tmux` sessions (`agy_session` with `agy --version`, `codex_session` with `codex ready`).
    - Spawned bridges on ports `13296` (`antigravity`) and `13297` (`codex`).
    - Launched headless Chromium over CDP, connected to both instances, sent interactive terminal commands via PTY, asserted live execution feedback, and captured screenshots:
      - `chromium_14_agy_tmux_connected.png`
      - `chromium_15_codex_tmux_connected.png`

#### 14. Responsive Top Navigation (No Scrollbars), Default Condensed Header & Full Workspace Terminal Active Mode
* **Finding**: In Linux/Chromium environments with GTK, CSS `overflow-x: auto` on `.tab-nav` triggered sub-pixel vertical overflow, rendering an unsightly vertical scrollbar with arrow buttons (▲/▼) that broke responsiveness. In addition, the header previously defaulted to expanded mode, consuming valuable vertical screen real estate. When launching or attaching a terminal drawer inside `.chat-card`, the terminal was squeezed above the chat message stream and prompt input bar instead of occupying the full available height of the workspace.
* **Resolution**:
  - **Responsive Tab Navigation Without Scrollbars (`web-ui/static/style.css`)**:
    - Changed `.tab-nav` to `flex-wrap: wrap; overflow: visible; gap: 0.35rem;`.
    - Added `::-webkit-scrollbar { display: none; }` and `scrollbar-width: none;` on tab containers.
    - Buttons now flow and wrap responsively across different viewport widths without ever creating horizontal or vertical scrollbars.
  - **Condensed Header Active by Default (`web-ui/static/index.html` & `app.js`)**:
    - Initialized `<body class="compact-header">` in HTML.
    - Defaulted `compact-header` to active on application startup unless explicitly disabled (`localStorage.getItem("agent_hub_compact_header") !== "false"`).
    - Reduced header padding, shrunk title and logo icon size, saving ~80px of vertical height across all views (including CLI console).
  - **Terminal Active Mode Maximizing Workspace (`web-ui/static/style.css` & `app.js`)**:
    - Implemented `.chat-card.terminal-active` state class toggled whenever the native terminal is connected or displayed.
    - When `.terminal-active` is engaged, `.messages-stream`, `.composer-bar`, and `#console-approval-banner` are cleanly hidden via `display: none !important;`.
    - Expanded `.native-terminal-drawer` to occupy 100% of the available card height (`calc(100vh - 145px)`) with the terminal viewport stretching to `calc(100vh - 215px)`.
    - Enhanced `.native-terminal-drawer.maximized` to true edge-to-edge full screen (`position: fixed; inset: 0; width: 100vw; height: 100vh; margin: 0; border: none; border-radius: 0; z-index: 1000;`).
  - **Automated Verification**:
    - Updated both `chromium_full_scale_test.rs` and `chromium_tmux_cli_test.rs`.
    - Captured and visually verified screenshots:
      - `chromium_14_agy_tmux_connected.png` (Antigravity CLI attached with responsive menu & full-height terminal)
      - `chromium_15_codex_tmux_connected.png` (Codex CLI attached in full-height terminal)
      - `chromium_16_terminal_fullscreen_maximized.png` (True 100vw x 100vh edge-to-edge fullscreen terminal)
    - Full test suite passed (15/15 tests).

#### 15. Unified Single-Line Header Layout & "Simply Agent Hub" Home Navigation
* **Finding**: The dashboard previously stacked the brand logo / actions on one line and the navigation tab bar on a separate second line, taking up redundant vertical space. Furthermore, the brand title was static text and did not provide an intuitive way to return to the home console from other tabs or views.
* **Resolution**:
  - **Unified Single Horizontal Line Header (`web-ui/static/index.html` & `style.css`)**:
    - Embedded `<nav class="tab-nav">` directly inside `<header class="header">`.
    - Arranged `.header` as a single horizontal flex row (`display: flex; align-items: center; gap: 0.85rem; flex-wrap: nowrap;`).
    - Aligned `#brand-home` and `.tab-nav` on the left (`flex-shrink: 0;`), with `.header-actions` pushed to the far right (`margin-left: auto;`).
    - Streamlined tab labels across language packs (EN: "CLI Console", "Orchestration", "Git Diff", "Container Fleet", "JSON-RPC") and added `max-width: 175px` ellipsis truncation on active instance badges to prevent any flexbox overflow.
    - Reduced overall top-bar height to ~42px, allowing `.chat-card` and native terminal viewport to gain maximum vertical space.
  - **"Simply Agent Hub" Brand as Home Button (`index.html` & `app.js`)**:
    - Wrapped `.logo` into an accessible interactive anchor/button `#brand-home` (`role="button"`, `title="Simply Agent Hub — Accueil"`).
    - Clicking `#brand-home` restores any maximized terminal, resets window scroll to `(0, 0)`, and immediately activates `window.switchTab('console')`.
  - **Automated Verification**:
    - Added automated test step in `chromium_tmux_cli_test.rs`: switches away to `orchestration` tab, clicks `#brand-home`, asserts `console` tab is restored to active, and asserts `brand-home` and `.tab-nav` are horizontally aligned within $\pm 10$px on the same row.
    - Captured and visually verified screenshot:
      - `chromium_17_single_line_header_home.png`
    - All 15 tests passing across entire workspace.

---

### Verification Summary

| Test Suite | Command | Outcome |
| :--- | :--- | :--- |
| Workspace Cargo Check | `cargo check --all-targets` | **0 errors, 0 warnings** |
| Clippy Gate | `cargo clippy --all-targets -- -D warnings` | **0 warnings, 0 errors** |
| Code Formatting | `cargo fmt --check` | **0 differences** |
| Unit Tests (`agent-bridge`) | `cargo test --bin agent-bridge` | **1 passed, 0 failed** |
| Unit Tests (`agent-hub`) | `cargo test --bin agent-hub` | **8 passed, 0 failed** |
| Frontend Syntax & OpenAPI | `cargo test --test frontend_syntax_test` | **4 passed, 0 failed** |
| Chromium CDP E2E (Full Scale) | `cargo test --test chromium_full_scale_test` | **1 passed, 0 failed** |
| Chromium CDP E2E (Tmux CLIs) | `cargo test --test chromium_tmux_cli_test` | **1 passed, 0 failed** |
| **Total Test Suite** | `cargo test` | **15 passed, 0 failed** |
