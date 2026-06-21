//! # arama-theme
//!
//! Token-driven button styling for arama, backed by the Snora Design
//! system (RFC 010).
//!
//! ## Pattern
//!
//! snora's design style functions take a `&Tokens` explicitly. Rather than
//! thread a `&Tokens` through every `view()` signature in the UI crates,
//! this crate holds the active token set globally — the same approach
//! arama uses for i18n (a global locale + free `t()` function) — and
//! exposes drop-in style functions whose shape matches iced's built-in
//! button styles (`fn(&Theme, button::Status) -> button::Style`).
//!
//! Call sites therefore change only the function path:
//!
//! ```rust,ignore
//! // was:  .style(button::primary)      // iced built-in
//! // now:  .style(arama_theme::primary)  // Snora-Design token-driven
//! ```
//!
//! ## Preset
//!
//! arama uses iced's default `Theme::Light`, so the global initialises
//! with [`snora::design::Tokens::light`]. A future setting for light /
//! dark / high-contrast can change only the initialisation here without
//! touching any call site.

use std::sync::OnceLock;

use iced::{Theme, widget::button};
use snora::design::Tokens;

static TOKENS: OnceLock<Tokens> = OnceLock::new();

/// The active design tokens. Defaults to the light preset.
fn tokens() -> &'static Tokens {
    TOKENS.get_or_init(Tokens::light)
}

/// Primary (accent) button style — used for the active navigation item
/// and confirmation actions.
pub fn primary(_theme: &Theme, status: button::Status) -> button::Style {
    snora::design::style::button::primary(tokens(), status)
}

/// Ghost (transparent) button style — the token-driven equivalent of
/// iced's `button::text`, used for inactive navigation items.
pub fn ghost(_theme: &Theme, status: button::Status) -> button::Style {
    snora::design::style::button::ghost(tokens(), status)
}

/// Secondary button style — used for non-primary actions such as "Skip".
pub fn secondary(_theme: &Theme, status: button::Status) -> button::Style {
    snora::design::style::button::secondary(tokens(), status)
}

/// Danger button style — used for destructive actions such as "Stop".
pub fn danger(_theme: &Theme, status: button::Status) -> button::Style {
    snora::design::style::button::danger(tokens(), status)
}
