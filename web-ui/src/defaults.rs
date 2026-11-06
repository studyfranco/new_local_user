//! Default Role Prompts, Labels and Language Packs (Database Loader & Verification)
//!
//! All language data, role prompts, and available languages reside in the database tables
//! (`available_languages`, `role_prompts`, `language_packs`).
//!
//! At application startup, if the database is uninitialized or missing defaults, this module
//! reconciles the database by reading the canonical seed data from `static/defaults.js`.
//! In the Rust codebase, only configuration variables (`DEFAULT_LANG`, `DEFAULT_ROLE`) define
//! what to query from the database.

use crate::entities::{available_language, language_pack, role_prompt};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter, QueryOrder,
    Set,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{error, info};
use uuid::Uuid;

/// Default language code when none is requested or as fallback when a translation is missing.
pub const DEFAULT_LANG: &str = "en";

/// Default role when creating or querying an instance without an explicit role.
pub const DEFAULT_ROLE: &str = "developer";

/// Relative default file path for the JavaScript seed defaults.
pub const DEFAULT_JS_FILE_PATH: &str = "static/defaults.js";

/// Compile-time embedded copy of the JS defaults file to ensure container/test portability.
const EMBEDDED_DEFAULTS_JS: &str = include_str!("../static/defaults.js");

/// Deserialized seed representation from `static/defaults.js`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DefaultsSeed {
    pub available_languages: Vec<AvailableLanguageSeed>,
    pub role_prompts: Vec<RolePromptSeed>,
    pub language_packs: HashMap<String, LanguagePackSeed>,
}

/// Available language specification from `static/defaults.js`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AvailableLanguageSeed {
    pub code: String,
    pub name: String,
    pub native_name: String,
    pub is_default: bool,
    pub is_active: bool,
}

/// Role prompt specification from `static/defaults.js`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RolePromptSeed {
    pub role: String,
    pub lang: String,
    pub label: String,
    pub prompt: String,
}

/// Language pack specification from `static/defaults.js`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LanguagePackSeed {
    pub name: String,
    pub dictionary: serde_json::Value,
}

