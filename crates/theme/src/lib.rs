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

use arama_env::{TextScale, ThemePreset};
use iced::{Border, Shadow, Theme, theme, widget::button};
use snora::design::{Tokens, Typography, style::color::to_iced_color};

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

static TEXT_SCALE_ID: AtomicU8 = AtomicU8::new(0 /* TextScale::Default */);

/// Set the active text-size scale. Safe to call from any thread, the
/// same lock-free pattern as [`set_theme`] - but unlike theme and
/// locale, **production code must call this at most once, at startup,
/// before `App::settings()` is evaluated** (RFC 052 §2.2). The route
/// 118 of 120 `text(` sites depend on, `iced::Settings.
/// default_text_size`, is read once when the application starts and
/// never again; calling this again afterward would re-scale only the
/// `snora` chrome that reads [`tokens`] on every render, reproducing the
/// half-scaled interface RFC 052 §1 measured and rejected. The settings
/// dialog's own change handler persists a new choice to `settings.json`
/// and shows a restart notice - it must never call this function.
///
/// Mutable (not a one-shot `OnceLock`) because the headless layout
/// matrix (RFC 053 §2.1) needs to exercise all three steps within one
/// test binary; production's own single-call discipline is enforced by
/// review, the same way nothing but the theme-change handler calls
/// [`set_theme`].
pub fn set_text_scale(scale: TextScale) {
    TEXT_SCALE_ID.store(scale as u8, Ordering::Relaxed);
}

