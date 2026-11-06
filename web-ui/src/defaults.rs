//! Default Role Prompts, Labels and Language Packs (Database Healing & Verification)
//!
//! This module provides canonical fallback data and an automated reconciliation routine
//! (`ensure_database_defaults`) that verifies the database tables (`role_prompts` and
//! `language_packs`) and inserts any missing entries to heal incompletely initialized databases.

use crate::entities::{language_pack, role_prompt};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, Set,
};
use tracing::info;
use uuid::Uuid;

/// A canonical default prompt definition for database seeding and healing.
pub struct DefaultRolePromptSpec {
    pub role: &'static str,
    pub lang: &'static str,
    pub label: &'static str,
    pub prompt: &'static str,
}

/// Canonical default role prompts and labels across English, French, and German.
pub const CANONICAL_ROLE_PROMPTS: &[DefaultRolePromptSpec] = &[
    // ─── English (en) ────────────────────────────────────────────────────────
    DefaultRolePromptSpec {
        role: "architect",
        lang: "en",
        label: "🏗️ Software Architect",
        prompt: "You are the Lead Software Architect of the Simply Agent Hub collective. Your mission is to design technical architectures, break down tasks into clean milestones, specify API/database contracts, and coordinate other CLI agents (Developer, Reviewer, Tester) on the orchestration channel.",
    },
    DefaultRolePromptSpec {
        role: "developer",
        lang: "en",
        label: "💻 Lead Developer",
        prompt: "You are the Lead Implementation Developer of the Simply Agent Hub collective. Your mission is to write clean, performant, and idiomatic code in the shared workspace (/srv), implement architecture specifications, and address code review feedback.",
    },
    DefaultRolePromptSpec {
        role: "reviewer",
        lang: "en",
        label: "🔍 Code Reviewer & QA",
        prompt: "You are the Code Reviewer & QA Specialist of the Simply Agent Hub collective. Your mission is to inspect changes, spot potential bugs and edge cases, ensure error handling and security standards, and provide constructive feedback on the channel.",
    },
    DefaultRolePromptSpec {
        role: "tester",
        lang: "en",
        label: "🧪 QA & E2E Test Engineer",
        prompt: "You are the QA & E2E Test Engineer of the Simply Agent Hub collective. Your mission is to write and execute unit tests, integration tests, and verification scripts, ensuring builds compile cleanly with zero warnings or regressions.",
    },
    DefaultRolePromptSpec {
        role: "security",
        lang: "en",
        label: "🛡️ Security & DevSecOps Auditor",
        prompt: "You are the Security & DevSecOps Auditor of the Simply Agent Hub collective. Your mission is to audit permissions, rootless container boundaries, credentials safety, and input sanitization.",
    },
    DefaultRolePromptSpec {
        role: "general",
        lang: "en",
        label: "🤖 Versatile Agent",
        prompt: "You are a versatile CLI agent connected to the Simply Agent Hub orchestration bus. Collaborate cleanly and effectively with other agents on the channel.",
    },

    // ─── French (fr) ─────────────────────────────────────────────────────────
    DefaultRolePromptSpec {
        role: "architect",
        lang: "fr",
        label: "🏗️ Architecte Logiciel",
        prompt: "Tu es l'Architecte Logiciel du collectif Simply Agent Hub. Ton rôle est de concevoir l'architecture technique, découper les tâches de manière claire, définir les contrats d'API/bases de données et coordonner les autres agents CLI (Développeur, Reviewer, Testeur) sur le canal d'orchestration.",
    },
    DefaultRolePromptSpec {
        role: "developer",
        lang: "fr",
        label: "💻 Développeur Principal",
        prompt: "Tu es l'Agent Développeur Principal (Implémentation) du collectif Simply Agent Hub. Ton rôle est d'écrire du code propre, performant et idiomatique dans le workspace partagé (/srv), d'appliquer les spécifications de l'Architecte et de corriger les retours du Reviewer.",
    },
    DefaultRolePromptSpec {
        role: "reviewer",
        lang: "fr",
        label: "🔍 Code Reviewer & Qualité",
        prompt: "Tu es l'Agent Code Reviewer & Qualité du collectif Simply Agent Hub. Ton rôle est d'analyser les modifications, détecter les bugs potentiels, vérifier la sécurité, la gestion d'erreurs et la conformité au style du projet, puis de donner un retour constructif aux autres agents sur le canal.",
    },
    DefaultRolePromptSpec {
        role: "tester",
        lang: "fr",
        label: "🧪 Ingénieur QA & Tests E2E",
        prompt: "Tu es l'Agent Ingénieur QA & Tests E2E du collectif Simply Agent Hub. Ton rôle est d'élaborer et d'exécuter les tests unitaires, d'intégration et les vérifications curl/E2E, puis de valider que tout compile et passe sans régression.",
    },
    DefaultRolePromptSpec {
        role: "security",
        lang: "fr",
        label: "🛡️ Auditeur Sécurité & DevSecOps",
        prompt: "Tu es l'Agent Auditeur Sécurité & DevSecOps du collectif Simply Agent Hub. Ton rôle est de vérifier les permissions, l'isolation des conteneurs rootless, l'absence de secrets exposés et la robustesse des entrées utilisateur.",
    },
    DefaultRolePromptSpec {
        role: "general",
        lang: "fr",
        label: "🤖 Agent Polyvalent",
        prompt: "Tu es un Agent CLI polyvalent connecté au bus d'orchestration Simply Agent Hub. Collabore de manière concise et structurée avec les autres agents présents dans le canal.",
    },

    // ─── German (de) ─────────────────────────────────────────────────────────
    DefaultRolePromptSpec {
        role: "architect",
        lang: "de",
        label: "🏗️ Software-Architekt",
        prompt: "Du bist der leitende Software-Architekt des Simply Agent Hub Kollektivs. Deine Aufgabe ist es, modulare Architekturen zu entwerfen, Arbeitspakete zu definieren, API-/Datenbankverträge festzulegen und die anderen CLI-Agenten im Orchestrierungskanal zu koordinieren.",
    },
    DefaultRolePromptSpec {
        role: "developer",
        lang: "de",
        label: "💻 Lead-Entwickler",
        prompt: "Du bist der Lead-Entwickler des Simply Agent Hub Kollektivs. Deine Aufgabe ist es, sauberen, performanten und idiomatischen Code im gemeinsamen Arbeitsbereich (/srv) zu schreiben, Spezifikationen umzusetzen und Feedback zu berücksichtigen.",
    },
    DefaultRolePromptSpec {
        role: "reviewer",
        lang: "de",
        label: "🔍 Code-Reviewer & QA",
        prompt: "Du bist der Code-Reviewer und QA-Spezialist des Simply Agent Hub Kollektivs. Deine Aufgabe ist es, Codeänderungen zu prüfen, Fehler und Sicherheitsrisiken aufzuspüren und konstruktives Feedback im Kanal zu teilen.",
    },
    DefaultRolePromptSpec {
        role: "tester",
        lang: "de",
        label: "🧪 QA- & E2E-Testingenieur",
        prompt: "Du bist der QA- und E2E-Testingenieur des Simply Agent Hub Kollektivs. Deine Aufgabe ist es, Unit-, Integrations- und Verifizierungstests zu erstellen und sicherzustellen, dass Builds fehlerfrei und ohne Warnungen durchlaufen.",
    },
    DefaultRolePromptSpec {
        role: "security",
        lang: "de",
        label: "🛡️ Sicherheits- & DevSecOps-Auditor",
        prompt: "Du bist der Sicherheits- und DevSecOps-Auditor des Simply Agent Hub Kollektivs. Deine Aufgabe ist es, Berechtigungen, rootless Container-Isolation und sicheren Umgang mit Eingaben zu gewährleisten.",
    },
    DefaultRolePromptSpec {
        role: "general",
        lang: "de",
        label: "🤖 Vielseitiger Agent",
        prompt: "Du bist ein vielseitiger CLI-Agent im Simply Agent Hub Orchestrierungsbus. Arbeite strukturiert und präzise mit den anderen Agenten im Kanal zusammen.",
    },
];

