(() => {
    let currentLang = "en";

    const I18N = {
        en: {
            appTitle: "Simply Agent Hub",
            registeredClis: (count, online) => `${count} CLIs (${online} online)`,
            noCliSelected: "No CLI selected",
            activeCliReady: "Ready",
            btnRefreshAll: "↻ Refresh Status",
            tabConsole: "CLI Console, Roles & Native Terminal",
            tabOrchestration: "Multi-CLI Orchestration Channels",
            tabGit: "🔀 Git Diff & Commit",
            tabInventory: "Container Fleet (SeaORM)",
            tabRpc: "JSON-RPC Inspector",
            cliSelectorTitle: "CLI Container Selection",
            cliSelectorSubtitle: "Select, rename, assign a role, or authenticate any of your CLI instances.",
            activeCliLabel: "Active CLI Instance",
            btnAddCli: "+ Add CLI",
            btnEditCli: "✎ Rename / Edit",
            btnAuthCli: "🔑 Authenticate",
            agentRoleTitle: "🎭 Agent Role & Mission",
            roleInCollectiveLabel: "Role in collective",
            rolePromptLabel: "Role Prompt (suggested default / customizable)",
            btnSaveRole: "💾 Save Role",
            btnOnboardCli: "🎭 Initialize Agent",
            addCliFormTitle: "Add CLI Container",
            editCliFormTitle: (name) => `Rename / Edit "${name}"`,
            customNameLabel: "Custom Name",
            cliAgentTypeLabel: "CLI Agent Kind",
            defaultRoleLabel: "Default Role",
            bridgeUrlLabel: "Auth/Tmux Bridge URL (optional)",
            secretTokenLabel: "Secret / Token (optional)",
            workingDirLabel: "Working Directory (cwd)",
            btnSaveDatabase: "Save to Database",
            btnCancel: "Cancel",
            authPanelTitle: "Container Web Authentication",
            authPanelSubtitle: "Authenticate the selected container without leaving this dashboard.",
            btnCheckStatus: "Check Status",
            btnGetDeviceCode: "Get Device Code / OAuth Link",
            apiKeyInjectLabel: "Or inject an API key (OpenAI / Gemini / Anthropic)",
            btnApplyKey: "Apply Key",
            oauthCallbackLabel: "Or validate OAuth Callback URL (http://localhost:...)",
            btnValidateOauth: "Validate OAuth",
            btnConnectCli: "Connect to CLI",
            btnNewSession: "+ Session / Thread",
            btnNativeTerminal: "🖥 Native Terminal (Tmux/WS)",
            btnCliWebServer: "🌐 CLI Web Server",
            sessionsThreadsTitle: "Sessions / Threads",
            selectConnectHint: "Select and connect a CLI instance",
            btnInterrupt: "⏹ Interrupt",
            btnClear: "Clear",
            termDrawerTitle: "🖥 Interactive Native Terminal (Direct Tmux WebSocket)",
            btnClose: "✕ Close",
            btnSendPty: "Send to PTY ➤",
            approvalTitle: "Tool approval required by CLI",
            btnApprove: "Approve",
            btnDecline: "Decline",
            btnSend: "Send ➤",
            orchChannelsTitle: "Orchestration Channels",
            orchChannelsSubtitle: "Create a shared channel where multiple CLI instances (with distinct assigned roles) collaborate in real-time.",
            activeChannelLabel: "Active Channel",
            btnNewChannel: "+ New Channel",
            btnDelete: "🗑 Delete",
            createChannelTitle: "Create Channel",
            channelNameLabel: "Channel Name",
            descriptionLabel: "Description",
            btnCreate: "Create",
            clisJoinedChannelTitle: "CLIs in this Channel",
            clisJoinedChannelHint: "Check the instances that participate in this channel's exchanges:",
            btnSaveParticipants: "💾 Save Channel Participants",
            retentionTitle: "🧹 Configurable Retention (SeaORM)",
            retentionDaysLabel: "Retention (days, 0 = unlimited)",
            maxMessagesLabel: "Max message count (0 = unlimited)",
            btnSaveRetention: "💾 Apply & Prune",
            btnExportMd: "📥 Export .md",
            btnRefreshHistory: "↻ DB History",
            btnBroadcast: "Broadcast ➤",
            gitStatusTitle: "🔀 Workspace Git Status",
            gitStatusSubtitle: "Inspect changes produced by your CLI agents, revert a file, or create a commit.",
            gitCwdLabel: "Git Directory (cwd)",
            btnAnalyze: "↻ Analyze",
            activeBranchLabel: "Active branch: ",
            modifiedFilesTitle: "Modified Files",
            createCommitTitle: "Create Commit",
            commitMsgLabel: "Commit Message",
            btnGitCommit: "✅ Stage All (git add -A) & Commit",
            recentCommitsTitle: "Recent Commits",
            unifiedDiffTitle: "Unified Diff (`git diff HEAD`)",
            btnRevertAll: "↩ Revert entire workspace",
            inventoryTitle: "Registered CLI Container Fleet (SeaORM: SQLite / Postgres / MariaDB)",
            inventorySubtitle: "Manage all your Codex, Antigravity, Claude, and Goose instances persisted in the database.",
            btnNewCliContainer: "+ New CLI Container",
            thInstanceName: "Instance Name",
            thType: "Type",
            thRole: "Assigned Role",
            thTransport: "Transport",
            thEndpoint: "Endpoint",
            thStatus: "Status",
            thActions: "Actions",
            rpcTitle: "JSON-RPC 2.0 Frame Inspector & Orchestration Bus",
            rpcSubtitle: "Real-time log of frames exchanged with CLI containers.",
            btnClearLog: "Clear Log",
            onlineBadge: "Online",
            offlineBadge: "Offline",
            btnUse: "Use",
            btnRename: "✎ Rename",
            connectingTo: (name) => `Connecting to ${name}...`,
            connectedTo: (name) => `Connected (${name} — Tmux PTY)`,
            disconnected: "Disconnected",
            tmuxSessionReady: "(Tmux session ready)",
            cleanWorkingTree: "Clean working tree (0 modified files)",
            noDiffDetected: "No uncommitted changes in this repository.",
            roleSaved: (role) => `Role "${role}" and system prompt saved in SeaORM database.`,
            onboardingSent: (role, name) => `Sending identity and role prompt (${role}) to ${name}...`,
            onboardSuccess: (role) => `Role "${role}" initialized.`,
            participantsUpdated: "Channel participants updated in SeaORM database.",
            retentionApplied: (days, max, pruned) => `Retention applied (${days}d / max ${max} messages). Pruned: ${pruned}.`,
            btnCompactHeader: "🗕 Header",
            btnExpandHeader: "🗖 Header",
            btnToggleSidebar: "◀ Sidebar",
            btnExpandSidebar: "▶ Sidebar",
            btnMaximizeTerm: "🗖 Maximize",
            btnRestoreTerm: "🗗 Restore",
        },
        fr: {
            appTitle: "Simply Agent Hub",
            registeredClis: (count, online) => `${count} CLIs (${online} en ligne)`,
            noCliSelected: "Aucun CLI sélectionné",
            activeCliReady: "Prêt",
            btnRefreshAll: "↻ Actualiser l'état",
            tabConsole: "Console CLI, Rôles & Terminal Natif",
            tabOrchestration: "Canaux d'Orchestration Multi-CLIs",
            tabGit: "🔀 Git Diff & Commit",
            tabInventory: "Parc de Conteneurs (SeaORM)",
            tabRpc: "Inspecteur JSON-RPC",
            cliSelectorTitle: "Sélection du Conteneur CLI",
            cliSelectorSubtitle: "Choisissez, renommez, attribuez un rôle ou authentifiez n'importe laquelle de vos instances.",
            activeCliLabel: "Instance CLI active",
            btnAddCli: "+ Ajouter un CLI",
            btnEditCli: "✎ Renommer / Éditer",
            btnAuthCli: "🔑 Authentifier",
            agentRoleTitle: "🎭 Rôle & Mission de l'Agent",
            roleInCollectiveLabel: "Rôle dans le collectif",
            rolePromptLabel: "Prompt de rôle (par défaut proposé / paramétrable)",
            btnSaveRole: "💾 Sauver le rôle",
            btnOnboardCli: "🎭 Initialiser l'agent",
            addCliFormTitle: "Ajouter un conteneur CLI",
            editCliFormTitle: (name) => `Renommer / Modifier « ${name} »`,
            customNameLabel: "Nom personnalisé",
            cliAgentTypeLabel: "Type d'agent CLI",
            defaultRoleLabel: "Rôle par défaut",
            bridgeUrlLabel: "URL du Pont d'Auth/Tmux (optionnel)",
            secretTokenLabel: "Secret / Token (optionnel)",
            workingDirLabel: "Répertoire de travail (cwd)",
            btnSaveDatabase: "Enregistrer en base",
            btnCancel: "Annuler",
            authPanelTitle: "Authentification Web du Conteneur",
            authPanelSubtitle: "Authentifiez le conteneur sélectionné sans quitter cette interface.",
            btnCheckStatus: "Vérifier statut",
            btnGetDeviceCode: "Obtenir Code Device / Lien OAuth",
            apiKeyInjectLabel: "Ou injecter une clé API (OpenAI / Gemini / Anthropic)",
            btnApplyKey: "Appliquer la clé",
            oauthCallbackLabel: "Ou valider une URL Callback OAuth (http://localhost:...)",
            btnValidateOauth: "Valider OAuth",
            btnConnectCli: "Connecter au CLI",
            btnNewSession: "+ Session / Thread",
            btnNativeTerminal: "🖥 Terminal Natif (Tmux/WS)",
            btnCliWebServer: "🌐 Serveur Web du CLI",
            sessionsThreadsTitle: "Sessions / Threads",
            selectConnectHint: "Sélectionnez et connectez un CLI",
            btnInterrupt: "⏹ Interrompre",
            btnClear: "Effacer",
            termDrawerTitle: "🖥 Terminal Natif Interactif (Tmux Direct WebSocket)",
            btnClose: "✕ Fermer",
            btnSendPty: "Envoyer au PTY ➤",
            approvalTitle: "Autorisation d'outil requise par le CLI",
            btnApprove: "Autoriser",
            btnDecline: "Refuser",
            btnSend: "Envoyer ➤",
            orchChannelsTitle: "Canaux d'Orchestration",
            orchChannelsSubtitle: "Créez un canal partagé où plusieurs instances CLI (avec leurs rôles respectifs) échangent des messages en temps réel.",
            activeChannelLabel: "Canal actif",
            btnNewChannel: "+ Nouveau Canal",
            btnDelete: "🗑 Supprimer",
            createChannelTitle: "Créer un canal",
            channelNameLabel: "Nom du canal",
            descriptionLabel: "Description",
            btnCreate: "Créer",
            clisJoinedChannelTitle: "CLIs connectés à ce canal",
            clisJoinedChannelHint: "Cochez les instances qui participent aux échanges de ce canal :",
            btnSaveParticipants: "💾 Enregistrer les participants du canal",
            retentionTitle: "🧹 Rétention Paramétrable (SeaORM)",
            retentionDaysLabel: "Rétention (jours, 0 = illimité)",
            maxMessagesLabel: "Nombre max de messages conservés (0 = illimité)",
            btnSaveRetention: "💾 Appliquer & Purger",
            btnExportMd: "📥 Export .md",
            btnRefreshHistory: "↻ Historique DB",
            btnBroadcast: "Diffuser ➤",
            gitStatusTitle: "🔀 État Git du Workspace",
            gitStatusSubtitle: "Inspectez les modifications produites par vos agents CLI, annulez un fichier ou créez un commit.",
            gitCwdLabel: "Répertoire Git (cwd)",
            btnAnalyze: "↻ Analyser",
            activeBranchLabel: "Branche active : ",
            modifiedFilesTitle: "Fichiers modifiés",
            createCommitTitle: "Créer un Commit",
            commitMsgLabel: "Message de commit",
            btnGitCommit: "✅ Tout indexer (git add -A) & Commit",
            recentCommitsTitle: "Derniers Commits",
            unifiedDiffTitle: "Diff Unifié (`git diff HEAD`)",
            btnRevertAll: "↩ Revert tout le workspace",
            inventoryTitle: "Parc de Conteneurs CLI enregistrés (SeaORM : SQLite / Postgres / MariaDB)",
            inventorySubtitle: "Gérez l'ensemble de vos instances Codex, Antigravity, Claude et Goose persistées en base de données.",
            btnNewCliContainer: "+ Nouveau Conteneur CLI",
            thInstanceName: "Nom de l'instance",
            thType: "Type",
            thRole: "Rôle attribué",
            thTransport: "Transport",
            thEndpoint: "Endpoint",
            thStatus: "État",
            thActions: "Actions",
            rpcTitle: "Inspecteur de trames JSON-RPC 2.0 & Bus d'Orchestration",
            rpcSubtitle: "Journal temps réel des échanges avec les conteneurs CLI.",
            btnClearLog: "Vider le journal",
            onlineBadge: "En ligne",
            offlineBadge: "Hors ligne",
            btnUse: "Utiliser",
            btnRename: "✎ Renommer",
            connectingTo: (name) => `Connexion à ${name}...`,
            connectedTo: (name) => `Connecté (${name} — Tmux PTY)`,
            disconnected: "Déconnecté",
            tmuxSessionReady: "(Session tmux prête)",
            cleanWorkingTree: "Arbre de travail propre (0 fichier modifié)",
            noDiffDetected: "Aucune modification non commitée dans ce dépôt.",
            roleSaved: (role) => `Rôle « ${role} » et prompt système enregistrés en base SeaORM.`,
            onboardingSent: (role, name) => `Envoi du prompt d'identité et de rôle (${role}) à ${name}...`,
            onboardSuccess: (role) => `Rôle « ${role} » initialisé.`,
            participantsUpdated: "Participants du canal mis à jour en base SeaORM.",
            retentionApplied: (days, max, pruned) => `Politique appliquée (${days}j / max ${max} messages). Messages purgés: ${pruned}.`,
            btnCompactHeader: "🗕 En-tête",
            btnExpandHeader: "🗖 En-tête",
            btnToggleSidebar: "◀ Menu",
            btnExpandSidebar: "▶ Menu",
            btnMaximizeTerm: "🗖 Agrandir",
            btnRestoreTerm: "🗗 Réduire",
        },
        de: {
            appTitle: "Simply Agent Hub",
            registeredClis: (count, online) => `${count} CLIs (${online} online)`,
            noCliSelected: "Kein CLI ausgewählt",
            activeCliReady: "Bereit",
            btnRefreshAll: "↻ Status aktualisieren",
            tabConsole: "CLI-Konsole, Rollen & Natives Terminal",
            tabOrchestration: "Multi-CLI Orchestrierungskanäle",
            tabGit: "🔀 Git Diff & Commit",
            tabInventory: "Container-Flotte (SeaORM)",
            tabRpc: "JSON-RPC Inspektor",
            cliSelectorTitle: "CLI-Container-Auswahl",
            cliSelectorSubtitle: "Wählen, benennen, weisen Sie eine Rolle zu oder authentifizieren Sie eine Ihrer CLI-Instanzen.",
            activeCliLabel: "Aktive CLI-Instanz",
            btnAddCli: "+ CLI hinzufügen",
            btnEditCli: "✎ Umbenennen / Bearbeiten",
            btnAuthCli: "🔑 Authentifizieren",
            agentRoleTitle: "🎭 Agentenrolle & Mission",
            roleInCollectiveLabel: "Rolle im Kollektiv",
            rolePromptLabel: "Rollen-Prompt (vorgeschlagen / anpassbar)",
            btnSaveRole: "💾 Rolle speichern",
            btnOnboardCli: "🎭 Agent initialisieren",
            addCliFormTitle: "CLI-Container hinzufügen",
            editCliFormTitle: (name) => `"${name}" umbenennen / bearbeiten`,
            customNameLabel: "Benutzerdefinierter Name",
            cliAgentTypeLabel: "CLI-Agententyp",
            defaultRoleLabel: "Standardrolle",
            bridgeUrlLabel: "Auth/Tmux Bridge-URL (optional)",
            secretTokenLabel: "Geheimnis / Token (optional)",
            workingDirLabel: "Arbeitsverzeichnis (cwd)",
            btnSaveDatabase: "In Datenbank speichern",
            btnCancel: "Abbrechen",
            authPanelTitle: "Web-Authentifizierung des Containers",
            authPanelSubtitle: "Authentifizieren Sie den ausgewählten Container direkt über dieses Dashboard.",
            btnCheckStatus: "Status prüfen",
            btnGetDeviceCode: "Device Code / OAuth-Link abrufen",
            apiKeyInjectLabel: "Oder API-Schlüssel einfügen (OpenAI / Gemini / Anthropic)",
            btnApplyKey: "Schlüssel anwenden",
            oauthCallbackLabel: "Oder OAuth-Callback-URL validieren (http://localhost:...)",
            btnValidateOauth: "OAuth validieren",
            btnConnectCli: "Mit CLI verbinden",
            btnNewSession: "+ Sitzung / Thread",
            btnNativeTerminal: "🖥 Natives Terminal (Tmux/WS)",
            btnCliWebServer: "🌐 CLI-Webserver",
            sessionsThreadsTitle: "Sitzungen / Threads",
            selectConnectHint: "Wählen und verbinden Sie eine CLI-Instanz",
            btnInterrupt: "⏹ Unterbrechen",
            btnClear: "Leeren",
            termDrawerTitle: "🖥 Interaktives natives Terminal (Direktes Tmux WebSocket)",
            btnClose: "✕ Schließen",
            btnSendPty: "An PTY senden ➤",
            approvalTitle: "Werkzeuggenehmigung vom CLI angefordert",
            btnApprove: "Genehmigen",
            btnDecline: "Ablehnen",
            btnSend: "Senden ➤",
            orchChannelsTitle: "Orchestrierungskanäle",
            orchChannelsSubtitle: "Erstellen Sie einen geteilten Kanal, in dem mehrere CLI-Instanzen in Echtzeit zusammenarbeiten.",
            activeChannelLabel: "Aktiver Kanal",
            btnNewChannel: "+ Neuer Kanal",
            btnDelete: "🗑 Löschen",
            createChannelTitle: "Kanal erstellen",
            channelNameLabel: "Kanalname",
            descriptionLabel: "Beschreibung",
            btnCreate: "Erstellen",
            clisJoinedChannelTitle: "CLIs in diesem Kanal",
            clisJoinedChannelHint: "Wählen Sie die Instanzen aus, die an diesem Kanal teilnehmen:",
            btnSaveParticipants: "💾 Kanalteilnehmer speichern",
            retentionTitle: "🧹 Konfigurierbare Aufbewahrung (SeaORM)",
            retentionDaysLabel: "Aufbewahrung (Tage, 0 = unbegrenzt)",
            maxMessagesLabel: "Max. Nachrichtenanzahl (0 = unbegrenzt)",
            btnSaveRetention: "💾 Anwenden & Bereinigen",
            btnExportMd: "📥 Export .md",
            btnRefreshHistory: "↻ DB-Verlauf",
            btnBroadcast: "Senden ➤",
            gitStatusTitle: "🔀 Workspace Git-Status",
            gitStatusSubtitle: "Überprüfen Sie Änderungen Ihrer CLI-Agenten, setzen Sie Dateien zurück oder erstellen Sie Commits.",
            gitCwdLabel: "Git-Verzeichnis (cwd)",
            btnAnalyze: "↻ Analysieren",
            activeBranchLabel: "Aktiver Branch: ",
            modifiedFilesTitle: "Geänderte Dateien",
            createCommitTitle: "Commit erstellen",
            commitMsgLabel: "Commit-Nachricht",
            btnGitCommit: "✅ Alles stagen (git add -A) & Committen",
            recentCommitsTitle: "Neueste Commits",
            unifiedDiffTitle: "Unified Diff (`git diff HEAD`)",
            btnRevertAll: "↩ Gesamten Workspace zurücksetzen",
            inventoryTitle: "Registrierte CLI-Container-Flotte (SeaORM: SQLite / Postgres / MariaDB)",
            inventorySubtitle: "Verwalten Sie alle in der Datenbank gespeicherten Codex-, Antigravity-, Claude- und Goose-Instanzen.",
            btnNewCliContainer: "+ Neuer CLI-Container",
            thInstanceName: "Instanzname",
            thType: "Typ",
            thRole: "Zugewiesene Rolle",
            thTransport: "Transport",
            thEndpoint: "Endpunkt",
            thStatus: "Status",
            thActions: "Aktionen",
            rpcTitle: "JSON-RPC 2.0 Frame-Inspektor & Orchestrierungsbus",
            rpcSubtitle: "Echtzeitprotokoll der mit CLI-Containern ausgetauschten Frames.",
            btnClearLog: "Protokoll leeren",
            onlineBadge: "Online",
            offlineBadge: "Offline",
            btnUse: "Verwenden",
            btnRename: "✎ Umbenennen",
            connectingTo: (name) => `Verbinde mit ${name}...`,
            connectedTo: (name) => `Verbunden (${name} — Tmux PTY)`,
            disconnected: "Getrennt",
            tmuxSessionReady: "(Tmux-Sitzung bereit)",
            cleanWorkingTree: "Sauberer Arbeitsbaum (0 geänderte Dateien)",
            noDiffDetected: "Keine ungespeicherten Änderungen in diesem Repository.",
            roleSaved: (role) => `Rolle "${role}" und Systemprompt in SeaORM-Datenbank gespeichert.`,
            onboardingSent: (role, name) => `Sende Identität und Rollen-Prompt (${role}) an ${name}...`,
            onboardSuccess: (role) => `Rolle "${role}" initialisiert.`,
            participantsUpdated: "Kanalteilnehmer in SeaORM-Datenbank aktualisiert.",
            retentionApplied: (days, max, pruned) => `Aufbewahrung angewendet (${days} Tage / max. ${max} Nachrichten). Bereinigt: ${pruned}.`,
            btnCompactHeader: "🗕 Kopfzeile",
            btnExpandHeader: "🗖 Kopfzeile",
            btnToggleSidebar: "◀ Menü",
            btnExpandSidebar: "▶ Menü",
            btnMaximizeTerm: "🗖 Maximieren",
            btnRestoreTerm: "🗗 Verkleinern",
        }
    };

    function t(key, ...args) {
        const dict = I18N[currentLang] || I18N.en;
        const entry = dict[key] ?? I18N.en[key] ?? key;
        if (typeof entry === "function") {
            return entry(...args);
        }
        return entry;
    }

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

    window.switchTab = function (target) {
        document.querySelectorAll(".tab-btn").forEach((b) => {
            const isMatch = b.getAttribute("data-tab") === target;
            b.classList.toggle("active", isMatch);
            b.setAttribute("aria-selected", isMatch ? "true" : "false");
        });
        document.querySelectorAll(".tab-panel").forEach((panel) => {
            panel.classList.toggle("active", panel.id === `tab-${target}`);
        });
        if (target === "git") {
            loadGitStatus();
        }
    };

    window.switchView = window.switchTab;

    // ─── Tab Navigation ──────────────────────────────────────────────────────
    document.querySelectorAll(".tab-btn").forEach((btn) => {
        btn.addEventListener("click", () => {
            const target = btn.getAttribute("data-tab");
            window.switchTab(target);
        });
    });

    // ═════════════════════════════════════════════════════════════════════════
    // 1. CLI INSTANCES, ROLES & SEAORM CRUD
    // ═════════════════════════════════════════════════════════════════════════
    async function loadRolePresets() {
        try {
            const resp = await fetch(`/api/roles/presets?lang=${encodeURIComponent(currentLang)}`);
            const list = await resp.json();
            if (Array.isArray(list)) {
                state.rolePresets = {};
                list.forEach((r) => {
                    state.rolePresets[r.id] = r;
                });
                renderRoleOptions();
            }
        } catch {
            // ignore
        }
    }

    function renderRoleOptions() {
        const sel1 = $("cli-role-selector");
        const sel2 = $("cli-form-role");
        const currentVal = sel1 ? sel1.value : "developer";

        const opts = Object.values(state.rolePresets).map(
            (p) => `<option value="${escapeHtml(p.id)}">${escapeHtml(p.label)}</option>`
        ).join("");

        if (opts) {
            if (sel1) {
                sel1.innerHTML = opts;
                sel1.value = currentVal;
            }
            if (sel2) {
                sel2.innerHTML = opts;
            }
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
            appendMsg("console-messages", "system", "Error", `Unable to load instances: ${err.message}`);
        }
    }

    function renderCliSelector() {
        const sel = $("cli-selector");
        if (!sel) return;
        sel.innerHTML = "";
        const onlineCount = state.instances.filter((i) => i.online).length;
        $("header-instances-count").textContent = t("registeredClis", state.instances.length, onlineCount);

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
            headerBadge.textContent = t("noCliSelected");
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
                t("roleSaved", role)
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
            "Role Onboarding",
            t("onboardingSent", role, inst.name)
        );
        try {
            const resp = await fetch(`/api/instances/${inst.id}/onboard`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ custom_prompt: role_prompt, lang: currentLang }),
            });
            const data = await resp.json();
            logRpc("in", `onboard:${inst.name}`, data);
            if (data.error) throw new Error(data.error);
            appendMsg("console-messages", "agent", `${inst.name} [${data.role}]`, data.reply || t("onboardSuccess", data.role));
        } catch (err) {
            appendMsg("console-messages", "system", inst.name, `Onboarding error: ${err.message}`);
        }
    });

    function renderInventoryTable() {
        const tbodies = [ $("inventory-tbody"), $("instances-tbody") ].filter(Boolean);
        if (tbodies.length === 0) return;

        tbodies.forEach((tbody) => {
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
                            ${inst.online ? t("onlineBadge") : t("offlineBadge")}
                        </span>
                    </td>
                    <td>
                        <div class="btn-row" style="margin-top:0;">
                            <button class="btn btn-secondary btn-sm" data-action="select" data-id="${inst.id}">${t("btnUse")}</button>
                            <button class="btn btn-secondary btn-sm" data-action="edit" data-id="${inst.id}">${t("btnRename")}</button>
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
                        window.switchTab("console");
                    } else if (act === "edit") {
                        window.switchTab("console");
                        openEditCliForm();
                    } else if (act === "delete") {
                        await deleteActiveCli();
                    }
                });
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
        $("cli-form-title").textContent = t("addCliFormTitle");
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
        $("cli-form-title").textContent = t("editCliFormTitle", inst.name);
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
        window.switchTab("console");
        openAddCliForm();
    });
    $("btn-open-rename-cli").addEventListener("click", openEditCliForm);
    $("btn-cancel-cli").addEventListener("click", () => $("cli-form-box").classList.add("hidden"));

    $("btn-save-cli").addEventListener("click", async () => {
        const id = $("cli-form-id").value.trim();
        const payload = {
            name: $("cli-form-name").value.trim() || "New CLI",
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
            appendMsg("console-messages", "system", "SeaORM", `Instance "${saved.name}" (${saved.role}) saved.`);
        }
    });

    async function deleteActiveCli() {
        const inst = getActiveInstance();
        if (!inst) return;
        if (!confirm(`Delete CLI instance "${inst.name}"?`)) return;
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
        box.textContent = `Running (${action}) on ${inst.name}...`;

        try {
            const resp = await fetch(`/api/instances/${inst.id}/auth`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ action, value }),
            });
            const data = await resp.json();
            logRpc("in", `auth:${inst.name}`, data);
            const auth = data.auth || {};
            let html = `<strong>Instance:</strong> ${escapeHtml(inst.name)}\n`;
            if (auth.authenticated !== undefined) {
                html += `<strong>Authenticated:</strong> ${auth.authenticated ? "YES ✓" : "NO (Authentication required)"}\n`;
            }
            if (auth.url) {
                html += `<strong>Auth URL:</strong> <a href="${escapeHtml(auth.url)}" target="_blank" rel="noopener">${escapeHtml(auth.url)}</a>\n`;
            }
            if (auth.code) {
                html += `<strong>Device Code:</strong> <span style="color:var(--accent);font-size:1.05rem;font-weight:700;">${escapeHtml(auth.code)}</span>\n`;
            }
            if (auth.detail || auth.raw) {
                html += `\n${escapeHtml(auth.detail || auth.raw)}`;
            }
            box.innerHTML = html;
            await loadInstances();
        } catch (err) {
            box.textContent = `Auth error: ${err.message}`;
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
        statusEl.textContent = t("connectingTo", inst.name);
        statusEl.className = "badge";

        const ws = new WebSocket(`${wsBaseUrl()}/ws/instances/${encodeURIComponent(inst.id)}/terminal`);
        state.terminalWs = ws;

        ws.onopen = () => {
            statusEl.textContent = t("connectedTo", inst.name);
            statusEl.className = "badge badge-online";
            viewport.focus();
        };

        ws.onmessage = (ev) => {
            try {
                const msg = JSON.parse(ev.data);
                if (msg.type === "screen") {
                    viewport.innerHTML = ansiToHtml(msg.data || t("tmuxSessionReady"));
                    viewport.scrollTop = viewport.scrollHeight;
                }
            } catch {
                // ignore
            }
        };

        ws.onclose = () => {
            statusEl.textContent = t("disconnected");
            statusEl.className = "badge badge-offline";
        };
    }

    window.connectNativeTerminal = connectNativeTerminal;
    window.openNativeTerminal = connectNativeTerminal;
    window.openTerminalModal = function (id) {
        if (id) {
            state.activeInstanceId = id;
            renderCliSelector();
        }
        connectNativeTerminal();
    };

    window.closeModal = function (modalName) {
        if (modalName === "terminal-modal" || modalName === "terminal") {
            $("native-terminal-drawer").classList.add("hidden");
            if (state.terminalWs) {
                state.terminalWs.close();
                state.terminalWs = null;
            }
        }
    };

    window.sendTerminalLine = function () {
        const input = $("terminal-cmd-input");
        const text = input.value;
        input.value = "";
        if (state.terminalWs?.readyState === WebSocket.OPEN) {
            state.terminalWs.send(JSON.stringify({ type: "line", text }));
        }
    };

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

    function toggleTerminalMaximize(force) {
        const drawer = $("native-terminal-drawer");
        if (!drawer) return;
        const isMaximized = typeof force === "boolean"
            ? force
            : !drawer.classList.contains("maximized");
        drawer.classList.toggle("maximized", isMaximized);
        const maxBtn = $("btn-term-maximize");
        if (maxBtn) {
            maxBtn.textContent = isMaximized ? t("btnRestoreTerm") : t("btnMaximizeTerm");
            maxBtn.title = isMaximized ? "Restore terminal window" : "Maximize terminal across screen";
        }
        const viewport = $("terminal-viewport");
        if (viewport) {
            viewport.scrollTop = viewport.scrollHeight;
        }
    }

    $("btn-term-maximize")?.addEventListener("click", () => toggleTerminalMaximize());

    $("btn-term-close").addEventListener("click", () => {
        $("native-terminal-drawer").classList.add("hidden");
        $("native-terminal-drawer").classList.remove("maximized");
        if (state.terminalWs) {
            state.terminalWs.close();
            state.terminalWs = null;
        }
    });

    $("btn-term-refresh").addEventListener("click", () => {
        if (state.terminalWs?.readyState === WebSocket.OPEN) {
            state.terminalWs.send(JSON.stringify({ type: "refresh" }));
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
        window.sendTerminalLine();
    });

    $("terminal-viewport").addEventListener("keydown", (e) => {
        if (!state.terminalWs || state.terminalWs.readyState !== WebSocket.OPEN) return;
        e.preventDefault();
        if (e.ctrlKey && e.key === "c") {
            state.terminalWs.send(JSON.stringify({ type: "raw", data: "C-c", special: true }));
        } else if (e.key === "Enter") {
            state.terminalWs.send(JSON.stringify({ type: "raw", data: "Enter", special: true }));
        } else if (e.key === "Backspace") {
            state.terminalWs.send(JSON.stringify({ type: "raw", data: "BSpace", special: true }));
        } else if (e.key === "ArrowUp") {
            state.terminalWs.send(JSON.stringify({ type: "raw", data: "Up", special: true }));
        } else if (e.key === "ArrowDown") {
            state.terminalWs.send(JSON.stringify({ type: "raw", data: "Down", special: true }));
        } else if (e.key.length === 1) {
            state.terminalWs.send(JSON.stringify({ type: "raw", data: e.key, special: false }));
        }
    });

    $("btn-cli-native-web").addEventListener("click", () => {
        const inst = getActiveInstance();
        if (!inst) return;
        window.open(`/api/instances/${encodeURIComponent(inst.id)}/web/`, "_blank");
    });

    // ═════════════════════════════════════════════════════════════════════════
    // 4. ACTIVE CONSOLE CHAT & CLIENT PROTOCOLS (CODEX WS, GOOSE ACP, HTTP RPC)
    // ═════════════════════════════════════════════════════════════════════════
    function updateConsoleBadges() {
        const inst = getActiveInstance();
        const connBadge = $("console-conn-badge");
        const instBadge = $("console-instance-badge");
        const roleBadge = $("console-role-badge");
        const sessBadge = $("console-session-badge");

        if (inst) {
            instBadge.textContent = `${inst.name} [${inst.kind.toUpperCase()}]`;
            roleBadge.textContent = inst.role || "developer";
            roleBadge.className = "badge badge-online";
        } else {
            instBadge.textContent = "-";
            roleBadge.textContent = "-";
        }

        if (state.connected) {
            connBadge.textContent = "Connected";
            connBadge.className = "badge badge-online";
        } else {
            connBadge.textContent = "Standby";
            connBadge.className = "badge";
        }

        sessBadge.textContent = state.activeSessionId ? `Session: ${state.activeSessionId.substring(0, 8)}...` : "Session: -";
        $("btn-console-interrupt").classList.toggle("hidden", !state.turnRunning);
    }

    function renderSessionsList() {
        const container = $("cli-sessions-list");
        container.innerHTML = "";
        const inst = getActiveInstance();
        if (!inst) return;
        const list = state.sessionsByInstance[inst.id] || [];
        if (list.length === 0) {
            container.innerHTML = `<div class="field-hint">${t("selectConnectHint")}</div>`;
            return;
        }
        list.forEach((s) => {
            const div = document.createElement("div");
            div.className = `session-item ${s.id === state.activeSessionId ? "active" : ""}`;
            div.innerHTML = `<strong>${escapeHtml(s.title || s.id)}</strong><br><small style="color:var(--text-muted);">${s.id}</small>`;
            div.addEventListener("click", () => {
                state.activeSessionId = s.id;
                updateConsoleBadges();
                renderSessionsList();
            });
            container.appendChild(div);
        });
    }

    async function sendActiveWsRpc(method, params = {}) {
        return new Promise((resolve, reject) => {
            if (!state.ws || state.ws.readyState !== WebSocket.OPEN) {
                return reject(new Error("WebSocket not connected"));
            }
            const id = nextId();
            state.pendingRequests.set(id, { resolve, reject });
            const payload = { jsonrpc: "2.0", id, method, params };
            logRpc("out", method, payload);
            state.ws.send(JSON.stringify(payload));
        });
    }

    async function connectActiveCli() {
        const inst = getActiveInstance();
        if (!inst) return;

        if (state.ws) {
            state.ws.close();
            state.ws = null;
        }

        const wsEndpoint =
            inst.kind === "codex"
                ? `${wsBaseUrl()}/ws/codex?instance_id=${encodeURIComponent(inst.id)}`
                : inst.kind === "goose"
                ? `${wsBaseUrl()}/ws/goose?instance_id=${encodeURIComponent(inst.id)}`
                : null;

        if (!wsEndpoint) {
            state.connected = true;
            updateConsoleBadges();
            appendMsg("console-messages", "system", inst.name, `Instance connected via HTTP RPC (${inst.endpoint_url}).`);
            return;
        }

        appendMsg("console-messages", "system", inst.name, `Connecting via bridge: ${wsEndpoint}`);
        const ws = new WebSocket(wsEndpoint);
        state.ws = ws;

        ws.onopen = async () => {
            state.connected = true;
            updateConsoleBadges();
            appendMsg("console-messages", "system", inst.name, `WebSocket bridge established with ${inst.name}.`);

            if (inst.kind === "codex") {
                try {
                    await sendActiveWsRpc("initialize", {
                        clientInfo: { name: "simply-agent-hub", version: "0.2.0" },
                        capabilities: { experimental: {} },
                    });
                    ws.send(JSON.stringify({ jsonrpc: "2.0", method: "initialized", params: {} }));
                    await createNewSessionOnActiveCli();
                } catch (e) {
                    appendMsg("console-messages", "system", "Init Error", e.message);
                }
            } else if (inst.kind === "goose") {
                try {
                    await sendActiveWsRpc("initialize", {
                        protocolVersion: 1,
                        clientCapabilities: { fs: { readTextFile: true, writeTextFile: true }, terminal: true },
                        clientInfo: { name: "simply-agent-hub", version: "0.2.0" },
                    });
                    await createNewSessionOnActiveCli();
                } catch (e) {
                    appendMsg("console-messages", "system", "Init Error", e.message);
                }
            }
        };

        ws.onmessage = (ev) => {
            try {
                const msg = JSON.parse(ev.data);
                logRpc("in", inst.kind, msg);
                handleIncomingWsMessage(inst, msg);
            } catch {
                // ignore
            }
        };

        ws.onclose = () => {
            state.connected = false;
            updateConsoleBadges();
            appendMsg("console-messages", "system", inst.name, "WebSocket connection closed.");
        };

        ws.onerror = (err) => {
            appendMsg("console-messages", "system", "Error", `WebSocket error: ${err.message || "Failed"}`);
        };
    }

    function handleIncomingWsMessage(inst, msg) {
        if (msg.id && state.pendingRequests.has(msg.id)) {
            const { resolve, reject } = state.pendingRequests.get(msg.id);
            state.pendingRequests.delete(msg.id);
            if (msg.error) reject(new Error(msg.error.message || JSON.stringify(msg.error)));
            else resolve(msg.result);
            return;
        }

        const method = msg.method || "";

        if (method === "item/agentMessage/delta") {
            const delta = msg.params?.delta || "";
            if (!state.streamingEl) {
                state.streamingEl = appendMsg("console-messages", "agent", `${inst.name} [${inst.role || inst.kind}]`, "");
            }
            if (state.streamingEl) {
                state.streamingEl.textContent += delta;
                const c = $("console-messages");
                c.scrollTop = c.scrollHeight;
            }
            return;
        }

        if (method === "session/update") {
            const upd = msg.params?.update;
            if (upd?.sessionUpdate === "agent_message_chunk") {
                const chunk = upd.content?.text || "";
                if (!state.streamingEl) {
                    state.streamingEl = appendMsg("console-messages", "agent", `${inst.name} [${inst.role || inst.kind}]`, "");
                }
                if (state.streamingEl) {
                    state.streamingEl.textContent += chunk;
                    const c = $("console-messages");
                    c.scrollTop = c.scrollHeight;
                }
            }
            return;
        }

        if (method === "turn/completed") {
            state.streamingEl = null;
            state.turnRunning = false;
            updateConsoleBadges();
            return;
        }

        if (method === "item/commandExecution/requestApproval" || method === "session/request_permission") {
            state.pendingApproval = { inst, msg };
            $("console-approval-details").textContent = JSON.stringify(msg.params, null, 2);
            $("console-approval-banner").classList.remove("hidden");
        }
    }

    async function createNewSessionOnActiveCli() {
        const inst = getActiveInstance();
        if (!inst) return null;

        if (inst.kind === "codex") {
            const res = await sendActiveWsRpc("thread/start", { cwd: inst.default_cwd });
            const threadId = res?.thread?.id || res?.threadId || `thread-${Date.now()}`;
            state.activeSessionId = threadId;
            const list = state.sessionsByInstance[inst.id] || [];
            list.unshift({ id: threadId, title: `Thread ${list.length + 1}` });
            state.sessionsByInstance[inst.id] = list;
            updateConsoleBadges();
            renderSessionsList();
            return threadId;
        }

        if (inst.kind === "goose") {
            const res = await sendActiveWsRpc("session/new", { cwd: inst.default_cwd, mcpServers: [] });
            const sid = res?.sessionId || `session-${Date.now()}`;
            state.activeSessionId = sid;
            const list = state.sessionsByInstance[inst.id] || [];
            list.unshift({ id: sid, title: `Session ${list.length + 1}` });
            state.sessionsByInstance[inst.id] = list;
            updateConsoleBadges();
            renderSessionsList();
            return sid;
        }

        const sid = `sess-${Date.now()}`;
        state.activeSessionId = sid;
        const list = state.sessionsByInstance[inst.id] || [];
        list.unshift({ id: sid, title: `Session ${list.length + 1}` });
        state.sessionsByInstance[inst.id] = list;
        updateConsoleBadges();
        renderSessionsList();
        return sid;
    }

    $("btn-cli-connect").addEventListener("click", connectActiveCli);
    $("btn-cli-new-session").addEventListener("click", createNewSessionOnActiveCli);

    $("console-prompt-form").addEventListener("submit", async (e) => {
        e.preventDefault();
        const inst = getActiveInstance();
        if (!inst) return;
        const input = $("console-prompt-input");
        const text = input.value.trim();
        if (!text) return;
        input.value = "";

        appendMsg("console-messages", "user", "You", text);

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
            appendMsg("console-messages", "system", `Error (${inst.name})`, err.message);
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
        if (!sel) return;
        sel.innerHTML = "";
        state.channels.forEach((ch) => {
            const opt = document.createElement("option");
            opt.value = ch.id;
            opt.textContent = `#${ch.name} — ${ch.description || "Orchestration channel"}`;
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
        recipientSel.innerHTML = `<option value="@all">@all (All channel participants)</option>`;

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
        const ch = getActiveChannel();
        if (!confirm(`Delete channel "${ch ? ch.name : ""}"?`)) return;
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
        appendMsg("channel-messages", "system", "Orchestrator", t("participantsUpdated"));
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
        const isUser = m.sender_name === "User" || m.sender_name === "Utilisateur";
        const role = isUser ? "user" : "agent";
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

    window.publishOrchestrationMessage = async function () {
        if (!state.activeChannelId) return;
        const input = $("channel-message-input");
        const content = input.value.trim();
        if (!content) return;
        input.value = "";

        const recipient = $("channel-recipient-select").value || "@all";
        const resp = await fetch(`/api/orchestration/channels/${state.activeChannelId}/publish`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
                sender_name: "User",
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
    };

    $("channel-publish-form").addEventListener("submit", async (e) => {
        e.preventDefault();
        await window.publishOrchestrationMessage();
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
            "SeaORM Retention",
            t("retentionApplied", retention_days, max_messages, pruneData.pruned ?? 0)
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
            viewer.innerHTML = `<span class="field-hint">${t("noDiffDetected")}</span>`;
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
                filesList.innerHTML = `<div class="field-hint">${t("cleanWorkingTree")}</div>`;
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
            $("git-diff-viewer").textContent = `Git error: ${err.message}`;
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
        if (!confirm("Revert all modifications across the entire workspace (git checkout -- .)?")) return;
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

    function detectBrowserLanguage() {
        const saved = localStorage.getItem("agent_hub_lang");
        if (saved && I18N[saved]) {
            return saved;
        }
        const candidates = (navigator.languages && navigator.languages.length)
            ? navigator.languages
            : [navigator.language || "en"];
        for (const cand of candidates) {
            if (!cand) continue;
            const short = String(cand).toLowerCase().split(/[-_]/)[0];
            if (I18N[short]) {
                return short;
            }
        }
        return "en";
    }

    function renderLanguageSwitcher() {
        const container = document.querySelector(".lang-selector");
        if (!container) return;
        container.innerHTML = "";
        Object.keys(I18N).forEach((langCode) => {
            const btn = document.createElement("button");
            btn.id = `lang-btn-${langCode}`;
            btn.className = `btn-lang ${langCode === currentLang ? "active" : ""}`;
            btn.textContent = langCode.toUpperCase();
            btn.title = `Switch to ${langCode.toUpperCase()}`;
            btn.addEventListener("click", () => window.setAppLanguage(langCode));
            container.appendChild(btn);
        });
    }

    function toggleSidebar(force) {
        const grids = document.querySelectorAll(".workspace-grid");
        const anyCollapsed = grids[0]?.classList.contains("sidebar-collapsed");
        const shouldCollapse = typeof force === "boolean" ? !force : !anyCollapsed;
        grids.forEach((g) => g.classList.toggle("sidebar-collapsed", shouldCollapse));
        document.body.classList.toggle("sidebar-collapsed-active", shouldCollapse);
        localStorage.setItem("agent_hub_sidebar_collapsed", shouldCollapse ? "true" : "false");
        const btn = $("btn-toggle-sidebar");
        if (btn) {
            btn.textContent = shouldCollapse ? t("btnExpandSidebar") : t("btnToggleSidebar");
        }
    }

    function toggleCompactHeader(force) {
        const isCurrentlyCompact = document.body.classList.contains("compact-header");
        const shouldBeCompact = typeof force === "boolean" ? force : !isCurrentlyCompact;
        document.body.classList.toggle("compact-header", shouldBeCompact);
        localStorage.setItem("agent_hub_compact_header", shouldBeCompact ? "true" : "false");
        const btn = $("btn-toggle-compact-header");
        if (btn) {
            btn.textContent = shouldBeCompact ? t("btnExpandHeader") : t("btnCompactHeader");
        }
    }

    window.toggleSidebar = toggleSidebar;
    window.toggleCompactHeader = toggleCompactHeader;
    window.toggleTerminalMaximize = toggleTerminalMaximize;
    window.detectBrowserLanguage = detectBrowserLanguage;
    window.renderLanguageSwitcher = renderLanguageSwitcher;
    window.I18N = I18N;

    window.registerLanguagePack = function (code, dictionary) {
        if (!code || typeof code !== "string" || !dictionary) return;
        const short = code.toLowerCase().trim();
        I18N[short] = { ...(I18N.en || {}), ...dictionary };
        renderLanguageSwitcher();
    };

    window.setAppLanguage = function (lang) {
        const normalized = String(lang || "").toLowerCase().trim();
        currentLang = I18N[normalized] ? normalized : "en";
        localStorage.setItem("agent_hub_lang", currentLang);
        document.documentElement.lang = currentLang;

        document.querySelectorAll(".btn-lang").forEach((btn) => {
            btn.classList.toggle("active", btn.id === `lang-btn-${currentLang}`);
        });

        document.querySelectorAll("[data-i18n]").forEach((el) => {
            const k = el.getAttribute("data-i18n");
            if (k) el.textContent = t(k);
        });

        // Update dynamic action buttons
        const isCollapsed = document.body.classList.contains("sidebar-collapsed-active");
        const btnSb = $("btn-toggle-sidebar");
        if (btnSb) btnSb.textContent = isCollapsed ? t("btnExpandSidebar") : t("btnToggleSidebar");

        const isCompact = document.body.classList.contains("compact-header");
        const btnH = $("btn-toggle-compact-header");
        if (btnH) btnH.textContent = isCompact ? t("btnExpandHeader") : t("btnCompactHeader");

        const isTermMax = $("native-terminal-drawer")?.classList.contains("maximized");
        const btnTm = $("btn-term-maximize");
        if (btnTm) btnTm.textContent = isTermMax ? t("btnRestoreTerm") : t("btnMaximizeTerm");

        loadRolePresets();
        renderCliSelector();
        renderInventoryTable();
    };

    window.state = state;

    $("btn-toggle-sidebar")?.addEventListener("click", () => toggleSidebar());
    $("btn-toggle-compact-header")?.addEventListener("click", () => toggleCompactHeader());

    document.addEventListener("keydown", (e) => {
        if (e.key === "Escape") {
            const drawer = $("native-terminal-drawer");
            if (drawer?.classList.contains("maximized")) {
                toggleTerminalMaximize(false);
            }
        }
    });

    // Init
    currentLang = detectBrowserLanguage();
    renderLanguageSwitcher();
    window.setAppLanguage(currentLang);

    if (localStorage.getItem("agent_hub_compact_header") === "true") {
        toggleCompactHeader(true);
    }
    if (localStorage.getItem("agent_hub_sidebar_collapsed") === "true") {
        toggleSidebar(false);
    }

    loadRolePresets()
        .then(loadInstances)
        .then(loadChannels);
    setInterval(loadInstances, 15000);
})();
