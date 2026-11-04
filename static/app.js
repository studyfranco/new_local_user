(() => {
  const state = {
    activeAgent: "codex", // "codex" | "goose"
    rpcSeq: 1,
    pendingRequests: new Map(),
    pendingApproval: null,

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
      mode: "ws", // "ws" | "http"
      activeSessionId: null,
      sessions: [],
      streamingEl: null,
      turnRunning: false,
    },
  };

  // DOM Elements
  const el = {
    tabCodex: document.getElementById("tab-codex"),
    tabGoose: document.getElementById("tab-goose"),
    dotCodex: document.getElementById("dot-codex"),
    dotGoose: document.getElementById("dot-goose"),
    configTitle: document.getElementById("config-title"),
    controlsCodex: document.getElementById("controls-codex"),
    controlsGoose: document.getElementById("controls-goose"),
    codexMode: document.getElementById("codex-mode"),
    codexTarget: document.getElementById("codex-target"),
    codexCwd: document.getElementById("codex-cwd"),
    btnCodexConnect: document.getElementById("btn-codex-connect"),
    btnCodexNew: document.getElementById("btn-codex-new"),
    gooseMode: document.getElementById("goose-mode"),
    gooseTarget: document.getElementById("goose-target"),
    gooseToken: document.getElementById("goose-token"),
    gooseCwd: document.getElementById("goose-cwd"),
    btnGooseConnect: document.getElementById("btn-goose-connect"),
    btnGooseNew: document.getElementById("btn-goose-new"),
    btnProbe: document.getElementById("btn-probe"),
    sessionsLabel: document.getElementById("sessions-label"),
    btnRefreshSessions: document.getElementById("btn-refresh-sessions"),
    sessionList: document.getElementById("session-list"),
    connBadge: document.getElementById("conn-badge"),
    activeSessionDisplay: document.getElementById("active-session-display"),
    btnInterrupt: document.getElementById("btn-interrupt"),
    btnClearChat: document.getElementById("btn-clear-chat"),
    messages: document.getElementById("messages-container"),
    approvalBanner: document.getElementById("approval-banner"),
    approvalDetails: document.getElementById("approval-details"),
    btnApprove: document.getElementById("btn-approve"),
    btnDecline: document.getElementById("btn-decline"),
    promptForm: document.getElementById("prompt-form"),
    promptInput: document.getElementById("prompt-input"),
    rpcDrawer: document.getElementById("rpc-drawer"),
    btnToggleRpc: document.getElementById("btn-toggle-rpc"),
    btnClearRpc: document.getElementById("btn-clear-rpc"),
    btnCloseRpc: document.getElementById("btn-close-rpc"),
    rpcLog: document.getElementById("rpc-log"),
    rpcCustomForm: document.getElementById("rpc-custom-form"),
    rpcCustomInput: document.getElementById("rpc-custom-input"),
  };

  function nextId() {
    return state.rpcSeq++;
  }

  function wsBaseUrl() {
    const proto = window.location.protocol === "https:" ? "wss:" : "ws:";
    return `${proto}//${window.location.host}`;
  }

  function logRpc(direction, payload) {
    const div = document.createElement("div");
    div.className = `rpc-entry ${direction}`;
    const prefix = direction === "out" ? "→ OUT " : "← IN  ";
    div.textContent = prefix + (typeof payload === "string" ? payload : JSON.stringify(payload));
    el.rpcLog.appendChild(div);
    el.rpcLog.scrollTop = el.rpcLog.scrollHeight;
  }

  function appendMessage(role, title, text) {
    const welcome = el.messages.querySelector(".welcome-card");
    if (welcome) welcome.remove();

    const wrapper = document.createElement("div");
    wrapper.className = `msg msg-${role}`;

    const header = document.createElement("div");
    header.className = "msg-header";
    header.innerHTML = `<span>${escapeHtml(title)}</span><span>${new Date().toLocaleTimeString()}</span>`;

    const body = document.createElement("div");
    body.className = "msg-body";
    body.textContent = text;

    wrapper.appendChild(header);
    wrapper.appendChild(body);
    el.messages.appendChild(wrapper);
    el.messages.scrollTop = el.messages.scrollHeight;
    return body;
  }

  function getOrCreateStreamingBubble(agentKey, label) {
    const agentState = state[agentKey];
    if (!agentState.streamingEl) {
      agentState.streamingEl = appendMessage("agent", label, "");
    }
    return agentState.streamingEl;
  }

  function escapeHtml(str) {
    return String(str)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  function updateHeaderUI() {
    const isCodex = state.activeAgent === "codex";
    const current = isCodex ? state.codex : state.goose;

    if (current.connected) {
      el.connBadge.textContent = `Connecté (${isCodex ? "Codex" : "Goose"})`;
      el.connBadge.classList.add("connected");
    } else {
      el.connBadge.textContent = "Déconnecté";
      el.connBadge.classList.remove("connected");
    }

    const activeId = isCodex ? state.codex.activeThreadId : state.goose.activeSessionId;
    el.activeSessionDisplay.textContent = activeId
      ? `${isCodex ? "Thread" : "Session"}: ${activeId}`
      : "Aucune session active";

    if (current.turnRunning) {
      el.btnInterrupt.classList.remove("hidden");
    } else {
      el.btnInterrupt.classList.add("hidden");
    }

    renderSessionsList();
  }

  function renderSessionsList() {
    el.sessionList.innerHTML = "";
    if (state.activeAgent === "codex") {
      const threads = state.codex.threads;
      if (!threads.length) {
        el.sessionList.innerHTML = `<div class="empty-hint">Aucun thread Codex</div>`;
        return;
      }
      threads.forEach((t) => {
        const id = t.id || t.threadId || String(t);
        const preview = t.preview || t.title || t.cwd || "Thread Codex";
        const div = document.createElement("div");
        div.className = `session-item ${state.codex.activeThreadId === id ? "active" : ""}`;
        div.innerHTML = `
          <span class="session-item-id">${escapeHtml(id)}</span>
          <span class="session-item-preview">${escapeHtml(preview)}</span>
        `;
        div.addEventListener("click", () => resumeCodexThread(id));
        el.sessionList.appendChild(div);
      });
    } else {
      const sessions = state.goose.sessions;
      if (!sessions.length) {
        el.sessionList.innerHTML = `<div class="empty-hint">Aucune session Goose</div>`;
        return;
      }
      sessions.forEach((s) => {
        const id = s.id || s.sessionId || String(s);
        const preview = s.description || s.cwd || "Session Goose ACP";
        const div = document.createElement("div");
        div.className = `session-item ${state.goose.activeSessionId === id ? "active" : ""}`;
        div.innerHTML = `
          <span class="session-item-id">${escapeHtml(id)}</span>
          <span class="session-item-preview">${escapeHtml(preview)}</span>
        `;
        div.addEventListener("click", () => loadGooseSession(id));
        el.sessionList.appendChild(div);
      });
    }
  }

  // =========================================================================
  // CODEX APP-SERVER CLIENT (JSON-RPC 2.0 over WS or Unix Socket)
  // =========================================================================
  function sendCodexRpc(method, params = {}, isNotification = false) {
    if (!state.codex.ws || state.codex.ws.readyState !== WebSocket.OPEN) {
      appendMessage("system", "Codex", "WebSocket Codex non connecté.");
      return Promise.reject(new Error("Codex socket not open"));
    }
    const msg = { jsonrpc: "2.0", method, params };
    if (!isNotification) {
      msg.id = nextId();
    }
    const raw = JSON.stringify(msg);
    logRpc("out", msg);
    state.codex.ws.send(raw);

    if (isNotification) return Promise.resolve();
    return new Promise((resolve, reject) => {
      state.pendingRequests.set(msg.id, { resolve, reject, agent: "codex", method });
    });
  }

  function connectCodex() {
    if (state.codex.ws) {
      state.codex.ws.close();
    }
    const mode = el.codexMode.value;
    const target = el.codexTarget.value.trim();
    const params = new URLSearchParams({ mode });
    if (target) params.set("target", target);

    const url = `${wsBaseUrl()}/ws/codex?${params.toString()}`;
    appendMessage("system", "Codex", `Connexion au pont Rust (${mode})...`);

    const ws = new WebSocket(url);
    state.codex.ws = ws;

    ws.onopen = async () => {
      state.codex.connected = true;
      updateHeaderUI();
      try {
        await sendCodexRpc("initialize", {
          clientInfo: { name: "agent-hub", version: "1.0.0" },
          capabilities: {},
        });
        await sendCodexRpc("initialized", {}, true);
        appendMessage("system", "Codex", "Handshake JSON-RPC initialize/initialized réussi.");
        refreshCodexThreads();
      } catch (err) {
        appendMessage("system", "Erreur Codex", String(err.message || err));
      }
    };

    ws.onmessage = (ev) => {
      let msg;
      try {
        msg = JSON.parse(ev.data);
      } catch {
        logRpc("in", ev.data);
        return;
      }
      logRpc("in", msg);
      handleCodexFrame(msg);
    };

    ws.onclose = () => {
      state.codex.connected = false;
      state.codex.turnRunning = false;
      updateHeaderUI();
      appendMessage("system", "Codex", "Connexion fermée.");
    };

    ws.onerror = () => {
      appendMessage("system", "Erreur Codex", "Erreur sur le flux WebSocket.");
    };
  }

  function handleCodexFrame(msg) {
    // 1. Response to our request
    if (msg.id !== undefined && (msg.result !== undefined || msg.error !== undefined)) {
      const pending = state.pendingRequests.get(msg.id);
      if (pending) {
        state.pendingRequests.delete(msg.id);
        if (msg.error) {
          pending.reject(new Error(msg.error.message || JSON.stringify(msg.error)));
        } else {
          pending.resolve(msg.result);
        }
      }
      return;
    }

    // 2. Server-initiated JSON-RPC Request (e.g. Approval request)
    if (msg.id !== undefined && msg.method) {
      showApprovalModal("codex", msg.id, msg.method, msg.params);
      return;
    }

    // 3. Server Notification
    const method = msg.method || "";
    const params = msg.params || {};

    if (method === "turn/started") {
      state.codex.turnRunning = true;
      state.codex.streamingEl = null;
      updateHeaderUI();
    } else if (method === "turn/completed") {
      state.codex.turnRunning = false;
      state.codex.streamingEl = null;
      updateHeaderUI();
    } else if (method === "item/agentMessage/delta") {
      const bubble = getOrCreateStreamingBubble("codex", "Codex");
      bubble.textContent += params.delta || "";
      el.messages.scrollTop = el.messages.scrollHeight;
    } else if (
      method === "item/reasoning/summaryTextDelta" ||
      method === "item/reasoning/textDelta"
    ) {
      const bubble = getOrCreateStreamingBubble("codex", "Codex (Raisonnement)");
      bubble.textContent += params.delta || "";
      el.messages.scrollTop = el.messages.scrollHeight;
    } else if (method === "item/started") {
      const item = params.item || {};
      if (item.type === "commandExecution") {
        appendMessage("tool", "Codex Tool • Commande", `$ ${item.command || JSON.stringify(item)}`);
      } else if (item.type === "fileChange") {
        appendMessage("tool", "Codex Tool • Modification fichier", JSON.stringify(item.changes || item, null, 2));
      }
    } else if (method === "error" || method === " bridge/error") {
      appendMessage("system", "Codex", params.message || JSON.stringify(params));
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
      updateHeaderUI();
    } catch (err) {
      appendMessage("system", "Codex", `Impossible de lister les threads: ${err.message}`);
    }
  }

  async function startNewCodexThread() {
    if (!state.codex.connected) {
      appendMessage("system", "Codex", "Connectez-vous d'abord à Codex.");
      return null;
    }
    const cwd = el.codexCwd.value.trim() || "/srv";
    try {
      const res = await sendCodexRpc("thread/start", { cwd });
      const id = res?.thread?.id || res?.threadId || res?.id;
      if (id) {
        state.codex.activeThreadId = id;
        if (!state.codex.threads.find((t) => (t.id || t.threadId) === id)) {
          state.codex.threads.unshift({ id, cwd, preview: `Nouveau thread (${cwd})` });
        }
        updateHeaderUI();
        appendMessage("system", "Codex", `Nouveau thread démarré : ${id}`);
      }
      return id;
    } catch (err) {
      appendMessage("system", "Erreur Codex", `Échec thread/start: ${err.message}`);
      return null;
    }
  }

  async function resumeCodexThread(threadId) {
    state.codex.activeThreadId = threadId;
    updateHeaderUI();
    if (!state.codex.connected) return;
    try {
      await sendCodexRpc("thread/resume", { threadId });
      appendMessage("system", "Codex", `Thread repris : ${threadId}`);
    } catch (err) {
      appendMessage("system", "Codex", `Sélection du thread ${threadId} (${err.message})`);
    }
  }

  // =========================================================================
  // GOOSE SERVE ACP CLIENT (JSON-RPC 2.0 over WebSocket or HTTP)
  // =========================================================================
  async function sendGooseRpc(method, params = {}, isNotification = false) {
    const mode = el.gooseMode.value;
    const msg = { jsonrpc: "2.0", method, params };
    if (!isNotification) {
      msg.id = nextId();
    }
    logRpc("out", msg);

    if (mode === "http") {
      const headers = { "Content-Type": "application/json" };
      const token = el.gooseToken.value.trim();
      if (token) headers["X-Secret-Key"] = token;

      const resp = await fetch("/api/goose/acp", {
        method: "POST",
        headers,
        body: JSON.stringify(msg),
      });
      const data = await resp.json();
      logRpc("in", data);
      handleGooseFrame(data);
      if (data.error) throw new Error(data.error.message || JSON.stringify(data.error));
      return data.result;
    }

    if (!state.goose.ws || state.goose.ws.readyState !== WebSocket.OPEN) {
      appendMessage("system", "Goose", "WebSocket Goose non connecté.");
      throw new Error("Goose WebSocket not open");
    }
    state.goose.ws.send(JSON.stringify(msg));
    if (isNotification) return;
    return new Promise((resolve, reject) => {
      state.pendingRequests.set(msg.id, { resolve, reject, agent: "goose", method });
    });
  }

  async function connectGoose() {
    const mode = el.gooseMode.value;
    state.goose.mode = mode;

    if (mode === "http") {
      try {
        state.goose.connected = true;
        updateHeaderUI();
        await sendGooseRpc("initialize", {
          protocolVersion: 1,
          clientCapabilities: {
            fs: { readTextFile: true, writeTextFile: true },
            terminal: true,
          },
          clientInfo: { name: "agent-hub", version: "1.0.0" },
        });
        appendMessage("system", "Goose", "Connecté à Goose via HTTP JSON-RPC (/api/goose/acp).");
        await startNewGooseSession();
      } catch (err) {
        state.goose.connected = false;
        updateHeaderUI();
        appendMessage("system", "Erreur Goose HTTP", err.message);
      }
      return;
    }

    if (state.goose.ws) {
      state.goose.ws.close();
    }
    const target = el.gooseTarget.value.trim();
    const token = el.gooseToken.value.trim();
    const params = new URLSearchParams();
    if (target) params.set("target", target);
    if (token) params.set("token", token);

    const url = `${wsBaseUrl()}/ws/goose?${params.toString()}`;
    appendMessage("system", "Goose", "Connexion au pont WebSocket ACP Goose...");

    const ws = new WebSocket(url);
    state.goose.ws = ws;

    ws.onopen = async () => {
      state.goose.connected = true;
      updateHeaderUI();
      try {
        await sendGooseRpc("initialize", {
          protocolVersion: 1,
          clientCapabilities: {
            fs: { readTextFile: true, writeTextFile: true },
            terminal: true,
          },
          clientInfo: { name: "agent-hub", version: "1.0.0" },
        });
        appendMessage("system", "Goose", "Handshake ACP initialize réussi.");
        if (!state.goose.activeSessionId) {
          await startNewGooseSession();
        }
      } catch (err) {
        appendMessage("system", "Erreur Goose", String(err.message || err));
      }
    };

    ws.onmessage = (ev) => {
      let msg;
      try {
        msg = JSON.parse(ev.data);
      } catch {
        logRpc("in", ev.data);
        return;
      }
      logRpc("in", msg);
      handleGooseFrame(msg);
    };

    ws.onclose = () => {
      state.goose.connected = false;
      state.goose.turnRunning = false;
      updateHeaderUI();
      appendMessage("system", "Goose", "Connexion WebSocket Goose fermée.");
    };
  }

  function handleGooseFrame(msg) {
    if (msg.id !== undefined && (msg.result !== undefined || msg.error !== undefined)) {
      const pending = state.pendingRequests.get(msg.id);
      if (pending) {
        state.pendingRequests.delete(msg.id);
        if (msg.error) {
          pending.reject(new Error(msg.error.message || JSON.stringify(msg.error)));
        } else {
          pending.resolve(msg.result);
        }
      }
      return;
    }

    if (msg.id !== undefined && msg.method) {
      showApprovalModal("goose", msg.id, msg.method, msg.params);
      return;
    }

    const method = msg.method || "";
    const params = msg.params || {};

    if (method === "session/update") {
      const update = params.update || {};
      const kind = update.sessionUpdate;
      if (kind === "agent_message_chunk") {
        const text = update.content?.text || "";
        const bubble = getOrCreateStreamingBubble("goose", "Goose");
        bubble.textContent += text;
        el.messages.scrollTop = el.messages.scrollHeight;
      } else if (kind === "agent_thought_chunk") {
        const text = update.content?.text || "";
        const bubble = getOrCreateStreamingBubble("goose", "Goose (Pensée)");
        bubble.textContent += text;
        el.messages.scrollTop = el.messages.scrollHeight;
      } else if (kind === "tool_call") {
        appendMessage("tool", `Goose Tool • ${update.title || "Outil"}`, JSON.stringify(update, null, 2));
      }
    } else if (method === "bridge/error") {
      appendMessage("system", "Goose", params.message || JSON.stringify(params));
    }
  }

  async function startNewGooseSession() {
    if (!state.goose.connected) {
      appendMessage("system", "Goose", "Connectez-vous d'abord à Goose.");
      return null;
    }
    const cwd = el.gooseCwd.value.trim() || "/srv";
    try {
      const res = await sendGooseRpc("session/new", { cwd, mcpServers: [] });
      const id = res?.sessionId || res?.id;
      if (id) {
        state.goose.activeSessionId = id;
        if (!state.goose.sessions.find((s) => s.id === id)) {
          state.goose.sessions.unshift({ id, cwd });
        }
        updateHeaderUI();
        appendMessage("system", "Goose", `Session ACP créée : ${id}`);
      }
      return id;
    } catch (err) {
      appendMessage("system", "Erreur Goose", `Échec session/new: ${err.message}`);
      return null;
    }
  }

  async function loadGooseSession(sessionId) {
    state.goose.activeSessionId = sessionId;
    updateHeaderUI();
    const cwd = el.gooseCwd.value.trim() || "/srv";
    try {
      await sendGooseRpc("session/load", { sessionId, cwd, mcpServers: [] });
      appendMessage("system", "Goose", `Session chargée : ${sessionId}`);
    } catch (err) {
      appendMessage("system", "Goose", `Session sélectionnée : ${sessionId}`);
    }
  }

  // =========================================================================
  // APPROVAL REQUESTS & PROMPT SUBMISSION
  // =========================================================================
  function showApprovalModal(agent, requestId, method, params) {
    state.pendingApproval = { agent, requestId, method, params };
    el.approvalDetails.textContent = `${method}\n${JSON.stringify(params, null, 2)}`;
    el.approvalBanner.classList.remove("hidden");
  }

  function respondApproval(approved) {
    if (!state.pendingApproval) return;
    const { agent, requestId, params } = state.pendingApproval;
    state.pendingApproval = null;
    el.approvalBanner.classList.add("hidden");

    if (agent === "codex") {
      const reply = {
        jsonrpc: "2.0",
        id: requestId,
        result: { decision: approved ? "accept" : "decline" },
      };
      logRpc("out", reply);
      state.codex.ws?.send(JSON.stringify(reply));
    } else {
      const options = params?.options || [];
      const allowOpt = options.find((o) => o.kind?.startsWith("allow")) || options[0];
      const reply = {
        jsonrpc: "2.0",
        id: requestId,
        result: approved
          ? { outcome: { outcome: "selected", optionId: allowOpt?.optionId || "allow" } }
          : { outcome: { outcome: "cancelled" } },
      };
      logRpc("out", reply);
      state.goose.ws?.send(JSON.stringify(reply));
    }
  }

  async function handlePromptSubmit(e) {
    e.preventDefault();
    const text = el.promptInput.value.trim();
    if (!text) return;
    el.promptInput.value = "";

    appendMessage("user", "Vous", text);

    if (state.activeAgent === "codex") {
      if (!state.codex.connected) {
        appendMessage("system", "Codex", "Veuillez d'abord cliquer sur Connecter.");
        return;
      }
      let threadId = state.codex.activeThreadId;
      if (!threadId) {
        threadId = await startNewCodexThread();
        if (!threadId) return;
      }
      state.codex.streamingEl = null;
      state.codex.turnRunning = true;
      updateHeaderUI();
      try {
        await sendCodexRpc("turn/start", {
          threadId,
          input: [{ type: "text", text }],
        });
      } catch (err) {
        state.codex.turnRunning = false;
        updateHeaderUI();
        appendMessage("system", "Erreur Codex", err.message);
      }
    } else {
      if (!state.goose.connected) {
        appendMessage("system", "Goose", "Veuillez d'abord cliquer sur Connecter.");
        return;
      }
      let sessionId = state.goose.activeSessionId;
      if (!sessionId) {
        sessionId = await startNewGooseSession();
        if (!sessionId) return;
      }
      state.goose.streamingEl = null;
      state.goose.turnRunning = true;
      updateHeaderUI();
      try {
        const res = await sendGooseRpc("session/prompt", {
          sessionId,
          prompt: [{ type: "text", text }],
        });
        state.goose.turnRunning = false;
        state.goose.streamingEl = null;
        updateHeaderUI();
        if (res?.text) {
          appendMessage("agent", "Goose", res.text);
        }
      } catch (err) {
        state.goose.turnRunning = false;
        updateHeaderUI();
        appendMessage("system", "Erreur Goose", err.message);
      }
    }
  }

  async function interruptActiveTurn() {
    if (state.activeAgent === "codex" && state.codex.activeThreadId) {
      await sendCodexRpc("turn/interrupt", { threadId: state.codex.activeThreadId });
    } else if (state.activeAgent === "goose" && state.goose.activeSessionId) {
      await sendGooseRpc("session/cancel", { sessionId: state.goose.activeSessionId }, true);
      state.goose.turnRunning = false;
      updateHeaderUI();
    }
  }

  // =========================================================================
  // SERVER CONFIG & HEALTH PROBE
  // =========================================================================
  async function loadServerConfig() {
    try {
      const resp = await fetch("/api/config");
      const cfg = await resp.json();
      if (cfg.default_cwd) {
        el.codexCwd.value = cfg.default_cwd;
        el.gooseCwd.value = cfg.default_cwd;
      }
      if (cfg.codex_ws_url) {
        el.codexTarget.placeholder = cfg.codex_ws_url;
      }
      if (cfg.goose_ws_url) {
        el.gooseTarget.placeholder = cfg.goose_ws_url;
      }
      if (cfg.codex_unix_available) {
        el.codexMode.value = "unix";
      }
      if (cfg.has_goose_secret) {
        el.gooseToken.placeholder = "Clé configurée côté serveur Rust ✓";
      }
    } catch {
      // ignore
    }
  }

  async function probeStatus() {
    try {
      const resp = await fetch("/api/status");
      const st = await resp.json();
      const codexOk = st.codex_ws?.online || st.codex_unix?.online;
      const gooseOk = st.goose?.online;

      el.dotCodex.className = `status-dot ${codexOk ? "online" : "offline"}`;
      el.dotGoose.className = `status-dot ${gooseOk ? "online" : "offline"}`;
    } catch {
      el.dotCodex.className = "status-dot offline";
      el.dotGoose.className = "status-dot offline";
    }
  }

  function switchAgentTab(agent) {
    state.activeAgent = agent;
    const isCodex = agent === "codex";
    el.tabCodex.classList.toggle("active", isCodex);
    el.tabGoose.classList.toggle("active", !isCodex);
    el.controlsCodex.classList.toggle("hidden", !isCodex);
    el.controlsGoose.classList.toggle("hidden", isCodex);
    el.configTitle.textContent = isCodex ? "Connexion Codex" : "Connexion Goose";
    el.sessionsLabel.textContent = isCodex ? "Threads Codex" : "Sessions Goose";
    updateHeaderUI();
  }

  // Event Listeners
  el.tabCodex.addEventListener("click", () => switchAgentTab("codex"));
  el.tabGoose.addEventListener("click", () => switchAgentTab("goose"));
  el.btnProbe.addEventListener("click", probeStatus);
  el.btnCodexConnect.addEventListener("click", connectCodex);
  el.btnCodexNew.addEventListener("click", startNewCodexThread);
  el.btnGooseConnect.addEventListener("click", connectGoose);
  el.btnGooseNew.addEventListener("click", startNewGooseSession);
  el.btnRefreshSessions.addEventListener("click", () => {
    if (state.activeAgent === "codex") refreshCodexThreads();
    else renderSessionsList();
  });
  el.btnInterrupt.addEventListener("click", interruptActiveTurn);
  el.btnClearChat.addEventListener("click", () => {
    el.messages.innerHTML = "";
  });
  el.btnApprove.addEventListener("click", () => respondApproval(true));
  el.btnDecline.addEventListener("click", () => respondApproval(false));
  el.promptForm.addEventListener("submit", handlePromptSubmit);
  el.promptInput.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handlePromptSubmit(e);
    }
  });
  el.btnToggleRpc.addEventListener("click", () => {
    el.rpcDrawer.classList.toggle("hidden");
  });
  el.btnCloseRpc.addEventListener("click", () => {
    el.rpcDrawer.classList.add("hidden");
  });
  el.btnClearRpc.addEventListener("click", () => {
    el.rpcLog.innerHTML = "";
  });
  el.rpcCustomForm.addEventListener("submit", (e) => {
    e.preventDefault();
    const raw = el.rpcCustomInput.value.trim();
    if (!raw) return;
    el.rpcCustomInput.value = "";
    logRpc("out", raw);
    if (state.activeAgent === "codex") {
      state.codex.ws?.send(raw);
    } else {
      state.goose.ws?.send(raw);
    }
  });

  // Init
  loadServerConfig().then(probeStatus);
  setInterval(probeStatus, 10000);
})();