/// Returns the fallback static label for a given role and language.
pub fn fallback_role_label(role: &str, lang: &str) -> &'static str {
    CANONICAL_ROLE_PROMPTS
        .iter()
        .find(|p| p.role == role && p.lang == lang)
        .map(|p| p.label)
        .unwrap_or_else(|| {
            CANONICAL_ROLE_PROMPTS
                .iter()
                .find(|p| p.role == role && p.lang == "en")
                .map(|p| p.label)
                .unwrap_or("🤖 Agent")
        })
}

/// Returns the fallback static prompt for a given role and language.
pub fn fallback_role_prompt(role: &str, lang: &str) -> &'static str {
    CANONICAL_ROLE_PROMPTS
        .iter()
        .find(|p| p.role == role && p.lang == lang)
        .map(|p| p.prompt)
        .unwrap_or_else(|| {
            CANONICAL_ROLE_PROMPTS
                .iter()
                .find(|p| p.role == role && p.lang == "en")
                .map(|p| p.prompt)
                .unwrap_or("You are an agent connected to Simply Agent Hub.")
        })
}

/// Complete English UI dictionary JSON.
fn default_lang_dict_en() -> &'static str {
    r#"{
        "appTitle": "Simply Agent Hub",
        "btnRefreshAll": "↻ Refresh Status",
        "tabConsole": "CLI Console, Roles & Native Terminal",
        "tabOrchestration": "Multi-CLI Orchestration Channels",
        "tabGit": "🔀 Git Diff & Commit",
        "tabInventory": "Container Fleet",
        "tabRpc": "JSON-RPC Inspector",
        "cliSelectorTitle": "CLI Container Selection",
        "cliSelectorSubtitle": "Select, rename, assign a role, or authenticate any of your CLI instances.",
        "activeCliLabel": "Active CLI Instance",
        "btnAddCli": "+ Add CLI",
        "btnEditCli": "✎ Rename / Edit",
        "btnAuthCli": "🔑 Authenticate",
        "agentRoleTitle": "🎭 Agent Role & Mission",
        "roleInCollectiveLabel": "Role in collective",
        "rolePromptLabel": "Role Prompt (suggested default / customizable)",
        "btnSaveRole": "💾 Save Role",
        "btnSaveRoleTemplate": "💾 Save to DB",
        "btnOnboardCli": "🎭 Initialize Agent",
        "addCliFormTitle": "Add CLI Container",
        "customNameLabel": "Custom Name",
        "cliAgentTypeLabel": "CLI Agent Kind",
        "defaultRoleLabel": "Default Role",
        "bridgeUrlLabel": "Auth/Tmux Bridge URL (optional)",
        "secretTokenLabel": "Secret / Token (optional)",
        "workingDirLabel": "Working Directory (cwd)",
        "btnSaveDatabase": "Save to Database",
        "btnCancel": "Cancel",
        "authPanelTitle": "Container Web Authentication",
        "authPanelSubtitle": "Authenticate the selected container without leaving this dashboard.",
        "btnCheckStatus": "Check Status",
        "btnGetDeviceCode": "Get Device Code / OAuth Link",
        "apiKeyInjectLabel": "Or inject an API key (OpenAI / Gemini / Anthropic)",
        "btnApplyKey": "Apply Key",
        "oauthCallbackLabel": "Or validate OAuth Callback URL (http://localhost:...)",
        "btnValidateOauth": "Validate OAuth",
        "btnConnectCli": "Connect to CLI",
        "btnNewSession": "+ Session / Thread",
        "btnNativeTerminal": "🖥 Native Terminal (Tmux/WS)",
        "btnCliWebServer": "🌐 CLI Web Server",
        "sessionsThreadsTitle": "Sessions / Threads",
        "selectConnectHint": "Select and connect a CLI instance",
        "btnInterrupt": "⏹ Interrupt",
        "btnClear": "Clear",
        "termDrawerTitle": "🖥 Interactive Native Terminal (Direct Tmux WebSocket)",
        "btnClose": "✕ Close",
        "btnSendPty": "Send to PTY ➤",
        "approvalTitle": "Tool approval required by CLI",
        "btnApprove": "Approve",
        "btnDecline": "Decline",
        "btnSend": "Send ➤",
        "orchChannelsTitle": "Orchestration Channels",
        "orchChannelsSubtitle": "Create a shared channel where multiple CLI instances exchange messages in real-time.",
        "activeChannelLabel": "Active Channel",
        "btnNewChannel": "+ New Channel",
        "btnDelete": "🗑 Delete",
        "createChannelTitle": "Create Channel",
        "channelNameLabel": "Channel Name",
        "descriptionLabel": "Description",
        "btnCreate": "Create",
        "clisJoinedChannelTitle": "CLIs in this channel",
        "clisJoinedChannelHint": "Check the instances that participate in this channel's exchanges:",
        "btnSaveParticipants": "💾 Save Channel Participants",
        "retentionTitle": "🧹 Configurable Retention",
        "retentionDaysLabel": "Retention (days, 0 = unlimited)",
        "maxMessagesLabel": "Max message count (0 = unlimited)",
        "btnSaveRetention": "💾 Apply & Prune",
        "btnExportMd": "📥 Export .md",
        "btnRefreshHistory": "↻ DB History",
        "btnBroadcast": "Broadcast ➤",
        "gitStatusTitle": "🔀 Workspace Git Status",
        "gitStatusSubtitle": "Inspect changes produced by your CLI agents, revert a file, or create a commit.",
        "gitCwdLabel": "Git Directory (cwd)",
        "btnAnalyze": "↻ Analyze",
        "activeBranchLabel": "Active branch: ",
        "modifiedFilesTitle": "Modified Files",
        "createCommitTitle": "Create Commit",
        "commitMsgLabel": "Commit Message",
        "btnGitCommit": "✅ Stage All (git add -A) & Commit",
        "recentCommitsTitle": "Recent Commits",
        "unifiedDiffTitle": "Unified Diff (`git diff HEAD`)",
        "btnRevertAll": "↩ Revert entire workspace",
        "inventoryTitle": "Registered CLI Container Fleet",
        "inventorySubtitle": "Manage all your Codex, Antigravity, Claude, and Goose instances persisted in the database.",
        "btnNewCliContainer": "+ New CLI Container",
        "thInstanceName": "Instance Name",
        "thType": "Type",
        "thRole": "Assigned Role",
        "thTransport": "Transport",
        "thEndpoint": "Endpoint",
        "thCwd": "CWD",
        "thStatus": "Status",
        "thActions": "Actions",
        "rpcTitle": "JSON-RPC 2.0 Frame Inspector & Orchestration Bus",
        "rpcSubtitle": "Real-time log of frames exchanged with CLI containers.",
        "btnClearLog": "Clear Log",
        "onlineBadge": "Online",
        "offlineBadge": "Offline",
        "btnUse": "Use",
        "btnRename": "✎ Rename",
        "btnCompactHeader": "🗕 Header",
        "btnExpandHeader": "🗖 Header",
        "btnToggleSidebar": "◀ Sidebar",
        "btnExpandSidebar": "▶ Sidebar",
        "btnMaximizeTerm": "🗖 Maximize",
        "btnRestoreTerm": "🗗 Restore",
        "btnOpenSettings": "⚙",
        "settingsTitle": "⚙ Settings & Preferences",
        "settingsLangLabel": "Interface Language",
        "settingsLangHint": "Choose your preferred interface language. Additional languages can be added dynamically.",
        "settingsDisplayLabel": "Ergonomics & Display Options",
        "settingsCompactHeader": "Compact header layout (maximize vertical space)",
        "settingsRetractSidebar": "Retract left sidebar (maximize horizontal workspace)",
        "settingsClose": "Close"
    }"#
}

