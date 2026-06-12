//! # arama-i18n
//!
//! Lightweight internationalisation support for arama (RFC 006).
//!
//! ## Usage
//!
//! ```rust,no_run
//! use arama_i18n::{Locale, set_locale, t};
//!
//! // Set once at application startup from the stored setting.
//! set_locale(Locale::Ja);
//!
//! // Translate anywhere; falls back gracefully on missing keys.
//! let label = t("settings.tab.general");  // → "一般"
//! ```
//!
//! ## Design
//!
//! The active locale is stored in a global `AtomicU8` so `set_locale`
//! and `t` are lock-free and safe to call from any thread.
//!
//! Translation tables live in `en.rs` and `ja.rs` as static `match`
//! expressions. Keys follow a `component.element` convention
//! (e.g. `settings.tab.general`, `cache.column.files`).
//!
//! Missing keys fall back first to the English table, then to the raw
//! key string, so untranslated views degrade to English rather than
//! showing blank labels.

use std::sync::atomic::{AtomicU8, Ordering};

mod en;
mod ja;

// ---------------------------------------------------------------------------
// Locale enum
// ---------------------------------------------------------------------------

/// All supported locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    #[default]
    En,
    Ja,
}

impl Locale {
    /// BCP-47 code for this locale.
    pub fn code(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Ja => "ja",
        }
    }

    /// Human-readable name in the locale's own script.
    pub fn display_name(&self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::Ja => "日本語",
        }
    }

    /// All supported locales in display order.
    pub fn all() -> &'static [Locale] {
        &[Locale::En, Locale::Ja]
    }
}

// ---------------------------------------------------------------------------
// Global locale state
// ---------------------------------------------------------------------------

static LOCALE_ID: AtomicU8 = AtomicU8::new(0 /* Locale::En */);

/// Set the active locale. Safe to call from any thread.
pub fn set_locale(locale: Locale) {
    LOCALE_ID.store(locale as u8, Ordering::Relaxed);
}

/// Return the currently active locale.
pub fn current_locale() -> Locale {
    match LOCALE_ID.load(Ordering::Relaxed) {
        1 => Locale::Ja,
        _ => Locale::En,
    }
}

// ---------------------------------------------------------------------------
// Translation lookup
// ---------------------------------------------------------------------------

/// Look up `key` in the current locale.
///
/// Fallback chain: current locale → English → raw key string.
pub fn t(key: &str) -> String {
    // Try the current locale.
    let get = match current_locale() {
        Locale::En => en::get,
        Locale::Ja => ja::get,
    };
    if let Some(s) = get(key) {
        return s.to_owned();
    }

    // Fall back to English (handles partially-translated locales).
    if !matches!(current_locale(), Locale::En) {
        if let Some(s) = en::get(key) {
            return s.to_owned();
        }
    }

    // Last resort: return the key itself.
    key.to_owned()
}
