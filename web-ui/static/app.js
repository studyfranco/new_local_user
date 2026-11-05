(() => {
    const state = {
        instances: [],
        activeInstanceId: null,
        rolePresets: {},
        rpcSeq: 1,
        pendingRequests: new Map(),
        pendingApproval: null,

        // Active console connection state
        ws: null,
        connected: false,
        activeSessionId: null,
        sessionsByInstance: {},
        streamingEl: null,
        turnRunning: false,

        // Native interactive terminal WS
        terminalWs: null,

        // Orchestration channels state
        channels: [],
        activeChannelId: null,
        channelWs: null,
        seenMessageIds: new Set(),
    };

    const $ = (id) => document.getElementById(id);

    function nextId() {
        return state.rpcSeq++;
    }

    function wsBaseUrl() {
        const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
        return `${proto}//${window.location.host}`;
    }

    function escapeHtml(str) {
        return String(str ?? "")
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
    }

    // Lightweight ANSI SGR color parser for native terminal viewport
    function ansiToHtml(raw) {
        const safe = escapeHtml(raw);
        const colorMap = {
            "30": "#555",
            "31": "#ec7063",
            "32": "#58d68d",
            "33": "#f5b041",
            "34": "#5dade2",
            "35": "#af7ac5",
            "36": "#1cdc9a",
            "37": "#eff0f1",
            "90": "#7f8c8d",
            "91": "#ff6b6b",
            "92": "#2ecc71",
            "93": "#f1c40f",
            "94": "#3daee9",
            "95": "#d2b4de",
            "96": "#48c9b0",
            "97": "#ffffff",
        };
        let openSpans = 0;
        const formatted = safe.replace(/\x1b\[([0-9;]*)m/g, (_, codesStr) => {
            const codes = codesStr.split(";");
            if (codesStr === "" || codes.includes("0") || codes.includes("39")) {
                let closing = "";
                while (openSpans > 0) {
                    closing += "</span>";
                    openSpans--;
                }
                return closing;
            }
            for (const c of codes) {
                if (colorMap[c]) {
                    openSpans++;
                    return `<span style="color:${colorMap[c]}">`;
                }
                if (c === "1") {
                    openSpans++;
                    return `<span style="font-weight:700">`;
                }
            }
            return "";
        });
        let tail = "";
        while (openSpans > 0) {
            tail += "</span>";
            openSpans--;
        }
        return formatted.replace(/\x1b\[[0-9;?]*[A-Za-z]/g, "") + tail;
    }

    function logRpc(direction, label, payload) {
        const log = $("rpc-log");
        if (!log) return;
        const div = document.createElement("div");
        div.className = `msg ${direction === "out" ? "msg-user" : "msg-tool"}`;
        div.style.maxWidth = "100%";
        div.style.padding = "0.45rem 0.75rem";
        const raw = typeof payload === "string" ? payload : JSON.stringify(payload);
        div.textContent = `[${new Date().toLocaleTimeString()}] [${label}] ${direction === "out" ? "→ OUT" : "← IN "} ${raw}`;
        log.appendChild(div);
        log.scrollTop = log.scrollHeight;
    }

    function appendMsg(containerId, role, sender, text) {
        const container = $(containerId);
        if (!container) return null;

        const wrapper = document.createElement("div");
        wrapper.className = `msg msg-${role}`;

        const header = document.createElement("div");
        header.className = "msg-header";
        header.innerHTML = `<span>${escapeHtml(sender)}</span><span>${new Date().toLocaleTimeString()}</span>`;

        const body = document.createElement("div");
        body.className = "msg-body";
        body.textContent = text;

        wrapper.appendChild(header);
        wrapper.appendChild(body);
        container.appendChild(wrapper);
        container.scrollTop = container.scrollHeight;
        return body;
    }

    function getActiveInstance() {
        return state.instances.find((i) => i.id === state.activeInstanceId) || null;
    }

    // ─── Tab Navigation ──────────────────────────────────────────────────────
    document.querySelectorAll(".tab-btn").forEach((btn) => {
        btn.addEventListener("click", () => {
            const target = btn.getAttribute("data-tab");
            document.querySelectorAll(".tab-btn").forEach((b) => {
                b.classList.toggle("active", b === btn);
                b.setAttribute("aria-selected", b === btn ? "true" : "false");
            });
            document.querySelectorAll(".tab-panel").forEach((panel) => {
                panel.classList.toggle("active", panel.id === `tab-${target}`);
            });
            if (target === "git") {
                loadGitStatus();
            }
        });
    });

    // ═════════════════════════════════════════════════════════════════════════
    // 1. CLI INSTANCES, ROLES & SEAORM CRUD
    // ═════════════════════════════════════════════════════════════════════════
    async function loadRolePresets() {
        try {
            const resp = await fetch("/api/roles/presets");
            const list = await resp.json();
            if (Array.isArray(list)) {
                list.forEach((r) => {
                    state.rolePresets[r.id] = r;
                });
            }
        } catch {
            // ignore
        }
    }

    async function loadInstances() {
        try {
            const resp = await fetch("/api/instances");
            const list = await resp.json();
            state.instances = Array.isArray(list) ? list : [];

            if (!state.activeInstanceId && state.instances.length > 0) {
                state.activeInstanceId = state.instances[0].id;
            } else if (
                state.activeInstanceId &&
                !state.instances.some((i) => i.id === state.activeInstanceId)
            ) {
                state.activeInstanceId = state.instances[0]?.id || null;
            }

            renderCliSelector();
            renderRolePanel();
            renderInventoryTable();
            renderChannelParticipants();
            updateConsoleBadges();
        } catch (err) {
            appendMsg("console-messages", "system", "Erreur", `Impossible de charger les instances: ${err.message}`);
        }
    }

    function renderCliSelector() {
        const sel = $("cli-selector");
        sel.innerHTML = "";
        const onlineCount = state.instances.filter((i) => i.online).length;
        $("header-instances-count").textContent = `${state.instances.length} CLIs (${onlineCount} en ligne)`;

        state.instances.forEach((inst) => {
            const opt = document.createElement("option");
            opt.value = inst.id;
            const dot = inst.online ? "🟢" : "🔴";
            opt.textContent = `${dot} [${inst.kind.toUpperCase()}] ${inst.name} (${inst.role || "developer"})`;
            if (inst.id === state.activeInstanceId) opt.selected = true;
            sel.appendChild(opt);
        });

        const active = getActiveInstance();
        const headerBadge = $("header-active-cli-badge");
        if (active) {
            headerBadge.textContent = `${active.name} (${active.role || active.kind})`;
            headerBadge.className = `badge ${active.online ? "badge-online" : "badge-offline"}`;
        } else {
            headerBadge.textContent = "Aucun CLI";
            headerBadge.className = "badge";
        }
    }

    function renderRolePanel() {
        const inst = getActiveInstance();
        if (!inst) return;
        const roleSel = $("cli-role-selector");
        const promptBox = $("cli-role-prompt");
        if (roleSel) roleSel.value = inst.role || "developer";
        if (promptBox) {
            promptBox.value =
                inst.role_prompt ||
                state.rolePresets[inst.role || "developer"]?.prompt ||
                "";
        }
    }

    $("cli-role-selector").addEventListener("change", (e) => {
        const selectedRole = e.target.value;
        const preset = state.rolePresets[selectedRole];
        if (preset && preset.prompt) {
            $("cli-role-prompt").value = preset.prompt;
        }
    });

    $("btn-save-role").addEventListener("click", async () => {
        const inst = getActiveInstance();
        if (!inst) return;
        const role = $("cli-role-selector").value;
        const role_prompt = $("cli-role-prompt").value.trim();
        const resp = await fetch(`/api/instances/${inst.id}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ role, role_prompt }),
        });
        const updated = await resp.json();
        if (updated.id) {
            await loadInstances();
            appendMsg(
                "console-messages",
                "system",
                inst.name,
                `Rôle « ${role} » et prompt système enregistrés en base SeaORM.`
            );
        }
    });

    $("btn-onboard-cli").addEventListener("click", async () => {
        const inst = getActiveInstance();
        if (!inst) return;
        const role = $("cli-role-selector").value;
        const role_prompt = $("cli-role-prompt").value.trim();

        // Save role & prompt first
        await fetch(`/api/instances/${inst.id}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ role, role_prompt }),
        });
        await loadInstances();

        appendMsg(
            "console-messages",
            "user",
            "Initialisation Rôle",
            `Envoi du prompt d'identité et de rôle (${role}) à ${inst.name}...`
        );
        try {
            const resp = await fetch(`/api/instances/${inst.id}/onboard`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ custom_prompt: role_prompt }),
            });
            const data = await resp.json();
            logRpc("in", `onboard:${inst.name}`, data);
            if (data.error) throw new Error(data.error);
            appendMsg("console-messages", "agent", `${inst.name} [${data.role}]`, data.reply || "Rôle initialisé.");
        } catch (err) {
            appendMsg("console-messages", "system", inst.name, `Erreur d'initialisation du rôle: ${err.message}`);
        }
    });

    function renderInventoryTable() {
        const tbody = $("inventory-tbody");
        if (!tbody) return;
        tbody.innerHTML = "";
        state.instances.forEach((inst) => {
            const tr = document.createElement("tr");
            tr.innerHTML = `
                <td><strong>${escapeHtml(inst.name)}</strong></td>
                <td><span class="badge badge-tier">${escapeHtml(inst.kind.toUpperCase())}</span></td>
                <td><span class="badge badge-online">${escapeHtml(inst.role || "developer")}</span></td>
                <td><code>${escapeHtml(inst.transport)}</code></td>
                <td><code>${escapeHtml(inst.endpoint_url)}</code></td>
                <td><code>${escapeHtml(inst.default_cwd)}</code></td>
                <td>
                    <span class="badge ${inst.online ? "badge-online" : "badge-offline"}" title="${escapeHtml(inst.status_detail)}">
                        ${inst.online ? "En ligne" : "Hors ligne"}
                    </span>
                </td>
                <td>
                    <div class="btn-row" style="margin-top:0;">
                        <button class="btn btn-secondary btn-sm" data-action="select" data-id="${inst.id}">Utiliser</button>
                        <button class="btn btn-secondary btn-sm" data-action="edit" data-id="${inst.id}">✎ Renommer</button>
                        <button class="btn btn-danger btn-sm" data-action="delete" data-id="${inst.id}">🗑</button>
                    </div>
                </td>
            `;
            tbody.appendChild(tr);
        });

        tbody.querySelectorAll("button[data-action]").forEach((b) => {
            b.addEventListener("click", async () => {
                const id = b.getAttribute("data-id");
                const act = b.getAttribute("data-action");
                state.activeInstanceId = id;
                renderCliSelector();
                renderRolePanel();
                updateConsoleBadges();
                if (act === "select") {
                    document.querySelector('.tab-btn[data-tab="console"]').click();
                } else if (act === "edit") {
                    document.querySelector('.tab-btn[data-tab="console"]').click();
                    openEditCliForm();
                } else if (act === "delete") {
                    await deleteActiveCli();
                }
            });
        });
    }

    $("cli-selector").addEventListener("change", (e) => {
        state.activeInstanceId = e.target.value;
        if (state.ws) {
            state.ws.close();
            state.ws = null;
        }
        if (state.terminalWs) {
            state.terminalWs.close();
            state.terminalWs = null;
        }
        state.connected = false;
        state.activeSessionId = null;
        renderCliSelector();
        renderRolePanel();
        updateConsoleBadges();
        renderSessionsList();
        if (!$("native-terminal-drawer").classList.contains("hidden")) {
            connectNativeTerminal();
        }
    });

    function openAddCliForm() {
        $("cli-form-title").textContent = "Ajouter un nouveau conteneur CLI";
        $("cli-form-id").value = "";
        $("cli-form-name").value = "";
        $("cli-form-kind").value = "codex";
        $("cli-form-role").value = "developer";
        $("cli-form-transport").value = "ws";
        $("cli-form-endpoint").value = "ws://codex:4500";
        $("cli-form-bridge").value = "http://codex:3287/rpc";
        $("cli-form-secret").value = "";
        $("cli-form-cwd").value = "/srv";
        $("cli-form-box").classList.remove("hidden");
        $("cli-auth-box").classList.add("hidden");
    }

    function openEditCliForm() {
        const inst = getActiveInstance();
        if (!inst) return;
        $("cli-form-title").textContent = `Renommer / Modifier « ${inst.name} »`;
        $("cli-form-id").value = inst.id;
        $("cli-form-name").value = inst.name;
        $("cli-form-kind").value = inst.kind;
        $("cli-form-role").value = inst.role || "developer";
        $("cli-form-transport").value = inst.transport;
        $("cli-form-endpoint").value = inst.endpoint_url;
        $("cli-form-bridge").value = inst.bridge_url || "";
        $("cli-form-secret").value = "";
        $("cli-form-cwd").value = inst.default_cwd || "/srv";
        $("cli-form-box").classList.remove("hidden");
        $("cli-auth-box").classList.add("hidden");
    }

    $("cli-form-kind").addEventListener("change", () => {
        const kind = $("cli-form-kind").value;
        if (!$("cli-form-id").value) {
            if (kind === "codex") {
                $("cli-form-transport").value = "ws";
                $("cli-form-endpoint").value = "ws://codex:4500";
                $("cli-form-bridge").value = "http://codex:3287/rpc";
            } else if (kind === "goose") {
                $("cli-form-transport").value = "ws";
                $("cli-form-endpoint").value = "ws://goose:3284/acp";
                $("cli-form-bridge").value = "http://goose:3288/rpc";
            } else if (kind === "antigravity") {
                $("cli-form-transport").value = "rpc";
                $("cli-form-endpoint").value = "http://antigravity:3286/rpc";
                $("cli-form-bridge").value = "http://antigravity:3286/rpc";
            } else if (kind === "claude") {
                $("cli-form-transport").value = "rpc";
                $("cli-form-endpoint").value = "http://claude:3285/rpc";
                $("cli-form-bridge").value = "http://claude:3285/rpc";
            }
        }
    });

    $("btn-open-add-cli").addEventListener("click", openAddCliForm);
    $("btn-inventory-add").addEventListener("click", () => {
        document.querySelector('.tab-btn[data-tab="console"]').click();
        openAddCliForm();
    });
    $("btn-open-rename-cli").addEventListener("click", openEditCliForm);
    $("btn-cancel-cli").addEventListener("click", () => $("cli-form-box").classList.add("hidden"));

    $("btn-save-cli").addEventListener("click", async () => {
        const id = $("cli-form-id").value.trim();
        const payload = {
            name: $("cli-form-name").value.trim() || "Nouveau CLI",
            kind: $("cli-form-kind").value,
            role: $("cli-form-role").value,
            transport: $("cli-form-transport").value,
            endpoint_url: $("cli-form-endpoint").value.trim(),
            bridge_url: $("cli-form-bridge").value.trim() || null,
            secret_token: $("cli-form-secret").value.trim() || null,
            default_cwd: $("cli-form-cwd").value.trim() || "/srv",
        };

        const url = id ? `/api/instances/${id}` : "/api/instances";
        const method = id ? "PUT" : "POST";
        const resp = await fetch(url, {
            method,
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(payload),
        });
        const saved = await resp.json();
        if (saved.id) {
            state.activeInstanceId = saved.id;
            $("cli-form-box").classList.add("hidden");
            await loadInstances();
            appendMsg("console-messages", "system", "Base SeaORM", `Instance « ${saved.name} » (${saved.role}) enregistrée.`);
        }
    });

    async function deleteActiveCli() {
        const inst = getActiveInstance();
        if (!inst) return;
        await fetch(`/api/instances/${inst.id}`, { method: "DELETE" });
        state.activeInstanceId = null;
        await loadInstances();
    }

    $("btn-delete-cli").addEventListener("click", deleteActiveCli);

    // ═════════════════════════════════════════════════════════════════════════
    // 2. WEB UI AUTHENTICATION FOR ANY CONTAINER (/api/instances/{id}/auth)
    // ═════════════════════════════════════════════════════════════════════════
    $("btn-open-auth-cli").addEventListener("click", () => {
        $("cli-auth-box").classList.toggle("hidden");
        $("cli-form-box").classList.add("hidden");
    });

    async function runInstanceAuth(action, value = null) {
        const inst = getActiveInstance();
        if (!inst) return;
        const box = $("auth-result-box");
        box.classList.remove("hidden");
        box.textContent = `Exécution (${action}) sur ${inst.name}...`;

        try {
            const resp = await fetch(`/api/instances/${inst.id}/auth`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ action, value }),
            });
            const data = await resp.json();
            logRpc("in", `auth:${inst.name}`, data);
            const auth = data.auth || {};
            let html = `<strong>Instance :</strong> ${escapeHtml(inst.name)}\n`;
            if (auth.authenticated !== undefined) {
                html += `<strong>Authentifié :</strong> ${auth.authenticated ? "OUI ✓" : "NON (Authentification requise)"}\n`;
            }
            if (auth.url) {
                html += `<strong>Lien d'authentification :</strong> <a href="${escapeHtml(auth.url)}" target="_blank" rel="noopener">${escapeHtml(auth.url)}</a>\n`;
            }
            if (auth.code) {
                html += `<strong>Code à usage unique :</strong> <span style="color:var(--accent);font-size:1.05rem;font-weight:700;">${escapeHtml(auth.code)}</span>\n`;
            }
            if (auth.detail || auth.raw) {
                html += `\n${escapeHtml(auth.detail || auth.raw)}`;
            }
            box.innerHTML = html;
            await loadInstances();
        } catch (err) {
            box.textContent = `Erreur auth: ${err.message}`;
        }
    }

    $("btn-auth-status").addEventListener("click", () => runInstanceAuth("status"));
    $("btn-auth-device").addEventListener("click", () => runInstanceAuth("device_start"));
    $("btn-auth-apikey").addEventListener("click", () => {
        const key = $("auth-apikey-input").value.trim();
        if (key) runInstanceAuth("api_key", key);
    });
    $("btn-auth-callback").addEventListener("click", () => {
        const cb = $("auth-callback-input").value.trim();
        if (cb) runInstanceAuth("oauth_callback", cb);
    });

    // ═════════════════════════════════════════════════════════════════════════
    // 3. NATIVE INTERACTIVE TERMINAL (WebSocket /ws/instances/{id}/terminal)
    // ═════════════════════════════════════════════════════════════════════════
    function connectNativeTerminal() {
        const inst = getActiveInstance();
        if (!inst) return;

        if (state.terminalWs) {
            state.terminalWs.close();
            state.terminalWs = null;
        }

        const drawer = $("native-terminal-drawer");
        drawer.classList.remove("hidden");
        const viewport = $("terminal-viewport");
        const statusEl = $("terminal-ws-status");
        statusEl.textContent = `Connexion à ${inst.name}...`;
        statusEl.className = "badge";

        const ws = new WebSocket(`${wsBaseUrl()}/ws/instances/${encodeURIComponent(inst.id)}/terminal`);
        state.terminalWs = ws;

        ws.onopen = () => {
            statusEl.textContent = `Connecté (${inst.name} — Tmux PTY)`;
            statusEl.className = "badge badge-online";
            viewport.focus();
        };

        ws.onmessage = (ev) => {
            try {
                const msg = JSON.parse(ev.data);
                if (msg.type === "screen") {
                    viewport.innerHTML = ansiToHtml(msg.data || "(Session tmux prête)");
                    viewport.scrollTop = viewport.scrollHeight;
                }
            } catch {
                // ignore
            }
        };

        ws.onclose = () => {
            statusEl.textContent = "Déconnecté";
            statusEl.className = "badge badge-offline";
        };
    }

    $("btn-cli-tmux").addEventListener("click", () => {
        const drawer = $("native-terminal-drawer");
        if (drawer.classList.contains("hidden")) {
            connectNativeTerminal();
        } else {
            drawer.classList.add("hidden");
            if (state.terminalWs) {
                state.terminalWs.close();
                state.terminalWs = null;
            }
        }
    });

    $("btn-term-close").addEventListener("click", () => {
        $("native-terminal-drawer").classList.add("hidden");
        if (state.terminalWs) {
            state.terminalWs.close();
            state.terminalWs = null;
        }
    });

    $("btn-term-refresh").addEventListener("click", () => {
        if (state.terminalWs?.readyState === WebSocket.OPEN) {
            state.terminalWs.send(JSON.stringify({ type: "refresh" }));
        } else {
            connectNativeTerminal();
        }
    });

    $("btn-term-ctrlc").addEventListener("click", () => {
        if (state.terminalWs?.readyState === WebSocket.OPEN) {
            state.terminalWs.send(JSON.stringify({ type: "raw", data: "C-c", special: true }));
        }
    });

    $("btn-term-enter").addEventListener("click", () => {
        if (state.terminalWs?.readyState === WebSocket.OPEN) {
            state.terminalWs.send(JSON.stringify({ type: "raw", data: "Enter", special: true }));
        }
    });

    $("terminal-input-form").addEventListener("submit", (e) => {
        e.preventDefault();
        const input = $("terminal-cmd-input");
        const text = input.value;
        input.value = "";
        if (state.terminalWs?.readyState === WebSocket.OPEN) {
            state.terminalWs.send(JSON.stringify({ type: "line", text }));
        }
    });

    $("terminal-viewport").addEventListener("keydown", (e) => {
        if (!state.terminalWs || state.terminalWs.readyState !== WebSocket.OPEN) return;
        if (e.ctrlKey && e.key.toLowerCase() === "c") {
            e.preventDefault();
            state.terminalWs.send(JSON.stringify({ type: "raw", data: "C-c", special: true }));
        } else if (e.key === "Enter") {
            e.preventDefault();
            state.terminalWs.send(JSON.stringify({ type: "raw", data: "Enter", special: true }));
        } else if (e.key === "Backspace") {
            e.preventDefault();
            state.terminalWs.send(JSON.stringify({ type: "raw", data: "BSpace", special: true }));
        } else if (e.key === "ArrowUp") {
            e.preventDefault();
            state.terminalWs.send(JSON.stringify({ type: "raw", data: "Up", special: true }));
        } else if (e.key === "ArrowDown") {
            e.preventDefault();
            state.terminalWs.send(JSON.stringify({ type: "raw", data: "Down", special: true }));
        } else if (e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
            e.preventDefault();
            state.terminalWs.send(JSON.stringify({ type: "raw", data: e.key, special: false }));
        }
    });

    // ═════════════════════════════════════════════════════════════════════════
    // 4. UNIFIED ACTIVE CLI CONSOLE (Codex WS/Unix, Goose ACP, Claude/Agy RPC)
    // ═════════════════════════════════════════════════════════════════════════
    function updateConsoleBadges() {
        const inst = getActiveInstance();
        $("console-conn-badge").textContent = state.connected ? "Flux Connecté" : "Prêt";
        $("console-conn-badge").className = `badge ${state.connected ? "badge-online" : ""}`;
        $("console-instance-badge").textContent = inst ? `${inst.name} [${inst.kind.toUpperCase()}]` : "Aucun CLI";
        $("console-role-badge").textContent = inst ? `Rôle: ${inst.role || "developer"}` : "-";
        $("console-session-badge").textContent = state.activeSessionId
            ? `Session/Thread : ${state.activeSessionId}`
            : "Session auto";
        $("btn-console-interrupt").classList.toggle("hidden", !state.turnRunning);
    }

    function renderSessionsList() {
        const listEl = $("cli-sessions-list");
        listEl.innerHTML = "";
        const inst = getActiveInstance();
        if (!inst) return;
        const sessions = state.sessionsByInstance[inst.id] || [];
        if (!sessions.length) {
            listEl.innerHTML = `<div class="field-hint">Aucune session enregistrée pour ${escapeHtml(inst.name)}</div>`;
            return;
        }
        sessions.forEach((s) => {
            const id = s.id || s.threadId || s.sessionId || String(s);
            const div = document.createElement("div");
            div.className = `session-item ${state.activeSessionId === id ? "active" : ""}`;
            div.innerHTML = `
                <span class="session-item-id">${escapeHtml(id)}</span>
                <span class="session-item-preview">${escapeHtml(s.preview || inst.default_cwd)}</span>
            `;
            div.addEventListener("click", () => {
                state.activeSessionId = id;
                updateConsoleBadges();
                renderSessionsList();
            });
            listEl.appendChild(div);
        });
    }

    function sendActiveWsRpc(method, params = {}, isNotification = false) {
        if (!state.ws || state.ws.readyState !== WebSocket.OPEN) {
            return Promise.reject(new Error("WebSocket non connecté"));
        }
        const inst = getActiveInstance();
        const msg = { jsonrpc: "2.0", method, params };
        if (!isNotification) msg.id = nextId();
        logRpc("out", inst?.name || "cli", msg);
        state.ws.send(JSON.stringify(msg));
        if (isNotification) return Promise.resolve();
        return new Promise((resolve, reject) => {
            state.pendingRequests.set(msg.id, { resolve, reject });
        });
    }

    async function connectActiveCli() {
        const inst = getActiveInstance();
        if (!inst) return;

        if (state.ws) {
            state.ws.close();
            state.ws = null;
        }

        if (inst.kind === "claude" || inst.kind === "antigravity" || inst.transport === "rpc") {
            state.connected = true;
            updateConsoleBadges();
            appendMsg(
                "console-messages",
                "system",
                inst.name,
                `Connecté au pont JSON-RPC/Tmux (${inst.endpoint_url}) avec le rôle « ${inst.role} ».`
            );
            return;
        }

        const route = inst.kind === "goose" ? "/ws/goose" : "/ws/codex";
        const wsUrl = `${wsBaseUrl()}${route}?instance_id=${encodeURIComponent(inst.id)}`;
        appendMsg("console-messages", "system", inst.name, `Ouverture du flux temps réel (${inst.endpoint_url})...`);

        const ws = new WebSocket(wsUrl);
        state.ws = ws;

        ws.onopen = async () => {
            state.connected = true;
            updateConsoleBadges();
            try {
                if (inst.kind === "codex") {
                    await sendActiveWsRpc("initialize", {
                        clientInfo: { name: "simply-agent-hub", version: "0.2.0" },
                        capabilities: {},
                    });
                    await sendActiveWsRpc("initialized", {}, true);
                    const res = await sendActiveWsRpc("thread/list", {});
                    const list = res?.data || res?.threads || [];
                    state.sessionsByInstance[inst.id] = list;
                    if (list.length > 0 && !state.activeSessionId) {
                        state.activeSessionId = list[0].id || list[0].threadId;
                    }
                    renderSessionsList();
                    updateConsoleBadges();
                    appendMsg("console-messages", "system", inst.name, "Handshake Codex JSON-RPC réussi.");
                } else if (inst.kind === "goose") {
                    await sendActiveWsRpc("initialize", {
                        protocolVersion: 1,
                        clientCapabilities: { fs: { readTextFile: true, writeTextFile: true }, terminal: true },
                        clientInfo: { name: "simply-agent-hub", version: "0.2.0" },
                    });
                    appendMsg("console-messages", "system", inst.name, "Handshake Goose ACP réussi.");
                    await createNewSessionOnActiveCli();
                }
            } catch (err) {
                appendMsg("console-messages", "system", inst.name, `Erreur handshake: ${err.message}`);
            }
        };

        ws.onmessage = (ev) => {
            let msg;
            try {
                msg = JSON.parse(ev.data);
            } catch {
                logRpc("in", inst.name, ev.data);
                return;
            }
            logRpc("in", inst.name, msg);
            handleActiveWsFrame(inst, msg);
        };

        ws.onclose = () => {
            state.connected = false;
            state.turnRunning = false;
            updateConsoleBadges();
        };
    }

    function handleActiveWsFrame(inst, msg) {
        if (msg.id !== undefined && (msg.result !== undefined || msg.error !== undefined)) {
            const pending = state.pendingRequests.get(msg.id);
            if (pending) {
                state.pendingRequests.delete(msg.id);
                if (msg.error) pending.reject(new Error(msg.error.message || JSON.stringify(msg.error)));
                else pending.resolve(msg.result);
            }
            return;
        }

        if (msg.id !== undefined && msg.method) {
            state.pendingApproval = { inst, msg };
            $("console-approval-details").textContent = `${msg.method}\n${JSON.stringify(msg.params, null, 2)}`;
            $("console-approval-banner").classList.remove("hidden");
            return;
        }

        const method = msg.method || "";
        const params = msg.params || {};

        if (method === "turn/started") {
            state.turnRunning = true;
            state.streamingEl = null;
            updateConsoleBadges();
        } else if (method === "turn/completed") {
            state.turnRunning = false;
            state.streamingEl = null;
            updateConsoleBadges();
        } else if (method === "item/agentMessage/delta") {
            if (!state.streamingEl) {
                state.streamingEl = appendMsg("console-messages", "agent", inst.name, "");
            }
            state.streamingEl.textContent += params.delta || "";
            $("console-messages").scrollTop = $("console-messages").scrollHeight;
        } else if (method === "session/update") {
            const upd = params.update || {};
            if (upd.sessionUpdate === "agent_message_chunk") {
                if (!state.streamingEl) {
                    state.streamingEl = appendMsg("console-messages", "agent", inst.name, "");
                }
                state.streamingEl.textContent += upd.content?.text || "";
                $("console-messages").scrollTop = $("console-messages").scrollHeight;
            }
        } else if (method === "bridge/error") {
            appendMsg("console-messages", "system", inst.name, params.message || "Erreur pont");
        }
    }

    async function createNewSessionOnActiveCli() {
        const inst = getActiveInstance();
        if (!inst) return null;
        if (!state.sessionsByInstance[inst.id]) state.sessionsByInstance[inst.id] = [];

        if (inst.kind === "codex" && state.connected && state.ws) {
            const res = await sendActiveWsRpc("thread/start", { cwd: inst.default_cwd });
            const id = res?.thread?.id || res?.threadId || res?.id;
            if (id) {
                state.activeSessionId = id;
                state.sessionsByInstance[inst.id].unshift({ id, preview: `Thread (${inst.default_cwd})` });
                renderSessionsList();
                updateConsoleBadges();
            }
            return id;
        } else if (inst.kind === "goose" && state.connected && state.ws) {
            const res = await sendActiveWsRpc("session/new", { cwd: inst.default_cwd, mcpServers: [] });
            const id = res?.sessionId || res?.id;
            if (id) {
                state.activeSessionId = id;
                state.sessionsByInstance[inst.id].unshift({ id, preview: `Session ACP (${inst.default_cwd})` });
                renderSessionsList();
                updateConsoleBadges();
            }
            return id;
        }
        return null;
    }

    $("btn-cli-connect").addEventListener("click", connectActiveCli);
    $("btn-cli-new-session").addEventListener("click", createNewSessionOnActiveCli);

    $("btn-cli-native-web").addEventListener("click", () => {
        const inst = getActiveInstance();
        if (!inst) return;
        const proxyUrl = `/api/instances/${encodeURIComponent(inst.id)}/web`;
        const container = $("console-messages");
        const wrapper = document.createElement("div");
        wrapper.className = "msg msg-agent";
        wrapper.style.width = "100%";
        wrapper.style.maxWidth = "100%";
        wrapper.innerHTML = `
            <div class="msg-header">
                <span>🌐 Serveur Web natif de ${escapeHtml(inst.name)}</span>
                <a href="${escapeHtml(proxyUrl)}" target="_blank" rel="noopener" style="color:var(--highlight);">↗ Ouvrir dans un nouvel onglet</a>
            </div>
            <iframe src="${escapeHtml(proxyUrl)}" style="width:100%;height:420px;border:1px solid var(--border);border-radius:var(--radius);background:#111315;"></iframe>
        `;
        container.appendChild(wrapper);
        container.scrollTop = container.scrollHeight;
    });

    $("console-prompt-form").addEventListener("submit", async (e) => {
        e.preventDefault();
        const text = $("console-prompt-input").value.trim();
        if (!text) return;
        $("console-prompt-input").value = "";

        const inst = getActiveInstance();
        if (!inst) return;
        appendMsg("console-messages", "user", "Vous", text);

        if (inst.kind === "codex" && state.connected && state.ws?.readyState === WebSocket.OPEN) {
            let tid = state.activeSessionId;
            if (!tid) tid = await createNewSessionOnActiveCli();
            state.streamingEl = null;
            state.turnRunning = true;
            updateConsoleBadges();
            await sendActiveWsRpc("turn/start", {
                threadId: tid,
                input: [{ type: "text", text }],
            });
            return;
        }

        if (inst.kind === "goose" && state.connected && state.ws?.readyState === WebSocket.OPEN) {
            let sid = state.activeSessionId;
            if (!sid) sid = await createNewSessionOnActiveCli();
            state.streamingEl = null;
            state.turnRunning = true;
            updateConsoleBadges();
            await sendActiveWsRpc("session/prompt", {
                sessionId: sid,
                prompt: [{ type: "text", text }],
            });
            state.turnRunning = false;
            state.streamingEl = null;
            updateConsoleBadges();
            return;
        }

        state.turnRunning = true;
        updateConsoleBadges();
        try {
            const resp = await fetch("/api/agents/ask", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    instance_id: inst.id,
                    prompt: text,
                    cwd: inst.default_cwd,
                }),
            });
            const data = await resp.json();
            logRpc("in", inst.name, data);
            if (data.error) throw new Error(data.error);
            appendMsg("console-messages", "agent", `${inst.name} [${inst.role || inst.kind}]`, data.reply || "");
        } catch (err) {
            appendMsg("console-messages", "system", `Erreur (${inst.name})`, err.message);
        } finally {
            state.turnRunning = false;
            updateConsoleBadges();
        }
    });

    $("btn-console-approve").addEventListener("click", () => {
        if (!state.pendingApproval) return;
        const { inst, msg } = state.pendingApproval;
        state.pendingApproval = null;
        $("console-approval-banner").classList.add("hidden");
        if (inst.kind === "codex") {
            state.ws?.send(JSON.stringify({ jsonrpc: "2.0", id: msg.id, result: { decision: "accept" } }));
        } else {
            const opts = msg.params?.options || [];
            const allow = opts.find((o) => o.kind?.startsWith("allow")) || opts[0];
            state.ws?.send(
                JSON.stringify({
                    jsonrpc: "2.0",
                    id: msg.id,
                    result: { outcome: { outcome: "selected", optionId: allow?.optionId || "allow" } },
                })
            );
        }
    });

    $("btn-console-decline").addEventListener("click", () => {
        if (!state.pendingApproval) return;
        const { inst, msg } = state.pendingApproval;
        state.pendingApproval = null;
        $("console-approval-banner").classList.add("hidden");
        if (inst.kind === "codex") {
            state.ws?.send(JSON.stringify({ jsonrpc: "2.0", id: msg.id, result: { decision: "decline" } }));
        } else {
            state.ws?.send(
                JSON.stringify({
                    jsonrpc: "2.0",
                    id: msg.id,
                    result: { outcome: { outcome: "cancelled" } },
                })
            );
        }
    });

    $("btn-console-clear").addEventListener("click", () => ($("console-messages").innerHTML = ""));

    // ═════════════════════════════════════════════════════════════════════════
    // 5. ORCHESTRATION CHANNELS & REAL-TIME MULTI-AGENT BUS
    // ═════════════════════════════════════════════════════════════════════════
    async function loadChannels() {
        try {
            const resp = await fetch("/api/orchestration/channels");
            const list = await resp.json();
            state.channels = Array.isArray(list) ? list : [];
            if (!state.activeChannelId && state.channels.length > 0) {
                state.activeChannelId = state.channels[0].id;
            }
            renderChannelSelector();
            renderChannelParticipants();
            if (state.activeChannelId) {
                await loadChannelHistory(state.activeChannelId);
                connectChannelWebSocket(state.activeChannelId);
            }
        } catch {
            // ignore
        }
    }

    function getActiveChannel() {
        return state.channels.find((c) => c.id === state.activeChannelId) || null;
    }

    function renderChannelSelector() {
        const sel = $("channel-selector");
        sel.innerHTML = "";
        state.channels.forEach((ch) => {
            const opt = document.createElement("option");
            opt.value = ch.id;
            opt.textContent = `#${ch.name} — ${ch.description || "Canal d'orchestration"}`;
            if (ch.id === state.activeChannelId) opt.selected = true;
            sel.appendChild(opt);
        });
        const active = getActiveChannel();
        $("channel-title-badge").textContent = active ? `#${active.name}` : "-";
        if (active) {
            if ($("channel-retention-days")) $("channel-retention-days").value = active.retention_days ?? 30;
            if ($("channel-max-messages")) $("channel-max-messages").value = active.max_messages ?? 200;
        }
    }

    function renderChannelParticipants() {
        const container = $("channel-participants-list");
        const recipientSel = $("channel-recipient-select");
        if (!container || !recipientSel) return;

        container.innerHTML = "";
        recipientSel.innerHTML = `<option value="@all">@all (Tous les participants du canal)</option>`;

        const ch = getActiveChannel();
        let selectedIds = [];
        if (ch && ch.participant_ids_json) {
            try {
                selectedIds = JSON.parse(ch.participant_ids_json);
            } catch {
                selectedIds = [];
            }
        }

        state.instances.forEach((inst) => {
            const checked = selectedIds.includes(inst.id);
            const label = document.createElement("label");
            label.className = "participant-check";
            label.innerHTML = `
                <input type="checkbox" value="${inst.id}" ${checked ? "checked" : ""}>
                <span>${inst.online ? "🟢" : "🔴"} <strong>${escapeHtml(inst.name)}</strong> <small>[${escapeHtml(inst.role || inst.kind)}]</small></span>
            `;
            container.appendChild(label);

            const opt = document.createElement("option");
            opt.value = inst.name;
            opt.textContent = `@${inst.name} (${inst.role || inst.kind})`;
            recipientSel.appendChild(opt);
        });
    }

    $("channel-selector").addEventListener("change", async (e) => {
        state.activeChannelId = e.target.value;
        renderChannelSelector();
        renderChannelParticipants();
        await loadChannelHistory(state.activeChannelId);
        connectChannelWebSocket(state.activeChannelId);
    });

    $("btn-open-new-channel").addEventListener("click", () => {
        $("new-channel-box").classList.toggle("hidden");
    });
    $("btn-cancel-channel").addEventListener("click", () => {
        $("new-channel-box").classList.add("hidden");
    });

    $("btn-create-channel").addEventListener("click", async () => {
        const name = $("new-channel-name").value.trim();
        const description = $("new-channel-desc").value.trim();
        if (!name) return;
        const allIds = state.instances.map((i) => i.id);
        const resp = await fetch("/api/orchestration/channels", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ name, description, participant_ids: allIds }),
        });
        const created = await resp.json();
        if (created.id) {
            state.activeChannelId = created.id;
            $("new-channel-name").value = "";
            $("new-channel-desc").value = "";
            $("new-channel-box").classList.add("hidden");
            await loadChannels();
        }
    });

    $("btn-delete-channel").addEventListener("click", async () => {
        if (!state.activeChannelId) return;
        await fetch(`/api/orchestration/channels/${state.activeChannelId}`, { method: "DELETE" });
        state.activeChannelId = null;
        await loadChannels();
    });

    $("btn-save-participants").addEventListener("click", async () => {
        if (!state.activeChannelId) return;
        const checked = Array.from(
            $("channel-participants-list").querySelectorAll('input[type="checkbox"]:checked')
        ).map((el) => el.value);

        await fetch(`/api/orchestration/channels/${state.activeChannelId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ participant_ids: checked }),
        });
        await loadChannels();
        appendMsg("channel-messages", "system", "Orchestrateur", "Participants du canal mis à jour en base SeaORM.");
    });

    async function loadChannelHistory(channelId) {
        $("channel-messages").innerHTML = "";
        state.seenMessageIds.clear();
        const resp = await fetch(`/api/orchestration/channels/${channelId}/messages`);
        const msgs = await resp.json();
        if (Array.isArray(msgs)) {
            msgs.forEach(renderChannelMessageObj);
        }
    }

    function renderChannelMessageObj(m) {
        if (!m || !m.id || state.seenMessageIds.has(m.id)) return;
        state.seenMessageIds.add(m.id);
        const role = m.sender_name === "Utilisateur" ? "user" : "agent";
        const label = `${m.sender_name} → ${m.recipient_name || "@all"}`;
        appendMsg("channel-messages", role, label, m.content);
    }

    function connectChannelWebSocket(channelId) {
        if (state.channelWs) {
            state.channelWs.close();
            state.channelWs = null;
        }
        const ws = new WebSocket(`${wsBaseUrl()}/ws/orchestration/${channelId}`);
        state.channelWs = ws;
        ws.onmessage = (ev) => {
            try {
                const msgObj = JSON.parse(ev.data);
                renderChannelMessageObj(msgObj);
            } catch {
                // ignore
            }
        };
    }

    $("channel-publish-form").addEventListener("submit", async (e) => {
        e.preventDefault();
        if (!state.activeChannelId) return;
        const content = $("channel-message-input").value.trim();
        if (!content) return;
        $("channel-message-input").value = "";

        const recipient = $("channel-recipient-select").value || "@all";
        const resp = await fetch(`/api/orchestration/channels/${state.activeChannelId}/publish`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                sender_name: "Utilisateur",
                recipient_name: recipient,
                content,
                auto_dispatch: true,
            }),
        });
        const data = await resp.json();
        if (data.message) renderChannelMessageObj(data.message);
        if (Array.isArray(data.replies)) {
            data.replies.forEach(renderChannelMessageObj);
        }
    });

    $("btn-refresh-channel-msgs").addEventListener("click", () => {
        if (state.activeChannelId) loadChannelHistory(state.activeChannelId);
    });

    $("btn-save-retention")?.addEventListener("click", async () => {
        if (!state.activeChannelId) return;
        const retention_days = parseInt($("channel-retention-days").value, 10) || 0;
        const max_messages = parseInt($("channel-max-messages").value, 10) || 0;
        await fetch(`/api/orchestration/channels/${state.activeChannelId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ retention_days, max_messages }),
        });
        const pruneResp = await fetch(`/api/orchestration/channels/${state.activeChannelId}/prune`, {
            method: "POST",
        });
        const pruneData = await pruneResp.json();
        await loadChannels();
        appendMsg(
            "channel-messages",
            "system",
            "Rétention SeaORM",
            `Politique appliquée (${retention_days}j / max ${max_messages} messages). Messages purgés: ${pruneData.pruned_messages ?? 0}.`
        );
    });

    $("btn-export-channel-md")?.addEventListener("click", () => {
        if (!state.activeChannelId) return;
        window.open(`/api/orchestration/channels/${encodeURIComponent(state.activeChannelId)}/export`, "_blank");
    });

    // ═════════════════════════════════════════════════════════════════════════
    // 6. INTEGRATED GIT DIFF, STATUS, COMMIT & REVERT (/api/git/*)
    // ═════════════════════════════════════════════════════════════════════════
    function renderHighlightedDiff(diffText) {
        const viewer = $("git-diff-viewer");
        if (!viewer) return;
        if (!diffText || !diffText.trim()) {
            viewer.innerHTML = `<span class="field-hint">Aucune modification non commitée dans ce dépôt.</span>`;
            return;
        }
        const lines = diffText.split("\n");
        const htmlLines = lines.map((line) => {
            const safe = escapeHtml(line);
            if (line.startsWith("+++") || line.startsWith("---") || line.startsWith("diff --git")) {
                return `<div class="diff-line diff-header">${safe}</div>`;
            }
            if (line.startsWith("@@")) {
                return `<div class="diff-line diff-hunk">${safe}</div>`;
            }
            if (line.startsWith("+")) {
                return `<div class="diff-line diff-add">${safe}</div>`;
            }
            if (line.startsWith("-")) {
                return `<div class="diff-line diff-del">${safe}</div>`;
            }
            return `<div class="diff-line">${safe}</div>`;
        });
        viewer.innerHTML = htmlLines.join("");
    }

    async function loadGitStatus() {
        const cwd = $("git-cwd-input")?.value?.trim() || "/srv";
        try {
            const resp = await fetch(`/api/git/status?cwd=${encodeURIComponent(cwd)}`);
            const data = await resp.json();
            $("git-branch-badge").textContent = data.branch || "-";

            const filesList = $("git-files-list");
            filesList.innerHTML = "";
            if (!data.files || data.files.length === 0) {
                filesList.innerHTML = `<div class="field-hint">Arbre de travail propre (0 fichier modifié)</div>`;
            } else {
                data.files.forEach((f) => {
                    const row = document.createElement("div");
                    row.className = "git-file-row";
                    row.innerHTML = `
                        <span><strong style="color:var(--accent);">${escapeHtml(f.status)}</strong> ${escapeHtml(f.path)}</span>
                        <button class="btn btn-danger btn-sm" data-revert-file="${escapeHtml(f.path)}">↩ Revert</button>
                    `;
                    filesList.appendChild(row);
                });
                filesList.querySelectorAll("button[data-revert-file]").forEach((b) => {
                    b.addEventListener("click", async () => {
                        const file = b.getAttribute("data-revert-file");
                        await fetch("/api/git/revert", {
                            method: "POST",
                            headers: { "Content-Type": "application/json" },
                            body: JSON.stringify({ cwd, file }),
                        });
                        await loadGitStatus();
                    });
                });
            }

            const recentEl = $("git-recent-commits");
            recentEl.innerHTML = "";
            (data.recent_commits || []).forEach((c) => {
                const d = document.createElement("div");
                d.className = "session-item";
                d.textContent = c;
                recentEl.appendChild(d);
            });

            renderHighlightedDiff(data.diff || "");
        } catch (err) {
            $("git-diff-viewer").textContent = `Erreur Git: ${err.message}`;
        }
    }

    $("btn-git-refresh")?.addEventListener("click", loadGitStatus);

    $("btn-git-commit")?.addEventListener("click", async () => {
        const cwd = $("git-cwd-input").value.trim() || "/srv";
        const message = $("git-commit-msg").value.trim();
        if (!message) return;
        const resp = await fetch("/api/git/commit", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ cwd, message }),
        });
        const data = await resp.json();
        logRpc("in", "git:commit", data);
        $("git-commit-msg").value = "";
        await loadGitStatus();
    });

    $("btn-git-revert-all")?.addEventListener("click", async () => {
        const cwd = $("git-cwd-input").value.trim() || "/srv";
        await fetch("/api/git/revert", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ cwd, file: "." }),
        });
        await loadGitStatus();
    });

    $("btn-refresh-all").addEventListener("click", () => {
        loadInstances();
        loadChannels();
        loadGitStatus();
    });

    $("btn-clear-rpc").addEventListener("click", () => ($("rpc-log").innerHTML = ""));

    // Init
    loadRolePresets()
        .then(loadInstances)
        .then(loadChannels);
    setInterval(loadInstances, 15000);
})();