/// Complete French UI dictionary JSON.
fn default_lang_dict_fr() -> &'static str {
    r#"{
        "appTitle": "Simply Agent Hub",
        "btnRefreshAll": "↻ Actualiser l'état",
        "tabConsole": "Console CLI, Rôles & Terminal Natif",
        "tabOrchestration": "Canaux d'Orchestration Multi-CLIs",
        "tabGit": "🔀 Git Diff & Commit",
        "tabInventory": "Parc de Conteneurs",
        "tabRpc": "Inspecteur JSON-RPC",
        "cliSelectorTitle": "Sélection du Conteneur CLI",
        "cliSelectorSubtitle": "Choisissez, renommez, attribuez un rôle ou authentifiez n'importe laquelle de vos instances.",
        "activeCliLabel": "Instance CLI active",
        "btnAddCli": "+ Ajouter un CLI",
        "btnEditCli": "✎ Renommer / Éditer",
        "btnAuthCli": "🔑 Authentifier",
        "agentRoleTitle": "🎭 Rôle & Mission de l'Agent",
        "roleInCollectiveLabel": "Rôle dans le collectif",
        "rolePromptLabel": "Prompt de rôle (par défaut proposé / paramétrable)",
        "btnSaveRole": "💾 Sauver le rôle",
        "btnSaveRoleTemplate": "💾 Enregistrer en base",
        "btnOnboardCli": "🎭 Initialiser l'agent",
        "addCliFormTitle": "Ajouter un conteneur CLI",
        "customNameLabel": "Nom personnalisé",
        "cliAgentTypeLabel": "Type d'agent CLI",
        "defaultRoleLabel": "Rôle par défaut",
        "bridgeUrlLabel": "URL du Pont d'Auth/Tmux (optionnel)",
        "secretTokenLabel": "Secret / Token (optionnel)",
        "workingDirLabel": "Répertoire de travail (cwd)",
        "btnSaveDatabase": "Enregistrer en base",
        "btnCancel": "Annuler",
        "authPanelTitle": "Authentification Web du Conteneur",
        "authPanelSubtitle": "Authentifiez le conteneur sélectionné sans quitter cette interface.",
        "btnCheckStatus": "Vérifier statut",
        "btnGetDeviceCode": "Obtenir Code Device / Lien OAuth",
        "apiKeyInjectLabel": "Ou injecter une clé API (OpenAI / Gemini / Anthropic)",
        "btnApplyKey": "Appliquer la clé",
        "oauthCallbackLabel": "Ou valider une URL Callback OAuth (http://localhost:...)",
        "btnValidateOauth": "Valider OAuth",
        "btnConnectCli": "Connecter au CLI",
        "btnNewSession": "+ Session / Thread",
        "btnNativeTerminal": "🖥 Terminal Natif (Tmux/WS)",
        "btnCliWebServer": "🌐 Serveur Web du CLI",
        "sessionsThreadsTitle": "Sessions / Threads",
        "selectConnectHint": "Sélectionnez et connectez un CLI",
        "btnInterrupt": "⏹ Interrompre",
        "btnClear": "Effacer",
        "termDrawerTitle": "🖥 Terminal Natif Interactif (Tmux Direct WebSocket)",
        "btnClose": "✕ Fermer",
        "btnSendPty": "Envoyer au PTY ➤",
        "approvalTitle": "Autorisation d'outil requise par le CLI",
        "btnApprove": "Autoriser",
        "btnDecline": "Refuser",
        "btnSend": "Envoyer ➤",
        "orchChannelsTitle": "Canaux d'Orchestration",
        "orchChannelsSubtitle": "Créez un canal partagé où plusieurs instances CLI échangent des messages en temps réel.",
        "activeChannelLabel": "Canal actif",
        "btnNewChannel": "+ Nouveau Canal",
        "btnDelete": "🗑 Supprimer",
        "createChannelTitle": "Créer un canal",
        "channelNameLabel": "Nom du canal",
        "descriptionLabel": "Description",
        "btnCreate": "Créer",
        "clisJoinedChannelTitle": "CLIs connectés à ce canal",
        "clisJoinedChannelHint": "Cochez les instances qui participent aux échanges de ce canal :",
        "btnSaveParticipants": "💾 Enregistrer les participants du canal",
        "retentionTitle": "🧹 Rétention Paramétrable",
        "retentionDaysLabel": "Rétention (jours, 0 = illimité)",
        "maxMessagesLabel": "Nombre max de messages conservés (0 = illimité)",
        "btnSaveRetention": "💾 Appliquer & Purger",
        "btnExportMd": "📥 Export .md",
        "btnRefreshHistory": "↻ Historique DB",
        "btnBroadcast": "Diffuser ➤",
        "gitStatusTitle": "🔀 État Git du Workspace",
        "gitStatusSubtitle": "Inspectez les modifications produites par vos agents CLI, annulez un fichier ou créez un commit.",
        "gitCwdLabel": "Répertoire Git (cwd)",
        "btnAnalyze": "↻ Analyser",
        "activeBranchLabel": "Branche active : ",
        "modifiedFilesTitle": "Fichiers modifiés",
        "createCommitTitle": "Créer un Commit",
        "commitMsgLabel": "Message de commit",
        "btnGitCommit": "✅ Tout indexer (git add -A) & Commit",
        "recentCommitsTitle": "Derniers Commits",
        "unifiedDiffTitle": "Diff Unifié (`git diff HEAD`)",
        "btnRevertAll": "↩ Revert tout le workspace",
        "inventoryTitle": "Parc de Conteneurs CLI enregistrés",
        "inventorySubtitle": "Gérez l'ensemble de vos instances Codex, Antigravity, Claude et Goose persistées en base de données.",
        "btnNewCliContainer": "+ Nouveau Conteneur CLI",
        "thInstanceName": "Nom de l'instance",
        "thType": "Type",
        "thRole": "Rôle attribué",
        "thTransport": "Transport",
        "thEndpoint": "Endpoint",
        "thCwd": "Dossier CWD",
        "thStatus": "État",
        "thActions": "Actions",
        "rpcTitle": "Inspecteur de trames JSON-RPC 2.0 & Bus d'Orchestration",
        "rpcSubtitle": "Journal temps réel des échanges avec les conteneurs CLI.",
        "btnClearLog": "Vider le journal",
        "onlineBadge": "En ligne",
        "offlineBadge": "Hors ligne",
        "btnUse": "Utiliser",
        "btnRename": "✎ Renommer",
        "btnCompactHeader": "🗕 En-tête",
        "btnExpandHeader": "🗖 En-tête",
        "btnToggleSidebar": "◀ Menu",
        "btnExpandSidebar": "▶ Menu",
        "btnMaximizeTerm": "🗖 Agrandir",
        "btnRestoreTerm": "🗗 Réduire",
        "btnOpenSettings": "⚙",
        "settingsTitle": "⚙ Paramètres & Préférences",
        "settingsLangLabel": "Langue de l'interface",
        "settingsLangHint": "Choisissez votre langue d'affichage. De nouvelles langues peuvent être chargées dynamiquement.",
        "settingsDisplayLabel": "Ergonomie & Options d'affichage",
        "settingsCompactHeader": "En-tête compact (maximiser l'espace vertical)",
        "settingsRetractSidebar": "Rétracter le volet gauche (maximiser la largeur de travail)",
        "settingsClose": "Fermer"
    }"#
}

