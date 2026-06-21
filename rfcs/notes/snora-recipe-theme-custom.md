# Recipe: `Theme::custom` from Snora Design tokens

**Status.** Recipe
**Format.** RFC-033 nine-section recipe

---

## 1. Purpose

Build an iced `Theme::custom(...)` from a Snora Design `Tokens` preset so
that stock iced widgets — text inputs, sliders, scrollbars, checkboxes,
pick-lists, tooltips — track the active design preset alongside the
Snora-styled widgets.

---

## 2. When to use

- Your app exposes a theme-switching feature (light / dark / high-contrast)
  and wants a coherent appearance across **all** widgets, not just the ones
  it styles explicitly with Snora style functions.
- You use iced's `.theme()` application callback to drive the base
  `Theme` and want it to reflect the active `Tokens` rather than defaulting
  to `Theme::Light` or `Theme::Dark`.

---

## 3. When not to use

- The Light / Dark base-theme approximation is acceptable for your users
  (i.e. you expose only light and dark, not high-contrast).
- Your app uses only Snora-styled widgets and no stock iced widgets.
- You want a simple, mechanical theme switch without carefully mapping
  semantic roles.

---

## 4. Data the app owns

- The active `Tokens` preset (typically stored in a global or app state,
  driven by a user setting).

---

## 5. Snora primitives used

- `snora::design::Tokens` — the active preset struct.
- `snora::design::Palette` — the 18 semantic color roles inside `Tokens`.
- `snora::design::contrast::composite_over` — used when an alpha-composite
  is needed before passing to iced (optional; only if a role has
  transparency).
- `snora::design::style::color::to_iced_color` — converts a
  `snora_design::Color` to an `iced::Color`.

---

## 6. Accessibility notes

This recipe specifically targets **high-contrast fidelity**. Without it,
snora's high-contrast presets apply fully to the widgets an app styles
directly (buttons, cards, notices) but stock iced widgets ignore them —
they fall back to `Theme::Light` / `Theme::Dark` and their standard
contrast ratios.

With this recipe, the 6 core roles that survive the mapping still carry
snora's contrast-verified values into the stock widgets. The remaining 12
roles (`surface` vs `surface_raised`, the `*_text` on-color pairs,
`border`, `focus`, `text_secondary`, `text_muted`) are handled by iced's
own palette-expansion algorithm and will not reproduce snora's hand-tuned
values. This is the best achievable with iced 0.14's `theme::Palette`
structure, and it is a strict improvement over the base `Theme::Light` /
`Theme::Dark` fallback.

---

## 7. Code example

```rust
use iced::{Color, Theme};
use iced::theme;
use snora::design::{Tokens, style::color::to_iced_color};

/// Build an iced `Theme` whose 6 core palette roles are derived from a
/// Snora Design `Tokens` preset.
///
/// # Mapping
///
/// Snora's `Palette` has 18 semantic roles; iced's `theme::Palette` has 6.
/// The 6 that map cleanly:
///
/// | iced role    | snora role        | notes                          |
/// |--------------|-------------------|--------------------------------|
/// | `background` | `surface`         | main window / panel surface    |
/// | `text`       | `text_primary`    | default body text              |
/// | `primary`    | `accent`          | interactive highlight          |
/// | `success`    | `success`         | positive status                |
/// | `warning`    | `warning`         | caution status                 |
/// | `danger`     | `danger`          | destructive / error status     |
///
/// iced then expands these 6 into its full extended palette via its own
/// algorithm (tints, shades, on-color derivations). That expansion cannot
/// reproduce snora's hand-tuned values for the 12 roles that don't
/// survive (`surface_raised`, `*_text` on-colors, `border`, `focus`,
/// `text_secondary`, `text_muted`). This is the best achievable with
/// iced 0.14's `theme::Palette` structure.
pub fn iced_theme_from_tokens(tokens: &Tokens, name: impl Into<String>) -> Theme {
    let p = &tokens.palette;
    let palette = theme::Palette {
        background: to_iced_color(p.surface),
        text:       to_iced_color(p.text_primary),
        primary:    to_iced_color(p.accent),
        success:    to_iced_color(p.success),
        warning:    to_iced_color(p.warning),
        danger:     to_iced_color(p.danger),
    };
    Theme::custom(name.into(), palette)
}
```

### Typical call site (application `.theme()` callback)

```rust
// In the application builder:
.theme(|state| iced_theme_from_tokens(&state.tokens, "app-theme"))

// Or, with a global-token pattern (no state threading):
.theme(|_state| iced_theme_from_tokens(current_tokens(), "app-theme"))
```

where `current_tokens()` resolves the active preset from an `AtomicU8`
or equivalent global (the same pattern used for runtime locale switching).

---

## 8. Customization points

- **`name`** — the string passed to `Theme::custom`; can include the
  preset name for debug visibility ("snora-light", "snora-hc-dark", …).
- **`background` role** — some apps prefer `tokens.palette.background`
  (the true window background) over `surface` (the main content panel
  surface). If your app sets the window background separately, use
  `background` here and `surface` for container styling.
- **Per-widget style overrides** — for the 12 roles iced's expansion
  doesn't reproduce, the existing Snora style functions
  (`style::container::card_surface`, etc.) applied per-widget remain the
  correct tool. This recipe handles the base theme; the per-widget bridge
  handles the rest.

---

## 9. Promotion status

**Recipe** — a copy-paste pattern that introduces no new stable Snora API.
Candidate for promotion to a documented helper in `snora::design::iced_theme`
if downstream demand warrants it. The promoting condition would be at least
two downstream apps using this pattern independently with the same mapping.

### Known downstream usage

- **arama** (offline AI-powered media similarity finder) — uses this
  mapping to drive a four-preset theme setting (light / dark / HC light /
  HC dark). Contributed as the basis for this recipe.
