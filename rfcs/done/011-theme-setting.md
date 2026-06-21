# RFC 011 — Application theme setting (light / dark / high-contrast)

**Status.** Implemented (v0.33.0)
**Tracks.** A user-selectable application theme that leverages all four
Snora Design presets (`light`, `dark`, `high_contrast_light`,
`high_contrast_dark`), building on the design-system adoption in RFC 010.
**Touches.** `crates/theme/` (mutable global + `ThemePreset` enum +
iced-`Theme` bridge), `env/src/config/settings.rs` (`theme` field),
`crates/ui/widgets/.../general_settings/` (selector UI + message),
`crates/ui/widgets/src/dialog/settings_dialog/` (message bubbling),
`app/src/core.rs` (init + `.theme()` callback + save),
`app/src/core/update.rs` (change handler), `crates/i18n/src/{en,ja}.rs`
(selector labels), `docs/src/users/settings.md`.

---

## 1. Motivation

RFC 010 wired arama's button styling to a single Snora Design preset
(`Tokens::light()`), hardcoded in `arama-theme` behind a write-once
`OnceLock`. snora ships four presets; arama exposes one. This RFC makes
the preset a runtime, persisted user setting.

The benefit is twofold:

1. **User preference** — light and dark are table-stakes for a desktop
   media application; users browsing image libraries in a dark room
   expect a dark UI.
2. **Accessibility** — the two high-contrast presets exist specifically
   for low-vision users. snora's contrast tests guarantee these presets
   meet (and exceed) WCAG AA; exposing them is a direct accessibility win
   that costs arama almost nothing, since the presets already exist.

---

## 2. The core design problem: three layers must move together

A naive implementation would switch only the snora **button** tokens.
That is wrong and would produce a visibly broken UI. arama's appearance
is governed by **three** independent styling layers, and a theme switch
must move all three coherently:

| Layer | What it colors | Driven by |
|---|---|---|
| **A. Snora button tokens** | the four button styles in `arama-theme` | `arama_theme::{primary,ghost,secondary,danger}` → `snora::design::style::button::*` with the active `Tokens` |
| **B. Snora container tokens** | card / surface backgrounds (`snora::design::style::container::card_*`) — not yet used by arama, but reserved | same active `Tokens` |
| **C. Base iced theme** | the application window background, default text color, scrollbars, text inputs, sliders, checkboxes, pick-lists, tooltips — every stock iced widget arama uses | the iced `Theme` returned from the application's `.theme()` callback |

Layer C is the subtle one. arama currently sets **no** `.theme()`
callback, so iced uses its built-in default (`Theme::Light`). Today that
happens to match `Tokens::light()`, which is why RFC 010 looked complete.
The moment a user selects "Dark", if only layer A switches, the buttons
turn dark-on-light — the window stays light because layer C never moved.

**Therefore this RFC introduces a `.theme()` callback** that returns an
iced `Theme` derived from the same preset that drives the snora tokens,
keeping A/B/C in lockstep.

### 2.1 Mapping a preset to an iced `Theme`

iced 0.14 ships built-in `Theme::Light` and `Theme::Dark`. snora does not
provide a `Tokens` → iced `Theme` bridge (its bridge is at the per-widget
style-function level, layers A/B). We map conservatively:

| `ThemePreset` | snora `Tokens` (layers A/B) | iced base `Theme` (layer C) |
|---|---|---|
| `Light` | `Tokens::light()` | `Theme::Light` |
| `Dark` | `Tokens::dark()` | `Theme::Dark` |
| `HighContrastLight` | `Tokens::high_contrast_light()` | `Theme::Light` |
| `HighContrastDark` | `Tokens::high_contrast_dark()` | `Theme::Dark` |