/// Complete German UI dictionary JSON.
fn default_lang_dict_de() -> &'static str {
    r#"{
        "appTitle": "Simply Agent Hub",
        "btnRefreshAll": "↻ Status aktualisieren",
        "tabConsole": "CLI-Konsole, Rollen & Natives Terminal",
        "tabOrchestration": "Multi-CLI Orchestrierungskanäle",
        "tabGit": "🔀 Git Diff & Commit",
        "tabInventory": "Container-Flotte",
        "tabRpc": "JSON-RPC Inspektor",
        "cliSelectorTitle": "CLI-Container-Auswahl",
        "cliSelectorSubtitle": "Wählen, benennen, weisen Sie eine Rolle zu oder authentifizieren Sie eine Ihrer CLI-Instanzen.",
        "activeCliLabel": "Aktive CLI-Instanz",
        "btnAddCli": "+ CLI hinzufügen",
        "btnEditCli": "✎ Umbenennen / Bearbeiten",
        "btnAuthCli": "🔑 Authentifizieren",
        "agentRoleTitle": "🎭 Agentenrolle & Mission",
        "roleInCollectiveLabel": "Rolle im Kollektiv",
        "rolePromptLabel": "Rollen-Prompt (vorgeschlagen / anpassbar)",
        "btnSaveRole": "💾 Rolle speichern",
        "btnSaveRoleTemplate": "💾 In DB speichern",
        "btnOnboardCli": "🎭 Agent initialisieren",
        "addCliFormTitle": "CLI-Container hinzufügen",
        "customNameLabel": "Benutzerdefinierter Name",
        "cliAgentTypeLabel": "CLI-Agententyp",
        "defaultRoleLabel": "Standardrolle",
        "bridgeUrlLabel": "Auth/Tmux Bridge-URL (optional)",
        "secretTokenLabel": "Geheimnis / Token (optional)",
        "workingDirLabel": "Arbeitsverzeichnis (cwd)",
        "btnSaveDatabase": "In Datenbank speichern",
        "btnCancel": "Abbrechen",
        "authPanelTitle": "Web-Authentifizierung des Containers",
        "authPanelSubtitle": "Authentifizieren Sie den ausgewählten Container direkt über dieses Dashboard.",
        "btnCheckStatus": "Status prüfen",
        "btnGetDeviceCode": "Device Code / OAuth-Link abrufen",
        "apiKeyInjectLabel": "Oder API-Schlüssel einfügen (OpenAI / Gemini / Anthropic)",
        "btnApplyKey": "Schlüssel anwenden",
        "oauthCallbackLabel": "Oder OAuth-Callback-URL validieren (http://localhost:...)",
        "btnValidateOauth": "OAuth validieren",
        "btnConnectCli": "Mit CLI verbinden",
        "btnNewSession": "+ Sitzung / Thread",
        "btnNativeTerminal": "🖥 Natives Terminal (Tmux/WS)",
        "btnCliWebServer": "🌐 CLI-Webserver",
        "sessionsThreadsTitle": "Sitzungen / Threads",
        "selectConnectHint": "Wählen und verbinden Sie eine CLI-Instanz",
        "btnInterrupt": "⏹ Unterbrechen",
        "btnClear": "Leeren",
        "termDrawerTitle": "🖥 Interaktives natives Terminal (Direktes Tmux WebSocket)",
        "btnClose": "✕ Schließen",
        "btnSendPty": "An PTY senden ➤",
        "approvalTitle": "Werkzeuggenehmigung vom CLI angefordert",
        "btnApprove": "Genehmigen",
        "btnDecline": "Ablehnen",
        "btnSend": "Senden ➤",
        "orchChannelsTitle": "Orchestrierungskanäle",
        "orchChannelsSubtitle": "Erstellen Sie einen geteilten Kanal, in dem mehrere CLI-Instanzen in Echtzeit zusammenarbeiten.",
        "activeChannelLabel": "Aktiver Kanal",
        "btnNewChannel": "+ Neuer Kanal",
        "btnDelete": "🗑 Löschen",
        "createChannelTitle": "Kanal erstellen",
        "channelNameLabel": "Kanalname",
        "descriptionLabel": "Beschreibung",
        "btnCreate": "Erstellen",
        "clisJoinedChannelTitle": "CLIs in diesem Kanal",
        "clisJoinedChannelHint": "Wählen Sie die Instanzen aus, die an diesem Kanal teilnehmen:",
        "btnSaveParticipants": "💾 Kanalteilnehmer speichern",
        "retentionTitle": "🧹 Konfigurierbare Aufbewahrung",
        "retentionDaysLabel": "Aufbewahrung (Tage, 0 = unbegrenzt)",
        "maxMessagesLabel": "Max. Nachrichtenanzahl (0 = unbegrenzt)",
        "btnSaveRetention": "💾 Anwenden & Bereinigen",
        "btnExportMd": "📥 Export .md",
        "btnRefreshHistory": "↻ DB-Verlauf",
        "btnBroadcast": "Senden ➤",
        "gitStatusTitle": "🔀 Workspace Git-Status",
        "gitStatusSubtitle": "Überprüfen Sie Änderungen Ihrer CLI-Agenten, setzen Sie Dateien zurück oder erstellen Sie Commits.",
        "gitCwdLabel": "Git-Verzeichnis (cwd)",
        "btnAnalyze": "↻ Analysieren",
        "activeBranchLabel": "Aktiver Branch: ",
        "modifiedFilesTitle": "Geänderte Dateien",
        "createCommitTitle": "Commit erstellen",
        "commitMsgLabel": "Commit-Nachricht",
        "btnGitCommit": "✅ Alles stagen (git add -A) & Committen",
        "recentCommitsTitle": "Neueste Commits",
        "unifiedDiffTitle": "Unified Diff (`git diff HEAD`)",
        "btnRevertAll": "↩ Gesamten Workspace zurücksetzen",
        "inventoryTitle": "Registrierte CLI-Container-Flotte",
        "inventorySubtitle": "Verwalten Sie alle in der Datenbank gespeicherten Codex-, Antigravity-, Claude- und Goose-Instanzen.",
        "btnNewCliContainer": "+ Neuer CLI-Container",
        "thInstanceName": "Instanzname",
        "thType": "Typ",
        "thRole": "Zugewiesene Rolle",
        "thTransport": "Transport",
        "thEndpoint": "Endpunkt",
        "thCwd": "Arbeitsverzeichnis (CWD)",
        "thStatus": "Status",
        "thActions": "Aktionen",
        "rpcTitle": "JSON-RPC 2.0 Frame-Inspektor & Orchestrierungsbus",
        "rpcSubtitle": "Echtzeitprotokoll der mit CLI-Containern ausgetauschten Frames.",
        "btnClearLog": "Protokoll leeren",
        "onlineBadge": "Online",
        "offlineBadge": "Offline",
        "btnUse": "Verwenden",
        "btnRename": "✎ Umbenennen",
        "btnCompactHeader": "🗕 Kopfzeile",
        "btnExpandHeader": "🗖 Kopfzeile",
        "btnToggleSidebar": "◀ Menü",
        "btnExpandSidebar": "▶ Menü",
        "btnMaximizeTerm": "🗖 Maximieren",
        "btnRestoreTerm": "🗗 Verkleinern",
        "btnOpenSettings": "⚙",
        "settingsTitle": "⚙ Einstellungen & Präferenzen",
        "settingsLangLabel": "Oberflächensprache",
        "settingsLangHint": "Wählen Sie Ihre bevorzugte Oberflächensprache. Weitere Sprachen können dynamisch geladen werden.",
        "settingsDisplayLabel": "Ergonomie & Anzeigeoptionen",
        "settingsCompactHeader": "Kompakte Kopfzeile (vertikalen Platz maximieren)",
        "settingsRetractSidebar": "Linke Seitenleiste einklappen (horizontalen Arbeitsbereich maximieren)",
        "settingsClose": "Schließen"
    }"#
}

