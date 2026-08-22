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
    if !matches!(current_locale(), Locale::En)
        && let Some(s) = en::get(key)
    {
        return s.to_owned();
    }

    // Last resort: return the key itself.
    key.to_owned()
}

/// Look up `key` via [`t`], then substitute each `(placeholder, value)`
/// pair by literal replacement (e.g. `("{count}", "3")`).
///
/// Placeholders live inside the translation text itself, so their order
/// is entirely up to the translation, not fixed by the call site — the
/// same value can appear anywhere a locale's own word order puts it,
/// appear more than once, or be omitted (Task 034).
pub fn t_with(key: &str, args: &[(&str, &str)]) -> String {
    let mut resolved = t(key);
    for (placeholder, value) in args {
        resolved = resolved.replace(placeholder, value);
    }
    resolved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_round_trip() {
        assert_eq!(Locale::En.code(), "en");
        assert_eq!(Locale::Ja.code(), "ja");
        assert_eq!(Locale::all().len(), 2);
    }

    // Locale state is global (AtomicU8), so all switching assertions live in
    // one test to keep them ordered and free of cross-test interference.
    #[test]
    fn translation_and_fallback() {
        // English (default explicit).
        set_locale(Locale::En);
        assert_eq!(t("settings.tab.general"), "General");

        // Japanese.
        set_locale(Locale::Ja);
        assert_eq!(t("settings.tab.general"), "\u{4e00}\u{822c}");

        // Unknown key falls back to the key string itself, in any locale.
        assert_eq!(t("no.such.key"), "no.such.key");
        set_locale(Locale::En);
        assert_eq!(t("no.such.key"), "no.such.key");
    }

    #[test]
    fn t_with_substitutes_named_placeholders_and_leaves_none_behind() {
        set_locale(Locale::En);
        assert_eq!(
            t_with("startup.scan_errors_total", &[("{count}", "3")]),
            "3 total scan errors"
        );

        set_locale(Locale::Ja);
        let ja = t_with("startup.scan_errors_total", &[("{count}", "3")]);
        assert!(ja.contains('3'), "the substituted value must appear: {ja}");
        assert!(
            !ja.contains("{count}"),
            "the placeholder token must not survive substitution: {ja}"
        );
        set_locale(Locale::En);
    }

    /// Task 034: every interpolated key it added, verified in both
    /// locales in one place - not spread across the `app` crate's own
    /// (much larger, much more parallel) test binary, where a
    /// locale-mutating test was found to race pre-existing tests that
    /// assert English text without expecting the locale to move under
    /// them. `arama-i18n`'s own suite is small and already follows the
    /// "one test, ordered" discipline (see the comment above
    /// `translation_and_fallback`), which is what makes mutating the
    /// global locale here safe. `app`'s own tests for the functions that
    /// call these keys (`settings_error_message`, `walk_errors_summary`,
    /// `cache_prune_complete_body`/`cache_prune_partial_body`,
    /// `embedding_report_summary`, `migrate_directory`) stay locale-
    /// neutral (English/default only) for the same reason.
    ///
    /// Some keys reach `app/src/core/data_locations.rs`'s two failure
    /// branches that a unit test cannot easily drive (the old settings
    /// manager reads from the process's current directory, not an
    /// injectable path) - verified directly against the key here
    /// instead, same as `notice.migration_failed.body` below.
    #[test]
    fn task_034_interpolated_keys_substitute_correctly_in_both_locales() {
        for locale in Locale::all() {
            set_locale(*locale);

            let write_error = t_with(
                "notice.settings_migration_failed.write_error.body",
                &[("{path}", "/new/settings.json"), ("{err}", "disk full")],
            );
            assert!(
                write_error.contains("/new/settings.json"),
                "{locale:?}: {write_error}"
            );
            assert!(
                write_error.contains("disk full"),
                "{locale:?}: {write_error}"
            );
            assert!(!write_error.contains('{'), "{locale:?}: {write_error}");

            let read_error = t_with(
                "notice.settings_migration_failed.read_error.body",
                &[("{err}", "permission denied")],
            );
            assert!(
                read_error.contains("permission denied"),
                "{locale:?}: {read_error}"
            );
            assert!(!read_error.contains('{'), "{locale:?}: {read_error}");

            let migration = t_with(
                "notice.migration_failed.body",
                &[
                    ("{kind}", &t("notice.migration.kind_cache")),
                    ("{legacy}", "/old/cache"),
                    ("{new}", "/new/cache"),
                    ("{err}", "disk full"),
                ],
            );
            assert!(migration.contains("/old/cache"), "{locale:?}: {migration}");
            assert!(migration.contains("/new/cache"), "{locale:?}: {migration}");
            assert!(migration.contains("disk full"), "{locale:?}: {migration}");
            assert!(!migration.contains('{'), "{locale:?}: {migration}");

            let prune_complete = t_with(
                "toast.cache_prune_complete.body",
                &[("{count}", "42"), ("{size}", "1.2 GB")],
            );
            assert!(
                prune_complete.contains("42"),
                "{locale:?}: {prune_complete}"
            );
            assert!(
                prune_complete.contains("1.2 GB"),
                "{locale:?}: {prune_complete}"
            );
            assert!(
                !prune_complete.contains('{'),
                "{locale:?}: {prune_complete}"
            );

            let prune_partial = t_with(
                "toast.cache_prune_partial.body",
                &[("{count}", "7"), ("{size}", "300.0 MB")],
            );
            assert!(prune_partial.contains('7'), "{locale:?}: {prune_partial}");
            assert!(
                prune_partial.contains("300.0 MB"),
                "{locale:?}: {prune_partial}"
            );
            assert!(!prune_partial.contains('{'), "{locale:?}: {prune_partial}");
        }
        set_locale(Locale::En);
    }

    /// Every plain (non-interpolated) key Task 034 added must resolve to
    /// real translated text, not fall back to the raw key string, in
    /// both locales - the same guarantee
    /// `similarity_absence_state_keys_resolve_to_real_text_in_both_locales`
    /// below already gives the pre-existing similarity keys.
    #[test]
    fn task_034_plain_keys_resolve_to_real_text_in_both_locales() {
        let keys = [
            "startup.location_error.settings_resolve",
            "startup.location_error.settings_create",
            "startup.location_error.data_resolve",
            "startup.location_error.data_create",
            "startup.location_error.cache_resolve",
            "startup.location_error.cache_create",
            "notice.settings_migration_failed.title",
            "notice.data_migration_failed.title",
            "notice.cache_migration_failed.title",
            "notice.migration.kind_data",
            "notice.migration.kind_cache",
            "notice.setup_init_failed.title",
            "notice.setup_init_failed.body",
            "toast.ffmpeg_settings.title",
            "toast.ffmpeg_settings.folder_unsafe.body",
            "toast.ffmpeg_settings.auto_save_failed.body",
            "toast.ffmpeg_settings.validated_save_failed.body",
            "toast.ffmpeg_settings.worker_stopped.body",
            "toast.similarity_pairs.title",
            "toast.similarity_pairs.select_dir_first.body",
            "toast.cache_error.title",
            "toast.cache_reload_failed.title",
            "toast.cache_reload_failed.image_reader.body",
            "toast.cache_reload_failed.video_reader.body",
            "toast.cache_reload_failed.storage_path.body",
            "toast.indexed_with_warnings.title",
            "toast.embedding_error.title",
            "toast.embedding_error.body",
            "toast.cache_clear_failed.title",
            "toast.cache_prune_complete.title",
            "toast.cache_prune_partial.title",
            "toast.cache_prune_failed.title",
            "toast.invalid_directory.title",
            "toast.invalid_directory.body",
            "cache.summary_report.files_skipped",
            "cache.summary_report.cache_writes_failed",
            "cache.summary_report.files_indexed",
            "settings.error.io",
            "settings.error.serialize",
            "settings.error.deserialize",
            "settings.error.invalid_path_component",
            "settings.error.platform",
            "settings.error.generic",
            "context_menu.open_with_default",
            "context_menu.file_manager",
            "header.dir_nav.folder_select_title",
        ];

        for locale in Locale::all() {
            set_locale(*locale);
            for key in keys {
                let resolved = t(key);
                assert_ne!(
                    resolved, key,
                    "{locale:?}: {key} must resolve to translated text, not the raw key"
                );
                assert!(!resolved.is_empty(), "{locale:?}: {key} resolved empty");
            }
        }
        set_locale(Locale::En);
    }

    #[test]
    fn t_with_substitutes_a_placeholder_that_repeats() {
        // notice.migration_failed.body uses {kind} twice - both must
        // resolve, not just the first occurrence.
        set_locale(Locale::En);
        let resolved = t_with(
            "notice.migration_failed.body",
            &[
                ("{kind}", "data"),
                ("{legacy}", "/old"),
                ("{new}", "/new"),
                ("{err}", "disk full"),
            ],
        );
        assert_eq!(resolved.matches("data").count(), 2);
        assert!(
            !resolved.contains('{'),
            "no placeholder token may remain: {resolved}"
        );
    }

    #[test]
    fn setup_ffmpeg_select_resolves_to_real_text_in_both_locales() {
        for get in [en::get, ja::get] {
            let resolved = get("setup.ffmpeg.select").expect("setup.ffmpeg.select must exist");
            assert_ne!(
                resolved, "setup.ffmpeg.select",
                "must resolve to translated text, not the raw key"
            );
            assert!(!resolved.is_empty());
        }
    }

    #[test]
    fn similarity_read_error_resolves_to_real_text_in_both_locales() {
        for get in [en::get, ja::get] {
            let resolved = get("similarity.read_error").expect("similarity.read_error must exist");
            assert_ne!(
                resolved, "similarity.read_error",
                "must resolve to translated text, not the raw key"
            );
            assert!(!resolved.is_empty());
        }
    }

    #[test]
    fn similarity_absence_state_keys_resolve_to_real_text_in_both_locales() {
        for key in [
            "similarity.nothing_indexed",
            "similarity.no_results",
            "similarity.video_unavailable",
        ] {
            for get in [en::get, ja::get] {
                let resolved = get(key).unwrap_or_else(|| panic!("{key} must exist"));
                assert_ne!(
                    resolved, key,
                    "{key} must resolve to translated text, not the raw key"
                );
                assert!(!resolved.is_empty());
            }
        }
    }

    /// Task 019: the two `FilesystemUnavailable` messages must stay
    /// distinct and neither must read as the other's advice — Auto mode has
    /// no single folder to check permissions on, Selected mode has nothing
    /// resembling a `PATH` scan.
    #[test]
    fn ffmpeg_filesystem_unavailable_keys_resolve_and_stay_distinct_in_both_locales() {
        for get in [en::get, ja::get] {
            let auto = get("settings.ai.ffmpeg_filesystem_unavailable_auto")
                .expect("settings.ai.ffmpeg_filesystem_unavailable_auto must exist");
            let selected = get("settings.ai.ffmpeg_filesystem_unavailable_selected")
                .expect("settings.ai.ffmpeg_filesystem_unavailable_selected must exist");
            assert!(!auto.is_empty());
            assert!(!selected.is_empty());
            assert_ne!(
                auto, selected,
                "Auto and Selected filesystem-unavailable text must differ"
            );
        }
        assert_eq!(en::get("settings.ai.ffmpeg_filesystem_unavailable"), None);
    }

    #[test]
    fn ffmpeg_guidance_is_external_platform_neutral_and_has_no_managed_keys() {
        for get in [en::get, ja::get] {
            for key in ["settings.ai.ffmpeg_external", "setup.ffmpeg.external_help"] {
                let guidance = get(key).expect("external ffmpeg guidance must exist");
                assert!(!guidance.contains("brew install"));
            }
            for removed in [
                "settings.ai.ffmpeg_missing",
                "settings.ai.ffmpeg_get",
                "settings.ai.ffmpeg_fetching",
            ] {
                assert_eq!(get(removed), None);
            }
        }
    }
}
