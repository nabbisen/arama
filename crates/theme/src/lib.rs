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
//! * **C — Base iced theme.** [`iced_theme`] returns an iced [`Theme`]
//!   derived from the active Snora token palette for the application's
//!   `.theme()` callback, so the window background and all stock iced widgets
//!   track the preset as closely as iced's six-role palette allows.
//!
//! ## Global state
//!
//! The active preset is stored in a global `AtomicU8` — the same lock-free
//! pattern arama uses for the i18n locale — so [`set_theme`] and the lookup
//! functions are safe to call from any thread without lifetime friction in
//! `view()`.

use std::sync::atomic::{AtomicU8, Ordering};

use arama_env::ThemePreset;
use iced::{Theme, theme, widget::button};
use snora::design::{Tokens, style::color::to_iced_color};

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
pub fn tokens() -> Tokens {
    tokens_for_preset(current_theme())
}

fn tokens_for_preset(preset: ThemePreset) -> Tokens {
    match preset {
        ThemePreset::Light => Tokens::light(),
        ThemePreset::Dark => Tokens::dark(),
        ThemePreset::HighContrastLight => Tokens::high_contrast_light(),
        ThemePreset::HighContrastDark => Tokens::high_contrast_dark(),
    }
}

/// The base iced [`Theme`] for the active preset (layer C).
///
/// Snora exposes 18 semantic palette roles, while iced 0.14's custom theme
/// palette accepts six core roles. This bridge maps the six roles that survive
/// cleanly so stock iced widgets receive the active preset's high-contrast
/// colors instead of falling back to the built-in light/dark palettes.
pub fn iced_theme() -> Theme {
    let preset = current_theme();
    let tokens = tokens_for_preset(preset);

    Theme::custom(theme_name(preset), iced_palette_from_tokens(&tokens))
}

fn iced_palette_from_tokens(tokens: &Tokens) -> theme::Palette {
    let palette = &tokens.palette;

    theme::Palette {
        background: to_iced_color(palette.surface),
        text: to_iced_color(palette.text_primary),
        primary: to_iced_color(palette.accent),
        success: to_iced_color(palette.success),
        warning: to_iced_color(palette.warning),
        danger: to_iced_color(palette.danger),
    }
}

fn theme_name(preset: ThemePreset) -> &'static str {
    match preset {
        ThemePreset::Light => "arama-light",
        ThemePreset::Dark => "arama-dark",
        ThemePreset::HighContrastLight => "arama-high-contrast-light",
        ThemePreset::HighContrastDark => "arama-high-contrast-dark",
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

// ---------------------------------------------------------------------------
// Text role functions (RFC 043) — the six-role type scale, resolved against
// the active preset with no argument, same shape as the button functions
// above. Kept here rather than importing `snora` into every UI crate that
// wants a role: this is the only crate the workspace wires to `snora`
// directly (besides `app`), so this is the seam.
// ---------------------------------------------------------------------------

use iced::{Pixels, widget::text::LineHeight};

/// `body` text size — ordinary explanatory text.
pub fn body_size() -> Pixels {
    snora::design::style::text::body_size(&tokens())
}

/// `body` line height.
pub fn body_line_height() -> LineHeight {
    snora::design::style::text::body_line_height(&tokens())
}

/// `body_small` text size — secondary metadata, compact help.
pub fn body_small_size() -> Pixels {
    snora::design::style::text::body_small_size(&tokens())
}

/// `body_small` line height.
pub fn body_small_line_height() -> LineHeight {
    snora::design::style::text::body_small_line_height(&tokens())
}

/// `label` text size — button, field and chip labels.
pub fn label_size() -> Pixels {
    snora::design::style::text::label_size(&tokens())
}

/// `label` line height.
pub fn label_line_height() -> LineHeight {
    snora::design::style::text::label_line_height(&tokens())
}

/// `title` text size — card / dialog / notice title.
pub fn title_size() -> Pixels {
    snora::design::style::text::title_size(&tokens())
}

/// `title` line height.
pub fn title_line_height() -> LineHeight {
    snora::design::style::text::title_line_height(&tokens())
}

/// `heading` text size — page or section heading.
pub fn heading_size() -> Pixels {
    snora::design::style::text::heading_size(&tokens())
}

/// `heading` line height.
pub fn heading_line_height() -> LineHeight {
    snora::design::style::text::heading_line_height(&tokens())
}

/// `display` text size — rare major page title.
pub fn display_size() -> Pixels {
    snora::design::style::text::display_size(&tokens())
}

/// `display` line height.
pub fn display_line_height() -> LineHeight {
    snora::design::style::text::display_line_height(&tokens())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static THEME_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn iced_theme_uses_snora_palette_for_each_preset() {
        let _guard = THEME_TEST_LOCK.lock().unwrap();
        for preset in ThemePreset::all() {
            set_theme(*preset);

            let expected = iced_palette_from_tokens(&tokens_for_preset(*preset));
            let actual = iced_theme().palette();

            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn high_contrast_presets_do_not_fall_back_to_builtin_palettes() {
        let _guard = THEME_TEST_LOCK.lock().unwrap();
        set_theme(ThemePreset::HighContrastLight);
        assert_ne!(iced_theme().palette(), theme::Palette::LIGHT);

        set_theme(ThemePreset::HighContrastDark);
        assert_ne!(iced_theme().palette(), theme::Palette::DARK);
    }
}
