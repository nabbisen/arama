//! # arama-theme
//!
//! Token-driven styling for arama, backed by the Snora Design system
//! (RFC 010) with a runtime-selectable theme preset (RFC 011).
//!
//! ## Three styling layers
//!
//! A theme switch must move three layers together:
//!
//! * **A — Snora button tokens.** The four button style functions below
//!   ([`primary`], [`ghost`], [`secondary`], [`danger`]) resolve the active
//!   [`snora::design::Tokens`] from the current preset.
//! * **B — Snora container tokens.** Reserved for future card surfaces;
//!   driven by the same [`tokens`].
//! * **C — Base iced theme.** [`iced_theme`] returns the matching iced
//!   [`Theme`] for the application's `.theme()` callback, so the window
//!   background and all stock iced widgets track the preset.
//!
//! ## Global state
//!
//! The active preset is stored in a global `AtomicU8` — the same lock-free
//! pattern arama uses for the i18n locale — so [`set_theme`] and the lookup
//! functions are safe to call from any thread without lifetime friction in
//! `view()`.

use std::sync::atomic::{AtomicU8, Ordering};

use arama_env::ThemePreset;
use iced::{Theme, widget::button};
use snora::design::Tokens;

// ---------------------------------------------------------------------------
// Global preset state
// ---------------------------------------------------------------------------

static THEME_ID: AtomicU8 = AtomicU8::new(0 /* ThemePreset::Light */);

/// Set the active theme preset. Safe to call from any thread.
pub fn set_theme(preset: ThemePreset) {
    THEME_ID.store(preset as u8, Ordering::Relaxed);
}

/// Return the currently active theme preset.
pub fn current_theme() -> ThemePreset {
    match THEME_ID.load(Ordering::Relaxed) {
        1 => ThemePreset::Dark,
        2 => ThemePreset::HighContrastLight,
        3 => ThemePreset::HighContrastDark,
        _ => ThemePreset::Light,
    }
}

// ---------------------------------------------------------------------------
// Preset → tokens (layers A / B) and → iced Theme (layer C)
// ---------------------------------------------------------------------------

/// The Snora Design tokens for the active preset.
///
/// Returns an owned `Tokens`; snora's style helpers clone tokens into their
/// style closures anyway, so this avoids any `'static` lifetime constraint.
/// `Tokens` is small and `Clone`; the per-button clone cost in `view()` is
/// negligible.
fn tokens() -> Tokens {
    match current_theme() {
        ThemePreset::Light => Tokens::light(),
        ThemePreset::Dark => Tokens::dark(),
        ThemePreset::HighContrastLight => Tokens::high_contrast_light(),
        ThemePreset::HighContrastDark => Tokens::high_contrast_dark(),
    }
}

/// The base iced [`Theme`] for the active preset (layer C).
///
/// iced 0.14 has no built-in high-contrast theme, so the high-contrast
/// presets map to the matching `Light` / `Dark` base. arama's own controls
/// (buttons, and future cards) still get the full high-contrast tokens via
/// layers A / B; only stock iced widgets fall back to the base theme.
pub fn iced_theme() -> Theme {
    match current_theme() {
        ThemePreset::Light | ThemePreset::HighContrastLight => Theme::Light,
        ThemePreset::Dark | ThemePreset::HighContrastDark => Theme::Dark,
    }
}

// ---------------------------------------------------------------------------
// Button style functions (layer A) — drop-in shape for iced's `.style(...)`
// ---------------------------------------------------------------------------

/// Primary (accent) button style — active navigation item, confirmations.
pub fn primary(_theme: &Theme, status: button::Status) -> button::Style {
    snora::design::style::button::primary(&tokens(), status)
}

/// Ghost (transparent) button style — token-driven equivalent of iced's
/// `button::text`, used for inactive navigation items.
pub fn ghost(_theme: &Theme, status: button::Status) -> button::Style {
    snora::design::style::button::ghost(&tokens(), status)
}

/// Secondary button style — non-primary actions such as "Skip".
pub fn secondary(_theme: &Theme, status: button::Status) -> button::Style {
    snora::design::style::button::secondary(&tokens(), status)
}

/// Danger button style — destructive actions such as "Stop".
pub fn danger(_theme: &Theme, status: button::Status) -> button::Style {
    snora::design::style::button::danger(&tokens(), status)
}