/// Parses canonical default seed data from the JavaScript file (`defaults.js`).
///
/// Looks first on disk (`static/defaults.js`, `web-ui/static/defaults.js`, or `DEFAULTS_JS_PATH`).
/// Falls back to the embedded defaults string if the external file is absent.
pub fn load_defaults_from_js() -> Result<DefaultsSeed, String> {
    let raw_content = if let Ok(custom_path) = std::env::var("DEFAULTS_JS_PATH")
        && let Ok(content) = std::fs::read_to_string(&custom_path)
    {
        content
    } else if let Ok(content) = std::fs::read_to_string(DEFAULT_JS_FILE_PATH) {
        content
    } else if let Ok(content) = std::fs::read_to_string("web-ui/static/defaults.js") {
        content
    } else if let Ok(content) = std::fs::read_to_string(Path::new("..").join(DEFAULT_JS_FILE_PATH))
    {
        content
    } else {
        EMBEDDED_DEFAULTS_JS.to_string()
    };

    let start = raw_content
        .find('{')
        .ok_or_else(|| "Failed to find opening '{' in defaults.js".to_string())?;

    // Find the matching closing '}' taking strings and escape sequences into account
    let mut depth = 0;
    let mut in_string = false;
    let mut is_escaped = false;
    let mut end = None;

    for (idx, ch) in raw_content[start..].char_indices() {
        if in_string {
            if is_escaped {
                is_escaped = false;
            } else if ch == '\\' {
                is_escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else {
            match ch {
                '"' => in_string = true,
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(start + idx);
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    let end = end
        .ok_or_else(|| "Failed to find matching closing '}' for defaults.js object".to_string())?;

    let json_slice = &raw_content[start..=end];
    serde_json::from_str::<DefaultsSeed>(json_slice)
        .map_err(|e| format!("Failed to parse defaults.js JSON payload: {e}"))
}

// ============================================================================
// DATABASE RECONCILIATION & AUTO-HEALING AT APPLICATION STARTUP
// ============================================================================

/// Ensures that the database contains all necessary defaults for:
/// 1. Master available languages (`available_languages` table)
/// 2. Language packs (`language_packs` table)
/// 3. Role prompts (`role_prompts` table)
///
/// Missing records are inserted from `defaults.js`; existing entries are preserved.
pub async fn ensure_database_defaults(db: &DatabaseConnection) -> Result<usize, DbErr> {
    let seed = match load_defaults_from_js() {
        Ok(s) => s,
        Err(e) => {
            error!("Could not load seed defaults from defaults.js: {e}");
            return Err(DbErr::Custom(e));
        }
    };

    let mut total_added = 0;
    total_added += ensure_available_languages_seeded(db, &seed).await?;
    total_added += ensure_language_packs_seeded(db, &seed).await?;
    total_added += ensure_role_prompts_seeded(db, &seed).await?;

    Ok(total_added)
}

/// Ensures the `available_languages` master table is seeded from the JS configuration.
pub async fn ensure_available_languages_seeded(
    db: &DatabaseConnection,
    seed: &DefaultsSeed,
) -> Result<usize, DbErr> {
    let now = Utc::now();
    let mut added = 0;

    for item in &seed.available_languages {
        let exists = available_language::Entity::find()
            .filter(available_language::Column::Code.eq(&item.code))
            .one(db)
            .await?;

        if exists.is_none() {
            let model = available_language::ActiveModel {
                id: Set(Uuid::new_v4()),
                code: Set(item.code.clone()),
                name: Set(item.name.clone()),
                native_name: Set(item.native_name.clone()),
                is_default: Set(item.is_default),
                is_active: Set(item.is_active),
                created_at: Set(now),
                updated_at: Set(now),
            };
            model.insert(db).await?;
            added += 1;
        }
    }

    if added > 0 {
        info!(
            "Database verification: healed/inserted {added} available language(s) from defaults.js"
        );
    }

    Ok(added)
}

/// Ensures the `language_packs` table is seeded from the JS configuration for all active available languages.
pub async fn ensure_language_packs_seeded(
    db: &DatabaseConnection,
    seed: &DefaultsSeed,
) -> Result<usize, DbErr> {
    let now = Utc::now();
    let mut added = 0;

    for (lang_code, pack_seed) in &seed.language_packs {
        let existing = language_pack::Entity::find()
            .filter(language_pack::Column::Lang.eq(lang_code))
            .one(db)
            .await?;

        if existing.is_none() {
            let dict_json =
                serde_json::to_string(&pack_seed.dictionary).unwrap_or_else(|_| "{}".to_string());
            let model = language_pack::ActiveModel {
                id: Set(Uuid::new_v4()),
                lang: Set(lang_code.clone()),
                name: Set(pack_seed.name.clone()),
                dictionary_json: Set(dict_json),
                created_at: Set(now),
                updated_at: Set(now),
            };
            model.insert(db).await?;
            added += 1;
        }
    }

    if added > 0 {
        info!(
            "Database verification: healed/inserted {added} missing language pack(s) in database"
        );
    }

    Ok(added)
}

/// Ensures the `role_prompts` table is seeded from the JS configuration.
pub async fn ensure_role_prompts_seeded(
    db: &DatabaseConnection,
    seed: &DefaultsSeed,
) -> Result<usize, DbErr> {
    let now = Utc::now();
    let mut added = 0;

    for spec in &seed.role_prompts {
        let existing = role_prompt::Entity::find()
            .filter(role_prompt::Column::Role.eq(&spec.role))
            .filter(role_prompt::Column::Lang.eq(&spec.lang))
            .one(db)
            .await?;

        if existing.is_none() {
            let model = role_prompt::ActiveModel {
                id: Set(Uuid::new_v4()),
                role: Set(spec.role.clone()),
                lang: Set(spec.lang.clone()),
                label: Set(spec.label.clone()),
                prompt: Set(spec.prompt.clone()),
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

// ============================================================================
// RUNTIME DATABASE QUERY & SELF-HEALING HELPERS
// ============================================================================

/// Queries the database for a role prompt, falling back to `DEFAULT_LANG` ("en") if missing.
/// If the database is missing both, heals the database from `defaults.js`.
pub async fn get_or_create_role_prompt(
    db: &DatabaseConnection,
    role: &str,
    lang: &str,
) -> Result<String, DbErr> {
    let normalized_role = role.trim().to_lowercase();
    let normalized_lang = lang.trim().to_lowercase();

    // 1. Direct query in database for requested language
    if let Some(record) = role_prompt::Entity::find()
        .filter(role_prompt::Column::Role.eq(&normalized_role))
        .filter(role_prompt::Column::Lang.eq(&normalized_lang))
        .one(db)
        .await?
    {
        return Ok(record.prompt);
    }

    // 2. Heal from defaults.js if missing in database for the requested language
    if let Ok(seed) = load_defaults_from_js()
        && let Some(spec) = seed
            .role_prompts
            .iter()
            .find(|s| s.role == normalized_role && s.lang == normalized_lang)
    {
        let now = Utc::now();
        let model = role_prompt::ActiveModel {
            id: Set(Uuid::new_v4()),
            role: Set(spec.role.clone()),
            lang: Set(spec.lang.clone()),
            label: Set(spec.label.clone()),
            prompt: Set(spec.prompt.clone()),
            created_at: Set(now),
            updated_at: Set(now),
        };
        let _ = model.insert(db).await;
        return Ok(spec.prompt.clone());
    }

    // 3. Query in database for default fallback language
    if normalized_lang != DEFAULT_LANG
        && let Some(fallback_record) = role_prompt::Entity::find()
            .filter(role_prompt::Column::Role.eq(&normalized_role))
            .filter(role_prompt::Column::Lang.eq(DEFAULT_LANG))
            .one(db)
            .await?
    {
        return Ok(fallback_record.prompt);
    }

    // 4. Heal English default from defaults.js if even default English is missing
    if let Ok(seed) = load_defaults_from_js()
        && let Some(spec) = seed
            .role_prompts
            .iter()
            .find(|s| s.role == normalized_role && s.lang == DEFAULT_LANG)
    {
        let now = Utc::now();
        let model = role_prompt::ActiveModel {
            id: Set(Uuid::new_v4()),
            role: Set(spec.role.clone()),
            lang: Set(spec.lang.clone()),
            label: Set(spec.label.clone()),
            prompt: Set(spec.prompt.clone()),
            created_at: Set(now),
            updated_at: Set(now),
        };
        let _ = model.insert(db).await;
        return Ok(spec.prompt.clone());
    }

    Ok(format!(
        "You are the {role} agent of the Simply Agent Hub collective. Collaborate cleanly on the channel."
    ))
}

/// Queries the database for a human-readable role label, falling back to `DEFAULT_LANG` ("en").
pub async fn get_or_create_role_label(
    db: &DatabaseConnection,
    role: &str,
    lang: &str,
) -> Result<String, DbErr> {
    let normalized_role = role.trim().to_lowercase();
    let normalized_lang = lang.trim().to_lowercase();

    if let Some(record) = role_prompt::Entity::find()
        .filter(role_prompt::Column::Role.eq(&normalized_role))
        .filter(role_prompt::Column::Lang.eq(&normalized_lang))
        .one(db)
        .await?
    {
        return Ok(record.label);
    }

    if let Ok(seed) = load_defaults_from_js()
        && let Some(spec) = seed
            .role_prompts
            .iter()
            .find(|s| s.role == normalized_role && s.lang == normalized_lang)
    {
        return Ok(spec.label.clone());
    }

    if normalized_lang != DEFAULT_LANG
        && let Some(fallback_record) = role_prompt::Entity::find()
            .filter(role_prompt::Column::Role.eq(&normalized_role))
            .filter(role_prompt::Column::Lang.eq(DEFAULT_LANG))
            .one(db)
            .await?
    {
        return Ok(fallback_record.label);
    }

    if let Ok(seed) = load_defaults_from_js()
        && let Some(spec) = seed
            .role_prompts
            .iter()
            .find(|s| s.role == normalized_role && s.lang == DEFAULT_LANG)
    {
        return Ok(spec.label.clone());
    }

    Ok(format!("🤖 {role}"))
}

/// Queries all available languages from the database table `available_languages`.
pub async fn list_available_languages(
    db: &DatabaseConnection,
) -> Result<Vec<available_language::Model>, DbErr> {
    available_language::Entity::find()
        .order_by_asc(available_language::Column::Code)
        .all(db)
        .await
}

/// Queries or creates an available language record in the database.
pub async fn get_or_create_available_language(
    db: &DatabaseConnection,
    code: &str,
    name: &str,
    native_name: &str,
) -> Result<available_language::Model, DbErr> {
    let normalized_code = code.trim().to_lowercase();
    if let Some(existing) = available_language::Entity::find()
        .filter(available_language::Column::Code.eq(&normalized_code))
        .one(db)
        .await?
    {
        return Ok(existing);
    }

    let now = Utc::now();
    let active = available_language::ActiveModel {
        id: Set(Uuid::new_v4()),
        code: Set(normalized_code),
        name: Set(name.to_string()),
        native_name: Set(native_name.to_string()),
        is_default: Set(false),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    active.insert(db).await
}