/// Checks the `role_prompts` table and inserts any missing canonical prompts.
///
/// Returns the number of missing prompts that were inserted into the database.
pub async fn ensure_role_prompts_seeded(db: &DatabaseConnection) -> Result<usize, DbErr> {
    let mut added = 0;
    let now = Utc::now();

    for spec in CANONICAL_ROLE_PROMPTS {
        let exists = role_prompt::Entity::find()
            .filter(role_prompt::Column::Role.eq(spec.role))
            .filter(role_prompt::Column::Lang.eq(spec.lang))
            .one(db)
            .await?;

        if exists.is_none() {
            let model = role_prompt::ActiveModel {
                id: Set(Uuid::new_v4()),
                role: Set(spec.role.to_string()),
                lang: Set(spec.lang.to_string()),
                label: Set(spec.label.to_string()),
                prompt: Set(spec.prompt.to_string()),
                created_at: Set(now),
                updated_at: Set(now),
            };
            model.insert(db).await?;
            added += 1;
        }
    }

    if added > 0 {
        info!("Database verification: healed/inserted {added} missing role prompt(s) in database");
    }

    Ok(added)
}

/// Checks the `language_packs` table and inserts any missing canonical packs.
///
/// Returns the number of missing language packs that were inserted into the database.
pub async fn ensure_language_packs_seeded(db: &DatabaseConnection) -> Result<usize, DbErr> {
    let mut added = 0;
    let now = Utc::now();
    let packs = [
        ("en", "English", default_lang_dict_en()),
        ("fr", "Français", default_lang_dict_fr()),
        ("de", "Deutsch", default_lang_dict_de()),
    ];

    for (lang, name, dict) in packs {
        let exists = language_pack::Entity::find()
            .filter(language_pack::Column::Lang.eq(lang))
            .one(db)
            .await?;

        if exists.is_none() {
            let model = language_pack::ActiveModel {
                id: Set(Uuid::new_v4()),
                lang: Set(lang.to_string()),
                name: Set(name.to_string()),
                dictionary_json: Set(dict.to_string()),
                created_at: Set(now),
                updated_at: Set(now),
            };
            model.insert(db).await?;
            added += 1;
        }
    }

    if added > 0 {
        info!("Database verification: healed/inserted {added} missing language pack(s) in database");
    }

    Ok(added)
}

