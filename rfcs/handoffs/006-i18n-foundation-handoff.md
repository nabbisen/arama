# Handoff — RFC 006: Multilingual GUI (i18n foundation)

**RFC.** [`rfcs/done/006-i18n-foundation.md`](../done/006-i18n-foundation.md)
**Shipped in.** v0.27.0

---

## 1. Implementation Handoff

### Goal
Introduce runtime translation infrastructure and ship English + Japanese
tables for the Settings and Cache pages plus nav tooltips, with a language
selector in Settings → General.

### Architecture
New zero-dependency `arama-i18n` crate:
- `Locale` enum (`En`, `Ja`), `Default = En`, with `code()`,
  `display_name()`, `all()`.
- Global `AtomicU8` backing `set_locale` / `current_locale` — lock-free,
  callable from any thread (this is the canonical pattern arama later reuses
  for the theme global).
- `t(key) -> String` with fallback chain **current locale → English → raw
  key**, so missing translations degrade to English rather than blank labels.
- `en.rs` / `ja.rs` are static `match` expressions; the `_` arm returns
  `None`, making missing keys detectable without panicking.

### Settings + propagation
- `Settings.locale: Locale` with `#[serde(default)]` — existing settings
  files load as English without error.
- `set_locale(settings.locale)` called at startup in `App::new`.
- Language selector: two buttons (EN / 日本語) in `GeneralSettings`, primary
  when active. `LocaleChanged` bubbles `GeneralSettings → SettingsDialog →
  App`, where `set_locale` is called and the setting saved. Immediate effect,
  no restart.

### Watch out for
- `t()` returns `String`; wrap in `text(...)` for `button`/`text` widgets.
- Japanese strings use brace unicode escapes (`\u{...}`), including full-width
  punctuation (`\u{ff1a}`), not bare `\uXXXX`.

---

## 2. Task Breakdown / PR Plan

### PR 1 — Crate + plumbing (no visible change yet)
1. New `crates/i18n/` (lib + en/ja tables), register in workspace.
2. `Settings.locale` field; `set_locale` at startup; `arama-i18n` dep on
   `env`, `ui/widgets`, `ui/main`, `app`.

### PR 2 — Selector + translate Settings/Cache/tooltips
3. `LocaleChanged` message through GeneralSettings → SettingsDialog → App.
4. Language selector UI in Settings → General.
5. Apply `t()` to Settings tabs, Cache page, nav tooltips; add the keys.

---

## 3. Acceptance / QA Checklist

### Automated
- [ ] `cargo check --workspace` — zero errors, zero warnings.
- [ ] `cargo test -p arama-i18n` — `locale_round_trip`,
      `translation_and_fallback` pass (added later; include if present).
- [ ] Existing `settings.json` without `locale` loads as English.

### Manual
- [ ] Settings → General shows EN / 日本語 buttons; active one is highlighted.
- [ ] Selecting 日本語 translates Settings tabs, the Cache page, and nav
      tooltips immediately (no restart).
- [ ] Selecting EN restores English.
- [ ] Restart preserves the chosen locale.

### Regression
- [ ] A key present in English but absent in Japanese falls back to English
      (not blank, not the raw key).
- [ ] A genuinely unknown key renders the raw key string (developer-visible,
      not a panic).
