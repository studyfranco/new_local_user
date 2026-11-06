//! In-memory thread-safe cache for languages and role presets
//!
//! Prevents continuous roundtrips to the database on frequent UI requests
//! (`/api/languages`, `/api/languages/{lang}`, `/api/languages/available`, `/api/roles/presets`).

use crate::entities::available_language;
use serde_json::Value;
use std::collections::HashMap;

/// Cached representation of a Language Pack to avoid database roundtrips.
#[derive(Debug, Clone)]
pub struct CachedLanguagePack {
    pub id: uuid::Uuid,
    pub lang: String,
    pub name: String,
    pub dictionary: Value,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Thread-safe in-memory cache for available languages, language packs, and role presets.
#[derive(Debug, Default, Clone)]
pub struct LanguageCache {
    /// Cached available languages (None = uninitialized or invalidated).
    pub available_languages: Option<Vec<available_language::Model>>,
    /// Cached language packs keyed by language code ("en", "fr", "de", etc.).
    pub language_packs: HashMap<String, CachedLanguagePack>,
    /// Cached role presets keyed by requested language code ("en", "fr", etc.).
    pub role_presets: HashMap<String, Vec<Value>>,
}

impl LanguageCache {
    /// Creates a new empty language cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all cached entries, forcing subsequent reads to reload from the database.
    pub fn invalidate_all(&mut self) {
        self.available_languages = None;
        self.language_packs.clear();
        self.role_presets.clear();
    }

    /// Invalidates cached available languages and language pack dictionaries.
    pub fn invalidate_languages(&mut self) {
        self.available_languages = None;
        self.language_packs.clear();
    }

    /// Invalidates cached role presets.
    pub fn invalidate_roles(&mut self) {
        self.role_presets.clear();
    }
}