The high-contrast presets reuse the matching light/dark iced base theme
for layer C. This is a deliberate, documented approximation whose root
cause is precise: snora's `Palette` has 18 semantic roles; iced 0.14's
`theme::Palette` has 6 (`background`, `text`, `primary`, `success`,
`warning`, `danger`). A lossy mapping of the 6 core roles is
straightforward, but iced then **expands** those 6 into its full extended
palette via its own algorithm — and that expansion cannot reproduce
snora's hand-tuned high-contrast values. The roles that don't survive the
round-trip are exactly the ones that matter most for high-contrast:
`surface` vs `surface_raised`, the `*_text` on-color pairs, `border`,
`focus`, `text_secondary`, `text_muted`. So it is not that the mapping
is impossible; it is that the expansion is lossy in precisely the
high-contrast-critical roles. snora's position (`design-decisions.md`,
RFC-025): "No replacement for iced `Theme`" — the per-widget style bridge
is deliberate, not a gap to be filled by snora. The high-contrast
presets still deliver their full benefit on the surfaces arama controls
directly (buttons via layer A, any future cards via layer B); the base
iced widgets fall back to standard light/dark. This is honestly described
to the user (see §6).

A future arama task may hand-roll a `Theme::custom(...)` from the 6
mappable palette roles to make layer C partially high-contrast for stock
widgets. That is an arama-side exercise — snora will not provide a
full-palette bridge by design — and it is bounded: only the 6 core roles
can be mapped; iced's expansion handles the rest.

---

## 3. `arama-theme` changes

### 3.1 From `OnceLock` to a mutable atomic global

The current `OnceLock<Tokens>` is write-once; a runtime setting needs a
mutable global. Follow the **exact pattern arama already uses for i18n**
(`arama_i18n` stores the locale in a global `AtomicU8`):

```rust
use std::sync::atomic::{AtomicU8, Ordering};

/// User-selectable application theme preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default,
         serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreset {
    #[default]
    Light,
    Dark,
    HighContrastLight,
    HighContrastDark,
}

static THEME_ID: AtomicU8 = AtomicU8::new(0 /* Light */);

pub fn set_theme(preset: ThemePreset) {
    THEME_ID.store(preset as u8, Ordering::Relaxed);
}

pub fn current_theme() -> ThemePreset {
    match THEME_ID.load(Ordering::Relaxed) {
        1 => ThemePreset::Dark,
        2 => ThemePreset::HighContrastLight,
        3 => ThemePreset::HighContrastDark,
        _ => ThemePreset::Light,
    }
}
```

`ThemePreset` lives in `arama-theme`, **not** in `arama-env`. Rationale:
`arama-env` is dependency-light (no iced, no snora) and several crates
depend on it; `arama-theme` already depends on iced+snora and is the
natural home for a theme type. `arama-env::Settings` will reference it
via a new `arama-theme` dependency on `arama-env` — see §4 for the
dependency-direction analysis.

### 3.2 Tokens lookup becomes preset-driven

```rust
fn tokens() -> Tokens {
    match current_theme() {
        ThemePreset::Light => Tokens::light(),
        ThemePreset::Dark => Tokens::dark(),
        ThemePreset::HighContrastLight => Tokens::high_contrast_light(),
        ThemePreset::HighContrastDark => Tokens::high_contrast_dark(),
    }
}
```

Note the return type changes from `&'static Tokens` to an owned `Tokens`.
The style functions already clone tokens into their closures (snora's
helpers take `&Tokens` and the closure owns a clone), so returning owned
`Tokens` is correct and the four `pub fn primary/ghost/secondary/danger`
bodies are unchanged apart from `tokens()` now returning a value:

```rust
pub fn primary(_theme: &Theme, status: button::Status) -> button::Style {
    snora::design::style::button::primary(&tokens(), status)
}
```

`Tokens` is `Clone` and small (a handful of palette/scale structs); one
clone per styled button per frame is negligible in iced's retained-mode
`view()`. (If profiling ever shows this matters, a `thread_local!` cache
keyed on `THEME_ID` is a drop-in optimization — explicitly noted as a
non-goal here to avoid premature complexity.)

### 3.3 The iced-theme bridge (layer C)

```rust
/// The base iced `Theme` for the active preset (layer C).
pub fn iced_theme() -> Theme {
    match current_theme() {
        ThemePreset::Light | ThemePreset::HighContrastLight => Theme::Light,
        ThemePreset::Dark  | ThemePreset::HighContrastDark  => Theme::Dark,
    }
}
```

---

## 4. Dependency direction

