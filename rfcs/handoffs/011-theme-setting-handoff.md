# Handoff — RFC 011: Application theme setting (light / dark / high-contrast)

**RFC.** [`rfcs/done/011-theme-setting.md`](../done/011-theme-setting.md)
**Shipped in.** v0.33.0
**Depends on.** RFC 010 (Snora Design adoption, `arama-theme` crate)

---

## 1. Implementation Handoff

### Goal
Let the user pick among the four Snora Design presets (light / dark /
high-contrast light / high-contrast dark) from Settings → General, persist
the choice, and apply it immediately across all three styling layers.

### The one thing not to get wrong
A theme switch must move **three** layers together, or the UI looks broken:

| Layer | Surface | Driver |
|---|---|---|
| A — snora button tokens | the 4 `arama-theme` button styles | `arama_theme::{primary,ghost,secondary,danger}` resolving the active `Tokens` |
| B — snora container tokens | future card surfaces (reserved) | same active `Tokens` |
| C — base iced theme | window background + all stock iced widgets | the `.theme()` callback returning `arama_theme::iced_theme()` |

If only layer A is switched, dark buttons render on a light window. Layer C
is mandatory.

### Key mechanics
- `arama-theme`'s global is a **mutable `AtomicU8`** (`set_theme` /
  `current_theme`), mirroring the i18n locale pattern — *not* the write-once
  `OnceLock` from RFC 010.
- `ThemePreset` is **pure data in `arama-env`** (GUI-free, sits with the
  other persisted setting enums). `arama-theme` depends on `arama-env` and
  maps the preset to `Tokens` (layers A/B) and iced `Theme` (layer C). No
  dependency cycle; `arama-env` stays GUI-free.
- Layer C maps HC presets to `Theme::Light` / `Theme::Dark`. This is not
  an iced limitation — it is because snora's 18-role `Palette` collapses
  to iced's 6-field `theme::Palette`, and iced's own expansion algorithm
  cannot reproduce the hand-tuned HC values for the 12 roles that don't
  survive (on-color pairs, `surface` variants, `border`, `focus`, etc.).
  snora won't provide a full-palette bridge by design. A future arama
  task may hand-roll `Theme::custom` from the 6 mappable roles.
- The `.theme()` callback must be a **named free function**
  (`fn app_theme(&App) -> iced::Theme`), not a closure — a closure fails
  iced's `ThemeFn` higher-ranked lifetime bound ("implementation of `Fn` is
  not general enough").

### Propagation
Mirrors `LocaleChanged` exactly:
`GeneralSettings::Message::ThemeChanged` → bubbles to
`SettingsDialog::Message::ThemeChanged` → app handler sets `settings.theme`,
calls `arama_theme::set_theme`, saves. Layer C needs no extra plumbing — iced
calls `.theme()` every render and it reads the same global.

---

## 2. Task Breakdown / PR Plan

Recommend **two PRs** (foundation, then UI/wiring) so the GUI-free pieces can
be reviewed and merged independently.

### PR 1 — Foundation (`arama-env` + `arama-theme`)
1. Add `ThemePreset` enum to `env/src/config/settings.rs` (serde
   `snake_case`, `#[default] Light`, `all()`), plus `theme` field on
   `Settings` with `#[serde(default)]`.
2. Add the two GUI-free round-trip tests + `serde_json` dev-dependency.
3. Rewrite `crates/theme/src/lib.rs`: `AtomicU8` global, `set_theme`,
   `current_theme`, preset-driven `tokens()` (now returns owned `Tokens`),
   `iced_theme()`. Add `arama-env` dep to `crates/theme/Cargo.toml`.
- **Reviewable in isolation:** `cargo test -p arama-env`, `cargo check -p arama-theme`.

### PR 2 — UI + app wiring
4. i18n: 6 new keys in `en.rs` / `ja.rs` (`settings.general.theme[.*]`).
5. `GeneralSettings`: `theme` field + ctor param, `ThemeChanged` message +
   handler, theme selector row + HC note in the view.
6. `SettingsDialog`: `ThemeChanged` in message enum, bubble in update, pass
   `theme` to `GeneralSettings::new`.
7. `app/src/core.rs`: `set_theme` init, `.theme(app_theme)` callback +
   `app_theme` free fn, save/reconstruct `theme`.
8. `app/src/core/update.rs`: `ThemeChanged` handler.
9. Docs: Theme row in `docs/src/users/settings.md`.

---

## 3. Acceptance / QA Checklist

### Automated
- [ ] `cargo check --workspace` — zero errors, zero warnings.
- [ ] `cargo test -p arama-env` — `theme_preset_discriminants` and
      `theme_preset_serde_round_trip` pass.
- [ ] Existing `settings.json` without a `theme` key loads as `Light`
      (serde default) — no parse error.

### Manual — layer coherence
- [ ] Select **Dark**: window background, text inputs, scrollbars, sliders,
      checkboxes, pick-lists, tooltips all darken (layer C), and buttons
      adopt the dark token palette (layer A). No light-on-dark or
      dark-on-light mismatch.
- [ ] Select **Light**: returns to the light appearance.
- [ ] Select **High contrast light** / **High contrast dark**: arama's own
      buttons show the high-contrast palette; stock iced widgets use the
      matching base light/dark (expected, per the HC note).
- [ ] The active preset's button is `primary`-styled; the others `ghost`.

### Manual — persistence & immediacy
- [ ] Change applies with no restart.
- [ ] Restart the app: the previously selected theme is restored from
      `settings.json`.
- [ ] Switching theme while a caching run is in progress does not interrupt
      the run.

### Regression
- [ ] Language selector still works (lives directly above the theme row).
- [ ] All other Settings → General controls (media types, sub-dir depth,
      similarity slider) unchanged.
