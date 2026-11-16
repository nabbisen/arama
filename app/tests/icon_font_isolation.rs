//! T074 Phase 0 item 3a, moved here per review 176's condition 1: this
//! must run in its own process.
//!
//! `iced_graphics::text::font_system()` is a process-wide `OnceLock`
//! singleton with no unload (`iced_graphics-0.14.0/src/text.rs:115-134`).
//! `app/src/core/tests/layout_probe.rs` lives inside the `arama` crate's
//! *lib* test binary, alongside - once the layout matrix lands - tests
//! that render through `App::settings()`, which always loads the Lucide
//! font. If this "with vs. without Lucide" control ran there too, any
//! matrix test `cargo test`'s default parallel scheduler happened to run
//! first would load Lucide into that shared singleton and silently
//! invalidate this control's "without" measurement - a false pass or
//! fail depending on test order, not on anything this test actually
//! checks.
//!
//! A file directly under `tests/` compiles to its own separate binary,
//! hence its own separate process with its own, never-touched
//! `font_system()` - the one isolation that actually fixes this, as
//! opposed to reordering or serializing within one binary, which Rust's
//! default test runner does not promise either way.
use lucide_icons::LUCIDE_FONT_BYTES;

#[test]
fn icon_measured_width_differs_with_and_without_the_lucide_font() {
    fn measured_width(settings: iced::Settings) -> f32 {
        let icon = lucide_icons::iced::icon_database();
        let element: iced::Element<'_, (), iced::Theme, iced::Renderer> = icon.into();
        let mut simulator = iced_test::Simulator::with_settings(settings, element);
        let found = simulator
            .find("\u{e0ad}")
            .expect("the icon's own text content must be findable either way");
        found.bounds().width
    }

    // Order still matters within this one process, for the same reason
    // the module doc above explains: "without" must run first, since
    // `FontSystem::load_font` has no unload.
    let without_lucide = measured_width(iced::Settings::default());
    let with_lucide = measured_width(iced::Settings {
        fonts: vec![LUCIDE_FONT_BYTES.into()],
        ..iced::Settings::default()
    });

    assert_ne!(
        with_lucide, without_lucide,
        "the icon's measured width must differ depending on whether the Lucide font \
         App::settings() loads is actually present (with={with_lucide}, \
         without={without_lucide}) - otherwise review 172's stray-glyph/empty-box \
         finding would be invisible to a headless measurement"
    );
}