`Settings` (in `arama-env`) needs to store a `ThemePreset`. `ThemePreset`
lives in `arama-theme`. So `arama-env` must depend on `arama-theme`.

**Check for cycles.** Does `arama-theme` depend on `arama-env`? No — it
depends only on `iced` and `snora`. So adding `arama-env → arama-theme`
is acyclic. ✔

However, `arama-theme` depends on `iced` + `snora`, which would make
`arama-env` (currently a light leaf crate) transitively pull in the GUI
stack. That is undesirable: `arama-env` is depended on widely and kept
light deliberately.

**Resolution.** `ThemePreset` is split so the *type* has no heavy deps:

- The `ThemePreset` enum (a plain `#[repr(u8)]`-style data enum with
  serde derives) is defined in `arama-env` itself, alongside `Locale`'s
  sibling settings types. It has **zero** GUI dependencies — it is pure
  data, exactly like `CacheLookupStrategy` and `TargetMediaType` already
  in `arama-env`.
- `arama-theme` depends on `arama-env` (already light) and maps
  `ThemePreset` → `Tokens` / iced `Theme`. The global atomic + `set_theme`
  / `current_theme` / `tokens` / `iced_theme` live in `arama-theme`.

Final direction: `arama-theme → arama-env` (for the `ThemePreset` enum) +
`arama-theme → iced, snora`. `arama-env` stays GUI-free. No cycle.

This mirrors how `Locale` is defined in `arama-i18n` and re-used by
`arama-env::Settings` — except here the enum is so trivial and so clearly
a *setting* that it belongs in `arama-env` next to the other config
enums, and `arama-theme` reads it. Both placements are defensible; this
RFC chooses **enum in `arama-env`** to keep `arama-theme` free of a
reverse dependency and to group it with the other persisted setting
enums.

---

## 5. Settings persistence

```rust
// env/src/config/settings.rs
pub struct Settings {
    // …existing…
    #[serde(default)]
    pub theme: ThemePreset,   // default: ThemePreset::Light
}
```

`#[serde(default)]` ensures existing `settings.json` files (which predate
the field) load cleanly as `Light` — identical to how `locale` was added
in RFC 006. No migration needed.

---

## 6. UI: theme selector in Settings → General

Placed directly beneath the existing **Language** selector, using the
identical four-button pattern (so the code is familiar and the i18n
labels follow the established key convention):

```
Language:  [ English ]  [ 日本語 ]
Theme:     [ Light ] [ Dark ] [ HC Light ] [ HC Dark ]
```

Each button is styled `arama_theme::primary` when it is the active preset
and `arama_theme::ghost` otherwise — exactly like the locale buttons.
Pressing a button emits `GeneralSettings::Message::ThemeChanged(ThemePreset)`.

### 6.1 Labels and i18n keys

New keys in `en.rs` / `ja.rs`:

```
settings.general.theme              "Theme"            / "テーマ"
settings.general.theme.light        "Light"            / "ライト"
settings.general.theme.dark         "Dark"             / "ダーク"
settings.general.theme.hc_light     "High contrast light" / "ハイコントラスト（明）"
settings.general.theme.hc_dark      "High contrast dark"  / "ハイコントラスト（暗）"
```

A short note rendered under the high-contrast options (secondary text)
honestly documents the layer-C approximation from §2.1:

```
settings.general.theme.hc_note
  "High-contrast affects arama's own controls; some standard widgets
   use the base light/dark theme."
  / "ハイコントラストは arama 独自のコントロールに適用されます。一部の
     標準ウィジェットは基本のライト/ダークテーマを使用します。"
```

---

## 7. Message propagation (mirrors `LocaleChanged` exactly)

```
GeneralSettings::Message::ThemeChanged(ThemePreset)
  └─ bubbles to ─► SettingsDialog::Message::ThemeChanged(ThemePreset)
       └─ app handler:
            self.settings.theme = preset;
            arama_theme::set_theme(preset);   // layers A/B take effect next frame
            self.save_settings();
            // layer C: the iced .theme() callback reads current_theme()
            //          on the next render automatically — no extra wiring.
```