/// Reconciles both role prompts and language packs in the database.
pub async fn ensure_database_defaults(db: &DatabaseConnection) -> Result<(), DbErr> {
    ensure_role_prompts_seeded(db).await?;
    ensure_language_packs_seeded(db).await?;
    Ok(())
}

/// Retrieves the role prompt for a given role and language from the database.
///
/// If missing from database, automatically inserts the default into the database and returns it.
pub async fn get_or_create_role_prompt(
    db: &DatabaseConnection,
    role: &str,
    lang: &str,
) -> Result<String, DbErr> {
    // 1. Direct match in DB
    if let Some(model) = role_prompt::Entity::find()
        .filter(role_prompt::Column::Role.eq(role))
        .filter(role_prompt::Column::Lang.eq(lang))
        .one(db)
        .await?
    {
        return Ok(model.prompt);
    }

    // 2. Not in DB: heal the database by inserting the default into the role_prompts table!
    let now = Utc::now();
    let prompt_text = fallback_role_prompt(role, lang);
    let label_text = fallback_role_label(role, lang);

    let model = role_prompt::ActiveModel {
        id: Set(Uuid::new_v4()),
        role: Set(role.to_string()),
        lang: Set(lang.to_string()),
        label: Set(label_text.to_string()),
        prompt: Set(prompt_text.to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let saved = model.insert(db).await?;
    info!("Database auto-healed: added missing role prompt for '{role}' ({lang}) into database");
    Ok(saved.prompt)
}

/// Retrieves the human-readable role label for a given role and language from the database.
///
/// If missing from database, automatically inserts the default into the database and returns it.
pub async fn get_or_create_role_label(
    db: &DatabaseConnection,
    role: &str,
    lang: &str,
) -> Result<String, DbErr> {
    if let Some(model) = role_prompt::Entity::find()
        .filter(role_prompt::Column::Role.eq(role))
        .filter(role_prompt::Column::Lang.eq(lang))
        .one(db)
        .await?
    {
        return Ok(model.label);
    }

    // Heal database
    let _ = get_or_create_role_prompt(db, role, lang).await?;
    if let Some(model) = role_prompt::Entity::find()
        .filter(role_prompt::Column::Role.eq(role))
        .filter(role_prompt::Column::Lang.eq(lang))
        .one(db)
        .await?
    {
        return Ok(model.label);
    }

    Ok(fallback_role_label(role, lang).to_string())
}
