//! In-memory thread-safe cache for languages and role presets
//!
//! Prevents continuous roundtrips to the database on frequent UI requests
//! (`/api/languages`, `/api/languages/{lang}`, `/api/languages/available`, `/api/roles/presets`).
//! Populates on-demand ("au fur et à mesure des appels") with monotonic TTL expiration.

use crate::entities::available_language;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Default cache time-to-live: 5 minutes.
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);

/// Generic timestamped cache entry for monotonic TTL expiration.
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub expires_at: Instant,
}

impl<T> CacheEntry<T> {
    pub fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            expires_at: Instant::now() + ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

/// Cached representation of a Language Pack to avoid database roundtrips.
#[derive(Debug, Clone)]
pub struct CachedLanguagePack {
    pub id: uuid::Uuid,
    pub lang: String,
    pub name: String,
    pub dictionary: Value,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Thread-safe in-memory cache for available languages, language packs, and role presets with TTL.
#[derive(Debug, Clone)]
pub struct LanguageCache {
    /// Monotonic TTL duration for all entries.
    pub ttl: Duration,
    /// Cached available languages (None = uninitialized or expired).
    pub available_languages: Option<CacheEntry<Vec<available_language::Model>>>,
    /// Cached language packs keyed by language code ("en", "fr", "de", etc.).
    pub language_packs: HashMap<String, CacheEntry<CachedLanguagePack>>,
    /// Cached role presets keyed by requested language code ("en", "fr", etc.).
    pub role_presets: HashMap<String, CacheEntry<Vec<Value>>>,
}

impl Default for LanguageCache {
    fn default() -> Self {
        Self::new(DEFAULT_CACHE_TTL)
    }
}

impl LanguageCache {
    /// Creates a new empty language cache with a specified TTL.
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            available_languages: None,
            language_packs: HashMap::new(),
            role_presets: HashMap::new(),
        }
    }

    /// Returns the cached available languages if present and not expired.
    pub fn get_available_languages(&self) -> Option<Vec<available_language::Model>> {
        self.available_languages
            .as_ref()
            .filter(|entry| !entry.is_expired())
            .map(|entry| entry.value.clone())
    }

    /// Caches the available languages with TTL.
    pub fn set_available_languages(&mut self, list: Vec<available_language::Model>) {
        self.available_languages = Some(CacheEntry::new(list, self.ttl));
    }

    /// Returns the cached language pack if present and not expired.
    pub fn get_language_pack(&self, lang: &str) -> Option<CachedLanguagePack> {
        let entry = self.language_packs.get(lang)?;
        if entry.is_expired() {
            None
        } else {
            Some(entry.value.clone())
        }
    }

    /// Caches a language pack for a specific language code with TTL.
    pub fn set_language_pack(&mut self, lang: impl Into<String>, pack: CachedLanguagePack) {
        self.language_packs
            .insert(lang.into(), CacheEntry::new(pack, self.ttl));
    }

    /// Returns the cached role presets for a given language code if present and not expired.
    pub fn get_role_presets(&self, lang: &str) -> Option<Vec<Value>> {
        let entry = self.role_presets.get(lang)?;
        if entry.is_expired() {
            None
        } else {
            Some(entry.value.clone())
        }
    }

    /// Caches role presets for a specific language code with TTL.
    pub fn set_role_presets(&mut self, lang: impl Into<String>, presets: Vec<Value>) {
        self.role_presets
            .insert(lang.into(), CacheEntry::new(presets, self.ttl));
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
