# RFC 006 — Multilingual GUI (i18n foundation)

**Status.** Implemented (v0.27.0)
**Tracks.** The project requirement "The GUI must support multiple
languages (i18n)", and the pending note in RFC 001 ("i18n work —
separate RFC"). A dependency-free translation infrastructure, English
and Japanese locale tables, and a language selector in Settings → General.
**Touches.** New `crates/i18n/` workspace member, `env/src/config/settings.rs`,
`crates/ui/widgets/src/dialog/settings_dialog/` (GeneralSettings),
`crates/ui/main/src/core/views/cache_page/`, `app/src/core/update.rs`
(locale initialisation + change handler).

## Summary

All user-visible strings in arama currently live in the source code as
Rust string literals. This RFC introduces the infrastructure to look
them up from a locale table at runtime, ships English and Japanese
tables for the Settings and Cache pages (the surfaces most recently
worked on), and adds a language selector (EN / 日本語) to Settings →
General.

Untranslated views (gallery, setup wizard, focus dialog, similar-pairs
dialog) fall back to their existing string literals in Phase 2 sweep.

## Design

### arama-i18n crate

A new, zero-dependency crate:

```
crates/i18n/
  Cargo.toml
  src/
    lib.rs     — public API
    en.rs      — English table
    ja.rs      — Japanese table
```

**Public API:**

```rust
/// All supported locales.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Locale { En, Ja }

/// Set the active locale (global, thread-safe).
pub fn set_locale(locale: Locale);

/// Return the current locale.
pub fn current_locale() -> Locale;

/// Look up `key` in the current locale.
/// Falls back to the English table when a key is absent, then to the
/// key string itself, so missing translations degrade gracefully.
pub fn t(key: &str) -> String;
```

The global locale is stored in an `AtomicU8`. Tables are static match
expressions keyed by `&str`, returning `Option<&'static str>`. The
`String` return type of `t()` avoids lifetime complexity at call sites.

### Settings struct

```rust
pub struct Settings {
    // ...existing...
    #[serde(default)]
    pub locale: Locale,  // default: Locale::En
}
```

### Language selector (GeneralSettings)

Two `button` widgets below the Similarity slider, styled `primary` for
the active locale and `text` for the inactive one:

```
Language:  [ EN ]  [ 日本語 ]
```

Emits `GeneralSettings::Message::LocaleChanged(Locale)`, bubbled to
`SettingsDialog::Message::LocaleChanged(Locale)`, then to `App` where
`set_locale(locale)` is called immediately and the setting is saved.

### Phase coverage

| Surface | Phase |
|---|---|
| Settings page (all tabs) | **1 (this RFC)** |
| Cache page | **1 (this RFC)** |
| Side-nav tooltips | **1 (this RFC)** |
| Gallery, focus dialog, similar-pairs dialog | Phase 2 |
| Setup wizard | Phase 2 |

### Translation tables

See `crates/i18n/src/en.rs` and `crates/i18n/src/ja.rs`.
Key namespace convention: `component.element` (e.g.
`settings.tab.general`, `cache.column.files`).

## Touches in detail

| File | Change |
|---|---|
| `crates/i18n/` | New crate |
| `Cargo.toml` (workspace) | Add `arama-i18n` member + dep |
| `env/src/config/settings.rs` | `locale: Locale` with `serde(default)` |
| `crates/ui/widgets/Cargo.toml` | Add `arama-i18n` dep |
| `crates/ui/main/Cargo.toml` | Add `arama-i18n` dep |
| `crates/ui/widgets/src/dialog/settings_dialog/tab/general_settings/` | Language selector + `LocaleChanged(Locale)` message |
| `crates/ui/widgets/src/dialog/settings_dialog/` | Bubble `LocaleChanged` |
| `crates/ui/widgets/src/dialog/settings_dialog/tab/*/view.rs` | `t("key")` calls |
| `crates/ui/main/src/core/views/cache_page/view.rs` | `t("key")` calls |
| `app/src/core.rs` | `set_locale(settings.locale)` at startup |
| `app/src/core/update.rs` | Handle `LocaleChanged` |
| `docs/src/users/settings.md` | Document language selector |

## Dependency changes

| Package | Change |
|---|---|
| `arama-i18n` | new workspace crate (zero external deps) |
| `serde` | already a `arama-i18n` dep (for `Locale` derive) |

## Open questions

None.