/// Return the currently active text-size scale.
pub fn current_text_scale() -> TextScale {
    match TEXT_SCALE_ID.load(Ordering::Relaxed) {
        1 => TextScale::Large,
        2 => TextScale::Larger,
        _ => TextScale::Default,
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
///
/// RFC 052 §2.1: scales [`Typography`] by [`current_text_scale`] before
/// returning - the one place that matters, since every typography
/// accessor below (`body_size`, `label_size`, ...) and
/// `App::settings()`'s own `default_text_size` all read `tokens()`
/// rather than `tokens_for_preset()` directly. `iced_theme()` below
/// calls `tokens_for_preset()` instead, deliberately unscaled: it only
/// ever reads `tokens.palette` (colour), never `typography`, so scaling
/// it there would be dead code, not a second place this needs doing.
pub fn tokens() -> Tokens {
    let mut tokens = tokens_for_preset(current_theme());
    tokens.typography = scale_typography(tokens.typography, current_text_scale());
    tokens
}

fn tokens_for_preset(preset: ThemePreset) -> Tokens {
    match preset {
        ThemePreset::Light => Tokens::light(),
        ThemePreset::Dark => Tokens::dark(),
        ThemePreset::HighContrastLight => Tokens::high_contrast_light(),
        ThemePreset::HighContrastDark => Tokens::high_contrast_dark(),
    }
}

/// Scales every [`snora::design::TextRole::size`] in `typography` by
/// `scale.factor()`. `line_height` is left untouched: it is already a
/// multiplier relative to `size` (`TextRole`'s own doc comment), so it
/// stays proportionally correct without being scaled a second time.
fn scale_typography(typography: Typography, scale: TextScale) -> Typography {
    let factor = scale.factor();
    let scale_role = |role: snora::design::TextRole| snora::design::TextRole {
        size: role.size * factor,
        line_height: role.line_height,
    };
    Typography {
        body: scale_role(typography.body),
        body_small: scale_role(typography.body_small),
        label: scale_role(typography.label),
        title: scale_role(typography.title),
        heading: scale_role(typography.heading),
        display: scale_role(typography.display),
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

/// `primary`, except its `Disabled` arm keeps the label fully readable
/// and marks "off" by shape - no fill, an outline - instead of a
/// uniform alpha fade (RFC 052 §4, T077).
///
/// `primary`'s own `Disabled` arm (`snora-style` 0.57.0's
/// `disabled_alpha`, ×0.45 on both the fill and the text, composited
/// over the page) measures 1.49-1.83:1 across arama's four presets
/// (measured directly: `composite_over` the faded fill over
/// `palette.surface` - the role `iced_theme()` actually paints the
/// app's background from, not `palette.background` - then the faded
/// text over that), far below WCAG 1.4.3's 4.5:1 floor. A control
/// whose disabled state *is* the message (RFC 052 §4) needs to stay
/// readable while disabled, not just legible when enabled.
///
/// `text_primary` against `palette.surface` measures 15.55-21.00:1 on
/// the same four presets - comfortably clear regardless of which
/// `Disabled` arm is actually painted over, since the fill is gone
/// entirely. The outline (`palette.border`, the dedicated "borders and
/// separators" role) is what marks the button as inactive (WCAG 1.4.1:
/// not by colour alone) - a border carries no contrast requirement of
/// its own, only the text and fill do.
pub fn primary_readable_when_disabled(theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Disabled => {
            let t = tokens();
            button::Style {
                background: None,
                text_color: to_iced_color(t.palette.text_primary),
                border: Border::default()
                    .width(1.0)
                    .color(to_iced_color(t.palette.border))
                    .rounded(t.radius.md),
                shadow: Shadow::default(),
                ..button::Style::default()
            }
        }
        other => primary(theme, other),
    }
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

    /// RFC 052 §2.1: every role scales by the same factor, and
    /// `line_height` never changes - it is already relative to `size`.
    #[test]
    fn text_scale_scales_every_typography_role_and_leaves_line_height_alone() {
        let _guard = THEME_TEST_LOCK.lock().unwrap();
        let unscaled = tokens_for_preset(ThemePreset::Light).typography;

        for scale in TextScale::all() {
            set_text_scale(*scale);
            let scaled = tokens().typography;
            let factor = scale.factor();

            for (unscaled_role, scaled_role) in [
                (unscaled.body, scaled.body),
                (unscaled.body_small, scaled.body_small),
                (unscaled.label, scaled.label),
                (unscaled.title, scaled.title),
                (unscaled.heading, scaled.heading),
                (unscaled.display, scaled.display),
            ] {
                assert_eq!(scaled_role.size, unscaled_role.size * factor);
                assert_eq!(scaled_role.line_height, unscaled_role.line_height);
            }
        }

        set_text_scale(TextScale::default());
    }

    /// The exact numbers `App::settings()` derives `default_text_size`
    /// from - stated plainly, matching `arama-env`'s own
    /// `text_scale_factors_and_default` test.
    #[test]
    fn text_scale_resulting_body_px_per_step() {
        let _guard = THEME_TEST_LOCK.lock().unwrap();
        set_theme(ThemePreset::Light);

        set_text_scale(TextScale::Default);
        assert_eq!(tokens().typography.body.size, 16.0);
        set_text_scale(TextScale::Large);
        assert_eq!(tokens().typography.body.size, 20.0);
        set_text_scale(TextScale::Larger);
        assert_eq!(tokens().typography.body.size, 24.0);

        set_text_scale(TextScale::default());
    }

    /// T064 (handoff condition 3, WCAG 1.4.11 non-text contrast): the
    /// side-bar rail's active-item fill (`snora::style::
    /// sidebar_active_color`, `extended_palette().primary.strong.color`,
    /// derived by iced from the theme rather than a token value read
    /// directly) against the rail's own unselected background
    /// (`extended_palette().background.base.color`, the same baseline
    /// `snora-widgets`' own `sidebar_button_style_text_meets_aa` test
    /// uses for its non-active case), for each of arama's four presets.
    /// 1.4.11 floors non-text UI-component contrast at 3:1, looser than
    /// 1.4.3's 4.5:1 for body text.
    #[test]
    fn side_bar_active_fill_meets_wcag_1_4_11_non_text_contrast_every_preset() {
        let _guard = THEME_TEST_LOCK.lock().unwrap();

        fn as_design_color(c: iced::Color) -> snora::design::Color {
            snora::design::Color::rgba(c.r, c.g, c.b, c.a)
        }

        for preset in ThemePreset::all() {
            set_theme(*preset);
            let theme = iced_theme();
            let active_fill = as_design_color(snora::style::sidebar_active_color(&theme));
            let page_background = as_design_color(theme.extended_palette().background.base.color);
            let ratio = snora::design::contrast::contrast_ratio(active_fill, page_background);
            eprintln!(
                "=== T064 side-bar active-fill contrast: {preset:?} = {ratio:.2}:1 \
                 (WCAG 1.4.11 floor 3.0) ==="
            );

            assert!(
                ratio >= 3.0,
                "{preset:?}: the side bar's active-item fill contrasts {ratio:.2}:1 \
                 against the page background, below WCAG 1.4.11's 3:1 floor for \
                 non-text UI components"
            );
        }

        set_theme(ThemePreset::default());
    }

    /// T077 (RFC 052 §4, WCAG 1.4.3): `primary`'s own `Disabled` arm
    /// (`disabled_alpha`, ×0.45 on both fill and text, in
    /// `snora-style`) measured 1.49-1.83:1 across the four presets
    /// during development - confirmed before building the replacement
    /// below, not re-asserted here since `primary_readable_when_
    /// disabled` no longer uses that arm at all. This test covers what
    /// the handoff asks for: the disabled button's own
    /// label against its background, and the "unavailable" line's
    /// colour (full-strength `text_primary`, the same role a plain
    /// `text(...)` widget already resolves to with no `.style(...)`
    /// override - see `iced_palette_from_tokens`) against the page.
    /// Both read `tokens().palette.surface` directly, matching what
    /// `primary_readable_when_disabled` and the page's own real
    /// background (`iced_theme()`'s own `background: to_iced_color(
    /// palette.surface)` mapping) actually use - not
    /// `palette.background`, a different, unused-for-rendering role.
    #[test]
    fn t077_disabled_button_and_unavailable_line_meet_wcag_1_4_3_every_preset() {
        use snora::design::contrast::contrast_ratio;
        let _guard = THEME_TEST_LOCK.lock().unwrap();

        for preset in ThemePreset::all() {
            set_theme(*preset);
            let t = tokens();
            let page = t.palette.surface;

            let disabled_style =
                primary_readable_when_disabled(&iced_theme(), button::Status::Disabled);
            let disabled_text = snora::design::Color::rgba(
                disabled_style.text_color.r,
                disabled_style.text_color.g,
                disabled_style.text_color.b,
                disabled_style.text_color.a,
            );
            let disabled_ratio = contrast_ratio(disabled_text, page);

            let unavailable_ratio = contrast_ratio(t.palette.text_primary, page);

            eprintln!(
                "=== T077 {preset:?}: disabled button label = {disabled_ratio:.2}:1, \
                 unavailable line = {unavailable_ratio:.2}:1 (WCAG 1.4.3 floor 4.5) ==="
            );

            assert!(
                disabled_ratio >= 4.5,
                "{preset:?}: the disabled Similarity-pairs button's label contrasts \
                 {disabled_ratio:.2}:1 against its background, below WCAG 1.4.3's 4.5:1"
            );
            assert!(
                unavailable_ratio >= 4.5,
                "{preset:?}: the unavailable line contrasts {unavailable_ratio:.2}:1 \
                 against the page, below WCAG 1.4.3's 4.5:1"
            );
            assert!(
                disabled_style.background.is_none(),
                "{preset:?}: the disabled style must have no fill - WCAG 1.4.1, \
                 \"off\" is marked by shape (the outline), not colour alone"
            );
        }

        set_theme(ThemePreset::default());
    }
}
