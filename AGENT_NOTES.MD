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

---

### Verification Summary

| Test Suite | Command | Outcome |
| :--- | :--- | :--- |
| Clippy Gate | `cargo clippy --all-targets -- -D warnings` | **0 warnings, 0 errors** |
| Unit Tests (`agent-bridge`) | `cargo test --bin agent-bridge` | **1 passed, 0 failed** |
| Unit Tests (`agent-hub`) | `cargo test --bin agent-hub` | **6 passed, 0 failed** |
| Frontend Syntax & OpenAPI | `cargo test --test frontend_syntax_test` | **3 passed, 0 failed** |
| Chromium CDP E2E | `cargo test --test chromium_full_scale_test` | **1 passed, 0 failed** |
| **Total Test Suite** | `cargo test` | **11 passed, 0 failed** |
