(() => {
    const state = {
        rpcSeq: 1,
        pendingRequests: new Map(),
        pendingApproval: { codex: null, goose: null },

        codex: {
            ws: null,
            connected: false,
            activeThreadId: null,
            threads: [],
            streamingEl: null,
            turnRunning: false,
        },

        goose: {
            ws: null,
            connected: false,
            mode: "ws",
            activeSessionId: null,
            sessions: [],
            streamingEl: null,
            turnRunning: false,
        },
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
        return String(str)
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
    }

    function logRpc(direction, agent, payload) {
        const log = $("rpc-log");
        if (!log) return;
        const div = document.createElement("div");
        div.className = `msg ${direction === "out" ? "msg-user" : "msg-tool"}`;
        div.style.maxWidth = "100%";
        div.style.padding = "0.45rem 0.75rem";
        const raw = typeof payload === "string" ? payload : JSON.stringify(payload);
        div.textContent = `[${new Date().toLocaleTimeString()}] [${agent.toUpperCase()}] ${direction === "out" ? "→ OUT" : "← IN "} ${raw}`;
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

    // ─── Tab Navigation (simply_ip_vault pattern) ────────────────────────────
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
        });
    });

    // ═════════════════════════════════════════════════════════════════════════
    // 1. CODEX APP-SERVER (JSON-RPC 2.0 over WS or Unix Socket)
    // ═════════════════════════════════════════════════════════════════════════
    function updateCodexUI() {
        const badge = $("codex-conn-badge");
        badge.textContent = state.codex.connected ? "Connecté (Codex)" : "Déconnecté";
        badge.className = `badge ${state.codex.connected ? "badge-online" : "badge-offline"}`;

        $("codex-active-thread").textContent = state.codex.activeThreadId
            ? `Thread : ${state.codex.activeThreadId}`
            : "Aucun thread";

        $("btn-codex-interrupt").classList.toggle("hidden", !state.codex.turnRunning);

        const listEl = $("codex-thread-list");
        listEl.innerHTML = "";
        if (!state.codex.threads.length) {
            listEl.innerHTML = `<div class="field-hint">Aucun thread actif</div>`;
            return;
        }
        state.codex.threads.forEach((t) => {
            const id = t.id || t.threadId || String(t);
            const preview = t.preview || t.title || t.cwd || "Thread Codex";
            const item = document.createElement("div");
            item.className = `session-item ${state.codex.activeThreadId === id ? "active" : ""}`;
            item.innerHTML = `
                <span class="session-item-id">${escapeHtml(id)}</span>
                <span class="session-item-preview">${escapeHtml(preview)}</span>
            `;
            item.addEventListener("click", () => resumeCodexThread(id));
            listEl.appendChild(item);
        });
    }

    function sendCodexRpc(method, params = {}, isNotification = false) {
        if (!state.codex.ws || state.codex.ws.readyState !== WebSocket.OPEN) {
            appendMsg("codex-messages", "system", "Codex", "WebSocket Codex non connecté.");
            return Promise.reject(new Error("Codex socket not open"));
        }
        const msg = { jsonrpc: "2.0", method, params };
        if (!isNotification) msg.id = nextId();
        logRpc("out", "codex", msg);
        state.codex.ws.send(JSON.stringify(msg));
        if (isNotification) return Promise.resolve();
        return new Promise((resolve, reject) => {
            state.pendingRequests.set(msg.id, { resolve, reject });
        });
    }

    function connectCodex() {
        if (state.codex.ws) state.codex.ws.close();
        const mode = $("codex-mode").value;
        const target = $("codex-target").value.trim();
        const params = new URLSearchParams({ mode });
        if (target) params.set("target", target);

        appendMsg("codex-messages", "system", "Codex", `Connexion au pont Rust (${mode})...`);
        const ws = new WebSocket(`${wsBaseUrl()}/ws/codex?${params.toString()}`);
        state.codex.ws = ws;

        ws.onopen = async () => {
            state.codex.connected = true;
            updateCodexUI();
            try {
                await sendCodexRpc("initialize", {
                    clientInfo: { name: "simply-agent-hub", version: "1.0.0" },
                    capabilities: {},
                });
                await sendCodexRpc("initialized", {}, true);
                appendMsg("codex-messages", "system", "Codex", "Handshake initialize/initialized réussi.");
                await refreshCodexThreads();
            } catch (err) {
                appendMsg("codex-messages", "system", "Erreur Codex", err.message);
            }
        };

        ws.onmessage = (ev) => {
            let msg;
            try {
                msg = JSON.parse(ev.data);
            } catch {
                logRpc("in", "codex", ev.data);
                return;
            }
            logRpc("in", "codex", msg);
            handleCodexFrame(msg);
        };

        ws.onclose = () => {
            state.codex.connected = false;
            state.codex.turnRunning = false;
            updateCodexUI();
            appendMsg("codex-messages", "system", "Codex", "Connexion fermée.");
        };
    }

    function handleCodexFrame(msg) {
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
            state.pendingApproval.codex = msg;
            $("codex-approval-details").textContent = `${msg.method}\n${JSON.stringify(msg.params, null, 2)}`;
            $("codex-approval-banner").classList.remove("hidden");
            return;
        }

        const method = msg.method || "";
        const params = msg.params || {};

        if (method === "turn/started") {
            state.codex.turnRunning = true;
            state.codex.streamingEl = null;
            updateCodexUI();
        } else if (method === "turn/completed") {
            const finalReply = state.codex.streamingEl ? state.codex.streamingEl.textContent : "";
            state.codex.turnRunning = false;
            state.codex.streamingEl = null;
            updateCodexUI();
            if (finalReply) checkAndForwardMentions("codex", finalReply, "codex-messages");
        } else if (method === "item/agentMessage/delta") {
            if (!state.codex.streamingEl) {
                state.codex.streamingEl = appendMsg("codex-messages", "agent", "Codex", "");
            }
            state.codex.streamingEl.textContent += params.delta || "";
            $("codex-messages").scrollTop = $("codex-messages").scrollHeight;
        } else if (method === "item/started") {
            const item = params.item || {};
            if (item.type === "commandExecution") {
                appendMsg("codex-messages", "tool", "Codex Outil • Commande", `$ ${item.command || JSON.stringify(item)}`);
            } else if (item.type === "fileChange") {
                appendMsg("codex-messages", "tool", "Codex Outil • Fichier", JSON.stringify(item.changes || item, null, 2));
            }
        } else if (method === "bridge/error" || method === "error") {
            appendMsg("codex-messages", "system", "Erreur Codex", params.message || JSON.stringify(params));
        }
    }

    async function refreshCodexThreads() {
        if (!state.codex.connected) return;
        try {
            const res = await sendCodexRpc("thread/list", {});
            const list = res?.data || res?.threads || (Array.isArray(res) ? res : []);
            state.codex.threads = list;
            if (!state.codex.activeThreadId && list.length > 0) {
                state.codex.activeThreadId = list[0].id || list[0].threadId;
            }
            updateCodexUI();
        } catch (err) {
            appendMsg("codex-messages", "system", "Codex", `Impossible de lister les threads: ${err.message}`);
        }
    }

    async function startNewCodexThread() {
        if (!state.codex.connected) {
            appendMsg("codex-messages", "system", "Codex", "Connectez-vous d'abord à Codex.");
            return null;
        }
        const cwd = $("codex-cwd").value.trim() || "/srv";
        try {
            const res = await sendCodexRpc("thread/start", { cwd });
            const id = res?.thread?.id || res?.threadId || res?.id;
            if (id) {
                state.codex.activeThreadId = id;
                state.codex.threads.unshift({ id, cwd, preview: `Thread (${cwd})` });
                updateCodexUI();
                appendMsg("codex-messages", "system", "Codex", `Nouveau thread créé : ${id}`);
            }
            return id;
        } catch (err) {
            appendMsg("codex-messages", "system", "Erreur Codex", err.message);
            return null;
        }
    }

    async function resumeCodexThread(threadId) {
        state.codex.activeThreadId = threadId;
        updateCodexUI();
        if (!state.codex.connected) return;
        try {
            await sendCodexRpc("thread/resume", { threadId });
            appendMsg("codex-messages", "system", "Codex", `Thread actif : ${threadId}`);
        } catch {
            appendMsg("codex-messages", "system", "Codex", `Thread sélectionné : ${threadId}`);
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // 2. GOOSE ACP (JSON-RPC 2.0 over WebSocket or HTTP)
    // ═════════════════════════════════════════════════════════════════════════
    function updateGooseUI() {
        const badge = $("goose-conn-badge");
        badge.textContent = state.goose.connected ? "Connecté (Goose ACP)" : "Déconnecté";
        badge.className = `badge ${state.goose.connected ? "badge-online" : "badge-offline"}`;

        $("goose-active-session").textContent = state.goose.activeSessionId
            ? `Session : ${state.goose.activeSessionId}`
            : "Aucune session";

        $("btn-goose-cancel").classList.toggle("hidden", !state.goose.turnRunning);

        const listEl = $("goose-session-list");
        listEl.innerHTML = "";
        if (!state.goose.sessions.length) {
            listEl.innerHTML = `<div class="field-hint">Aucune session active</div>`;
            return;
        }
        state.goose.sessions.forEach((s) => {
            const id = s.id || s.sessionId;
            const item = document.createElement("div");
            item.className = `session-item ${state.goose.activeSessionId === id ? "active" : ""}`;
            item.innerHTML = `
                <span class="session-item-id">${escapeHtml(id)}</span>
                <span class="session-item-preview">${escapeHtml(s.cwd || "/srv")}</span>
            `;
            item.addEventListener("click", () => {
                state.goose.activeSessionId = id;
                updateGooseUI();
            });
            listEl.appendChild(item);
        });
    }

    async function sendGooseRpc(method, params = {}, isNotification = false) {
        const mode = $("goose-mode").value;
        const msg = { jsonrpc: "2.0", method, params };
        if (!isNotification) msg.id = nextId();
        logRpc("out", "goose", msg);

        if (mode === "http") {
            const headers = { "Content-Type": "application/json" };
            const token = $("goose-token").value.trim();
            if (token) headers["X-Secret-Key"] = token;
            const resp = await fetch("/api/goose/acp", {
                method: "POST",
                headers,
                body: JSON.stringify(msg),
            });
            const data = await resp.json();
            logRpc("in", "goose", data);
            handleGooseFrame(data);
            if (data.error) throw new Error(data.error.message || JSON.stringify(data.error));
            return data.result;
        }

        if (!state.goose.ws || state.goose.ws.readyState !== WebSocket.OPEN) {
            throw new Error("WebSocket Goose non connecté");
        }
        state.goose.ws.send(JSON.stringify(msg));
        if (isNotification) return;
        return new Promise((resolve, reject) => {
            state.pendingRequests.set(msg.id, { resolve, reject });
        });
    }

    async function connectGoose() {
        const mode = $("goose-mode").value;
        if (mode === "http") {
            try {
                state.goose.connected = true;
                updateGooseUI();
                await sendGooseRpc("initialize", {
                    protocolVersion: 1,
                    clientCapabilities: { fs: { readTextFile: true, writeTextFile: true }, terminal: true },
                    clientInfo: { name: "simply-agent-hub", version: "1.0.0" },
                });
                appendMsg("goose-messages", "system", "Goose", "Connecté via HTTP JSON-RPC (/api/goose/acp).");
                await startNewGooseSession();
            } catch (err) {
                state.goose.connected = false;
                updateGooseUI();
                appendMsg("goose-messages", "system", "Erreur Goose", err.message);
            }
            return;
        }

        if (state.goose.ws) state.goose.ws.close();
        const target = $("goose-target").value.trim();
        const token = $("goose-token").value.trim();
        const params = new URLSearchParams();
        if (target) params.set("target", target);
        if (token) params.set("token", token);

        appendMsg("goose-messages", "system", "Goose", "Connexion au pont WebSocket ACP Goose...");
        const ws = new WebSocket(`${wsBaseUrl()}/ws/goose?${params.toString()}`);
        state.goose.ws = ws;

        ws.onopen = async () => {
            state.goose.connected = true;
            updateGooseUI();
            try {
                await sendGooseRpc("initialize", {
                    protocolVersion: 1,
                    clientCapabilities: { fs: { readTextFile: true, writeTextFile: true }, terminal: true },
                    clientInfo: { name: "simply-agent-hub", version: "1.0.0" },
                });
                appendMsg("goose-messages", "system", "Goose", "Handshake ACP initialize réussi.");
                if (!state.goose.activeSessionId) await startNewGooseSession();
            } catch (err) {
                appendMsg("goose-messages", "system", "Erreur Goose", err.message);
            }
        };

        ws.onmessage = (ev) => {
            let msg;
            try {
                msg = JSON.parse(ev.data);
            } catch {
                logRpc("in", "goose", ev.data);
                return;
            }
            logRpc("in", "goose", msg);
            handleGooseFrame(msg);
        };

        ws.onclose = () => {
            state.goose.connected = false;
            state.goose.turnRunning = false;
            updateGooseUI();
            appendMsg("goose-messages", "system", "Goose", "Connexion WebSocket Goose fermée.");
        };
    }

    function handleGooseFrame(msg) {
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
            state.pendingApproval.goose = msg;
            $("goose-approval-details").textContent = `${msg.method}\n${JSON.stringify(msg.params, null, 2)}`;
            $("goose-approval-banner").classList.remove("hidden");
            return;
        }

        if (msg.method === "session/update") {
            const update = msg.params?.update || {};
            if (update.sessionUpdate === "agent_message_chunk") {
                if (!state.goose.streamingEl) {
                    state.goose.streamingEl = appendMsg("goose-messages", "agent", "Goose", "");
                }
                state.goose.streamingEl.textContent += update.content?.text || "";
                $("goose-messages").scrollTop = $("goose-messages").scrollHeight;
            } else if (update.sessionUpdate === "tool_call") {
                appendMsg("goose-messages", "tool", `Goose Outil • ${update.title || "Tool"}`, JSON.stringify(update, null, 2));
            }
        } else if (msg.method === "bridge/error") {
            appendMsg("goose-messages", "system", "Erreur Goose", msg.params?.message || "");
        }
    }

    async function startNewGooseSession() {
        const cwd = $("goose-cwd").value.trim() || "/srv";
        try {
            const res = await sendGooseRpc("session/new", { cwd, mcpServers: [] });
            const id = res?.sessionId || res?.id;
            if (id) {
                state.goose.activeSessionId = id;
                state.goose.sessions.unshift({ id, cwd });
                updateGooseUI();
                appendMsg("goose-messages", "system", "Goose", `Session ACP créée : ${id}`);
            }
            return id;
        } catch (err) {
            appendMsg("goose-messages", "system", "Erreur Goose", err.message);
            return null;
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // 3. INTER-AGENT BRIDGE & ORCHESTRATION (Codex <-> Goose <-> Claude <-> Antigravity)
    // ═════════════════════════════════════════════════════════════════════════
    async function askSingleAgentViaBackend(agent, prompt, cwd = "/srv", mode = "auto") {
        logRpc("out", `bridge->${agent}`, { agent, prompt, cwd, mode });
        const resp = await fetch("/api/agents/ask", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ agent, prompt, cwd, mode }),
        });
        const data = await resp.json();
        logRpc("in", `bridge<-${agent}`, data);
        if (data.error) throw new Error(data.error);
        return data.reply || "";
    }

    async function checkAndForwardMentions(sourceAgent, replyText, containerId) {
        const autoMention = $("bridge-auto-mention");
        if (autoMention && !autoMention.checked) return;

        const regex = /@(codex|goose|claude|antigravity)\b\s*[:,-]?\s*(.+)/gi;
        let match;
        while ((match = regex.exec(replyText)) !== null) {
            const targetAgent = match[1].toLowerCase();
            const subPrompt = match[2].trim();
            if (!subPrompt || targetAgent === sourceAgent) continue;

            appendMsg(
                containerId,
                "system",
                "Pont Inter-Agents",
                `Relais automatique de ${sourceAgent.toUpperCase()} vers @${targetAgent.toUpperCase()} : « ${subPrompt} »`
            );
            try {
                const subReply = await askSingleAgentViaBackend(targetAgent, subPrompt, "/srv");
                appendMsg(containerId, "agent", `${targetAgent.toUpperCase()} (via Pont Inter-Agents)`, subReply);
            } catch (err) {
                appendMsg(containerId, "system", `Erreur @${targetAgent}`, err.message);
            }
        }
    }

    $("bridge-form").addEventListener("submit", async (e) => {
        e.preventDefault();
        const initialPrompt = $("bridge-input").value.trim();
        if (!initialPrompt) return;
        $("bridge-input").value = "";

        const steps = [
            $("bridge-step-1").value,
            $("bridge-step-2").value,
            $("bridge-step-3").value,
        ].filter(Boolean);

        const cwd = $("bridge-cwd").value.trim() || "/srv";
        const badge = $("bridge-status-badge");
        badge.textContent = "Orchestration en cours...";

        appendMsg("bridge-messages", "user", `Mission (${steps.join(" → ")})`, initialPrompt);

        let accumulatedContext = `Objectif initial de l'utilisateur :\n${initialPrompt}\n`;

        for (let i = 0; i < steps.length; i++) {
            const agent = steps[i];
            badge.textContent = `Étape ${i + 1}/${steps.length} : ${agent.toUpperCase()}...`;

            const stepPrompt =
                i === 0
                    ? initialPrompt
                    : `${accumulatedContext}\n---\nEn tant qu'agent ${agent.toUpperCase()} (étape ${i + 1}/${steps.length}), poursuis le travail, vérifie ou complète la réponse précédente :`;

            try {
                const reply = await askSingleAgentViaBackend(agent, stepPrompt, cwd);
                appendMsg("bridge-messages", "agent", `Étape ${i + 1} • ${agent.toUpperCase()}`, reply);
                accumulatedContext += `\n[Réponse de ${agent.toUpperCase()}] :\n${reply}\n`;
                await checkAndForwardMentions(agent, reply, "bridge-messages");
            } catch (err) {
                appendMsg("bridge-messages", "system", `Erreur (${agent.toUpperCase()})`, err.message);
                break;
            }
        }

        badge.textContent = "Prêt";
    });

    // ═════════════════════════════════════════════════════════════════════════
    // 4. CLAUDE & ANTIGRAVITY DIRECT BRIDGE & TMUX CAPTURE
    // ═════════════════════════════════════════════════════════════════════════
    $("peer-agent").addEventListener("change", () => {
        const ag = $("peer-agent").value;
        $("peer-active-label").textContent = `Agent : ${ag === "claude" ? "Claude Code" : "Antigravity CLI"}`;
    });

    $("btn-peer-capture").addEventListener("click", async () => {
        const agent = $("peer-agent").value;
        $("peer-tmux-output").textContent = `Capture de la session tmux de ${agent}...`;
        try {
            const resp = await fetch("/api/agents/tmux", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ agent, lines: 120 }),
            });
            const data = await resp.json();
            if (data.error) throw new Error(data.error);
            $("peer-tmux-output").textContent = data.output || "(Aucune sortie dans le panneau tmux)";
        } catch (err) {
            $("peer-tmux-output").textContent = `Erreur capture tmux (${agent}): ${err.message}`;
        }
    });

    $("peer-form").addEventListener("submit", async (e) => {
        e.preventDefault();
        const text = $("peer-input").value.trim();
        if (!text) return;
        $("peer-input").value = "";

        const agent = $("peer-agent").value;
        const mode = $("peer-mode").value;
        appendMsg("peer-messages", "user", "Vous", text);

        try {
            const reply = await askSingleAgentViaBackend(agent, text, "/srv", mode);
            appendMsg("peer-messages", "agent", agent.toUpperCase(), reply);
            if (mode === "tmux") {
                $("peer-tmux-output").textContent = reply;
            }
        } catch (err) {
            appendMsg("peer-messages", "system", `Erreur ${agent}`, err.message);
        }
    });

    // ═════════════════════════════════════════════════════════════════════════
    // 5. EVENT LISTENERS & STATUS PROBES
    // ═════════════════════════════════════════════════════════════════════════
    $("btn-codex-connect").addEventListener("click", connectCodex);
    $("btn-codex-new").addEventListener("click", startNewCodexThread);
    $("btn-codex-refresh").addEventListener("click", refreshCodexThreads);
    $("btn-codex-clear").addEventListener("click", () => ($("codex-messages").innerHTML = ""));
    $("btn-codex-interrupt").addEventListener("click", () => {
        if (state.codex.activeThreadId) {
            sendCodexRpc("turn/interrupt", { threadId: state.codex.activeThreadId });
        }
    });

    $("btn-codex-approve").addEventListener("click", () => {
        const req = state.pendingApproval.codex;
        if (!req) return;
        state.pendingApproval.codex = null;
        $("codex-approval-banner").classList.add("hidden");
        state.codex.ws?.send(JSON.stringify({ jsonrpc: "2.0", id: req.id, result: { decision: "accept" } }));
    });

    $("btn-codex-decline").addEventListener("click", () => {
        const req = state.pendingApproval.codex;
        if (!req) return;
        state.pendingApproval.codex = null;
        $("codex-approval-banner").classList.add("hidden");
        state.codex.ws?.send(JSON.stringify({ jsonrpc: "2.0", id: req.id, result: { decision: "decline" } }));
    });

    $("codex-prompt-form").addEventListener("submit", async (e) => {
        e.preventDefault();
        const text = $("codex-prompt-input").value.trim();
        if (!text) return;
        $("codex-prompt-input").value = "";
        appendMsg("codex-messages", "user", "Vous", text);

        if (!state.codex.connected) {
            appendMsg("codex-messages", "system", "Codex", "Connectez-vous d'abord à Codex.");
            return;
        }
        let threadId = state.codex.activeThreadId;
        if (!threadId) {
            threadId = await startNewCodexThread();
            if (!threadId) return;
        }
        state.codex.streamingEl = null;
        state.codex.turnRunning = true;
        updateCodexUI();
        try {
            await sendCodexRpc("turn/start", { threadId, input: [{ type: "text", text }] });
        } catch (err) {
            state.codex.turnRunning = false;
            updateCodexUI();
            appendMsg("codex-messages", "system", "Erreur Codex", err.message);
        }
    });

    $("btn-goose-connect").addEventListener("click", connectGoose);
    $("btn-goose-new").addEventListener("click", startNewGooseSession);
    $("btn-goose-clear").addEventListener("click", () => ($("goose-messages").innerHTML = ""));
    $("btn-goose-cancel").addEventListener("click", () => {
        if (state.goose.activeSessionId) {
            sendGooseRpc("session/cancel", { sessionId: state.goose.activeSessionId }, true);
        }
    });

    $("btn-goose-approve").addEventListener("click", () => {
        const req = state.pendingApproval.goose;
        if (!req) return;
        state.pendingApproval.goose = null;
        $("goose-approval-banner").classList.add("hidden");
        const opts = req.params?.options || [];
        const allow = opts.find((o) => o.kind?.startsWith("allow")) || opts[0];
        state.goose.ws?.send(
            JSON.stringify({
                jsonrpc: "2.0",
                id: req.id,
                result: { outcome: { outcome: "selected", optionId: allow?.optionId || "allow" } },
            })
        );
    });

    $("btn-goose-decline").addEventListener("click", () => {
        const req = state.pendingApproval.goose;
        if (!req) return;
        state.pendingApproval.goose = null;
        $("goose-approval-banner").classList.add("hidden");
        state.goose.ws?.send(
            JSON.stringify({
                jsonrpc: "2.0",
                id: req.id,
                result: { outcome: { outcome: "cancelled" } },
            })
        );
    });

    $("goose-prompt-form").addEventListener("submit", async (e) => {
        e.preventDefault();
        const text = $("goose-prompt-input").value.trim();
        if (!text) return;
        $("goose-prompt-input").value = "";
        appendMsg("goose-messages", "user", "Vous", text);

        if (!state.goose.connected) {
            appendMsg("goose-messages", "system", "Goose", "Connectez-vous d'abord à Goose.");
            return;
        }
        let sessionId = state.goose.activeSessionId;
        if (!sessionId) {
            sessionId = await startNewGooseSession();
            if (!sessionId) return;
        }
        state.goose.streamingEl = null;
        state.goose.turnRunning = true;
        updateGooseUI();
        try {
            const res = await sendGooseRpc("session/prompt", {
                sessionId,
                prompt: [{ type: "text", text }],
            });
            const finalReply = state.goose.streamingEl ? state.goose.streamingEl.textContent : res?.text || "";
            state.goose.turnRunning = false;
            state.goose.streamingEl = null;
            updateGooseUI();
            if (finalReply) await checkAndForwardMentions("goose", finalReply, "goose-messages");
        } catch (err) {
            state.goose.turnRunning = false;
            updateGooseUI();
            appendMsg("goose-messages", "system", "Erreur Goose", err.message);
        }
    });

    $("btn-bridge-clear").addEventListener("click", () => ($("bridge-messages").innerHTML = ""));
    $("btn-peer-clear").addEventListener("click", () => ($("peer-messages").innerHTML = ""));
    $("btn-clear-rpc").addEventListener("click", () => ($("rpc-log").innerHTML = ""));

    $("rpc-custom-form").addEventListener("submit", (e) => {
        e.preventDefault();
        const raw = $("rpc-custom-input").value.trim();
        if (!raw) return;
        $("rpc-custom-input").value = "";
        const target = $("rpc-target-agent").value;
        logRpc("out", target, raw);
        if (target === "codex") state.codex.ws?.send(raw);
        else state.goose.ws?.send(raw);
    });

    async function loadConfigAndProbe() {
        try {
            const cfg = await (await fetch("/api/config")).json();
            if (cfg.codex_unix_available) $("codex-mode").value = "unix";
            if (cfg.has_goose_secret) {
                $("goose-token").placeholder = "Clé configurée côté serveur Rust ✓";
            }
        } catch {
            // ignore
        }
        await probeAllStatus();
    }

    async function probeAllStatus() {
        try {
            const st = await (await fetch("/api/status")).json();
            const setBadge = (badgeId, dotId, online, label) => {
                const b = $(badgeId);
                const d = $(dotId);
                if (b) b.className = `badge ${online ? "badge-online" : "badge-offline"}`;
                if (d) d.className = `status-dot ${online ? "online" : "offline"}`;
                if (b && label) b.title = label;
            };
            const codexOnline = Boolean(st.codex_ws?.online || st.codex_unix?.online);
            setBadge("badge-codex", "dot-codex", codexOnline, st.codex_ws?.detail || st.codex_unix?.detail);
            setBadge("badge-goose", "dot-goose", Boolean(st.goose?.online), st.goose?.detail);
            setBadge("badge-claude", "dot-claude", Boolean(st.claude?.online), st.claude?.detail);
            setBadge("badge-antigravity", "dot-antigravity", Boolean(st.antigravity?.online), st.antigravity?.detail);
        } catch {
            // ignore
        }
    }

    $("btn-probe").addEventListener("click", probeAllStatus);
    loadConfigAndProbe();
    setInterval(probeAllStatus, 10000);
})();
