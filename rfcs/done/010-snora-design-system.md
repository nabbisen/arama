# RFC 010 — Adopt the Snora Design system (token-driven button styling)

**Status.** Implemented (v0.32.0)
**Tracks.** Updating snora 0.18.1 → 0.25.0 and adopting its newly-shipped
design system (the `design` feature) to gain WCAG AA-verified,
token-driven button styling in arama.
**Touches.** Workspace `Cargo.toml` (snora version + `design` feature),
new `crates/theme/` (arama-theme), `app/src/core/view.rs`,
`crates/ui/main/.../cache_page/view.rs`,
`crates/ui/main/.../setup/view.rs`,
`crates/ui/widgets/.../general_settings/view.rs`.

## Summary

snora 0.25.0 ships a complete design system (the `snora-design` token
crate + an iced style bridge behind the opt-in `design` feature). It
provides semantic, token-driven button styles whose colors are verified
to meet WCAG AA contrast (≥4.5:1) across four built-in presets. arama
currently styles its buttons with iced's *built-in* styles
(`button::primary`, `button::text`, `button::secondary`,
`button::danger`), which carry no such contrast guarantee.

This RFC does two things:

1. **Version bump 0.18.1 → 0.25.0.** Verified drop-in: every snora
   symbol arama uses (`AppLayout` + builder, `Toast`, `ToastIntent`,
   `ToastPosition`, `render`, `toast::subscription`,
   `toast::sweep_expired`) is unchanged. The single breaking change in
   the range — `Palette::roles()` made test-only in 0.24.0 — is not used
   by arama.

2. **Adopt token-driven button styling** via the `design` feature, using
   a contained global-tokens pattern.

## Why the design feature is low-cost

`snora-design` is an iced-free, **zero-dependency** pure-Rust crate. The
`design` feature on `snora-widgets` adds the style-bridge code but no new
external crates. Build-cost and binary-size impact are minimal.

## Design: arama-theme crate

snora's design style functions take `&Tokens` explicitly:

```rust
snora::design::style::button::primary(tokens: &Tokens, status) -> button::Style
```

Threading a `&Tokens` through every `view()` signature in three UI crates
would be invasive. Instead — consistent with how arama already handles
i18n (a global `AtomicU8` locale + free `t()` function) — a new small
`arama-theme` crate holds the tokens globally and exposes drop-in style
functions matching iced's built-in shape:

```rust
// crates/theme/src/lib.rs
use std::sync::OnceLock;
use iced::{Theme, widget::button};
use snora::design::Tokens;

static TOKENS: OnceLock<Tokens> = OnceLock::new();
fn tokens() -> &'static Tokens { TOKENS.get_or_init(Tokens::light) }

pub fn primary(_t: &Theme, s: button::Status) -> button::Style {
    snora::design::style::button::primary(tokens(), s)
}
pub fn ghost(_t: &Theme, s: button::Status) -> button::Style { /* … */ }
pub fn secondary(_t: &Theme, s: button::Status) -> button::Style { /* … */ }
pub fn danger(_t: &Theme, s: button::Status) -> button::Style { /* … */ }
```

Because each function has iced's exact `fn(&Theme, Status) -> Style`
shape, call sites change only the function path — no signature churn.

### Style mapping

| arama (was, iced built-in) | becomes (arama-theme) |
|---|---|
| `button::primary` | `arama_theme::primary` |
| `button::text` | `arama_theme::ghost` (transparent/text equivalent) |
| `button::secondary` | `arama_theme::secondary` |
| `button::danger` | `arama_theme::danger` |

### Call sites migrated

- `app/src/core/view.rs` — nav rail (active = primary, inactive = ghost)
- `crates/ui/widgets/.../general_settings/view.rs` — locale selector
- `crates/ui/main/.../cache_page/view.rs` — stop button (danger)
- `crates/ui/main/.../setup/view.rs` — skip button (secondary)

## Preset choice

arama uses iced's default `Theme::Light`, so `arama-theme` initialises
with `Tokens::light()`. The global is structured so a future RFC can add
a light / dark / high-contrast app setting (snora ships all four presets)
without touching call sites — only the initialisation changes.

## Visual change

Button colors will shift from iced's stock palette to snora's
contrast-verified palette. This is the intended accessibility
improvement, not a regression. The `Theme::Light` default and
`Tokens::light()` are aligned so the change is a palette refinement, not
a light/dark flip.

## Non-goals

- Cards, chips, notices, progress primitives — not adopted here. arama
  uses toasts (not inline notices) and has no current chip/card surface.
  These remain available for future RFCs.
- App-level light/dark theme setting — future RFC.

## Open questions

None.