Because iced calls the application `.theme()` callback every render, and
that callback returns `arama_theme::iced_theme()` which reads the same
global, layer C updates with **no additional plumbing** beyond
registering the callback once.

### 7.1 App wiring

```rust
// app/src/core.rs — App::start()
iced::application(App::new, App::update, App::view)
    .subscription(App::subscription)
    .settings(App::settings())
    .theme(|_state| arama_theme::iced_theme())   // NEW — layer C
    .run()

// App::new() — after loading settings, alongside set_locale:
arama_theme::set_theme(settings.theme);

// save_settings() / the Settings { } reconstructions:
theme: self.settings.theme,
```

---

## 8. Testing

`arama-theme` is currently dependency-heavy at test time (it links iced +
snora), so per RFC's testing philosophy we add **only** what is cheap and
meaningful and place it where it does not drag the GUI stack:

1. **`ThemePreset` round-trip** (in `arama-env`, GUI-free): `as u8`
   discriminant ↔ `current_theme()` decode for all four variants;
   serde round-trip (`Light` ↔ `"light"`, etc.). This guards the atomic
   encode/decode and the serde rename mapping.
2. **No `iced_test` / Simulator tests** — same rationale as the
   `iced_test` evaluation (RFC notes): not worth linking the GUI stack.

The `set_theme` / `current_theme` global is exercised by the round-trip
test using the same single-function-sequence approach used for the i18n
locale test (avoids cross-test global-state interference).

---

## 9. Alternatives considered

- **Switch only snora tokens (layers A/B), ignore layer C.** Rejected:
  produces dark buttons on a light window. The `.theme()` callback is
  mandatory for a coherent result.
- **Full `Tokens` → `Theme::custom` bridge for layer C.** Out of scope
  for snora by design ("theme-aware, not theme-owning"). A future arama
  task may hand-roll `Theme::custom` from the 6 mappable palette roles;
  this is an arama-side exercise with a known caveat: only 6 of snora's
  18 roles survive iced's palette-expansion algorithm, and the 12 that
  don't are exactly the high-contrast-critical ones (`surface` variants,
  `*_text` on-colors, `border`, `focus`, `text_secondary/muted`).
- **Auto-detect OS dark mode.** Out of scope. iced 0.14 has no portable
  OS-theme query; an explicit user setting is simpler, testable, and
  predictable. Could be a future enhancement layered on top (an `Auto`
  variant that resolves to `Light`/`Dark`).
- **Per-preset `thread_local!` `Tokens` cache.** Rejected as premature;
  the clone is negligible. Documented as a drop-in optimization if ever
  needed.

---

## 10. Touches checklist

| File / module | Change |
|---|---|
| `env/src/config/settings.rs` | `ThemePreset` enum (pure data) + `theme` field (`serde(default)`) + tests |
| `crates/theme/Cargo.toml` | add `arama-env` dep |
| `crates/theme/src/lib.rs` | `OnceLock` → `AtomicU8`; `set_theme`/`current_theme`; preset-driven `tokens()`; `iced_theme()` |
| `crates/ui/widgets/.../general_settings.rs` | `theme: ThemePreset` field + ctor param |
| `crates/ui/widgets/.../general_settings/message.rs` | `ThemeChanged(ThemePreset)` |
| `crates/ui/widgets/.../general_settings/update.rs` | handle `ThemeChanged` |
| `crates/ui/widgets/.../general_settings/view.rs` | theme selector row + hc note |
| `crates/ui/widgets/src/dialog/settings_dialog/message.rs` | `ThemeChanged(ThemePreset)` |
| `crates/ui/widgets/src/dialog/settings_dialog/update.rs` | bubble `ThemeChanged` |
| `crates/ui/widgets/src/dialog/settings_dialog.rs` | pass `theme` to `GeneralSettings::new` |
| `crates/i18n/src/{en,ja}.rs` | 6 new keys |
| `app/src/core.rs` | `set_theme` init; `.theme()` callback; save/reconstruct `theme` |
| `app/src/core/update.rs` | `ThemeChanged` handler |
| `docs/src/users/settings.md` | document the Theme setting + hc caveat |

## Open questions

None. (The high-contrast layer-C fidelity upgrade is a named future RFC,
not an open question for this one.)
