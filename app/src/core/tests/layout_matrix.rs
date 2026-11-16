//! T074 delivery 2 (RFC 053 §2): the headless layout matrix. Checks only
//! the properties `layout_probe.rs` (Phase 0, delivery 1) showed this
//! harness can actually detect - see review 176's conditions, each
//! addressed at its own use site below.
//!
//! **Property 1** (RFC 053 §2.3.1, scoped per Phase 0 item 1): every
//! `Target::Text`'s `visible_bounds` equals its `bounds` - proven only
//! for a `scrollable` ancestor's viewport (item 1a). A plain
//! `Container::clip(true)` changes neither `visible_bounds` (item 1b)
//! nor, found while building this delivery, the actually painted pixels
//! either (item 1c, confirmed on both `wgpu` and `tiny-skia`) - so this
//! property cannot see, and does not claim to see, overflow hidden by a
//! plain `clip(true)` container. **That is arama's own real
//! `crates/ui/layout/src/footer/view.rs` path label** - the one
//! production `Wrapping::None` site in the codebase, and the reason T055
//! added `clip(true)` there. No property below checks it; it remains an
//! open gap, not a silent claim.
//!
//! **Property 2** (non-clipping overflow, item 2's pixel method) is not
//! applied here at all. Item 2 proved the technique only for an element
//! whose exact colour and wrapping mode were fixed by the test itself,
//! requirements `Target` gives no way to learn for arbitrary real
//! content. The one real site that would need it (the footer above) is
//! `clip(true)`-protected, which item 1c just showed this harness cannot
//! tell apart from unprotected either way. Recorded as a gap (RFC 053
//! §2.3: "if nothing does, say so"), not implemented as a guess.

use super::layout_probe::find_all_text_with;
use super::*;

// ---------------------------------------------------------------------
// Backend choice (review 176, condition 2)
// ---------------------------------------------------------------------
//
// Phase 0 item 6 measured the default backend (`wgpu`, first in
// `Simulator`'s own fallback candidate list) at ~190ms/combination on
// Linux and ~258ms on Windows - the slowest runner. 24 combinations
// (locale x theme x window size; text size stays at Default until T063
// wires that dimension) across the dozen screen states below would be
// roughly 24 x 12 x 258ms =~ 74s on Windows alone, over RFC 053 §2.5's
// 60s budget. `ICED_TEST_BACKEND=tiny-skia` measured ~100x faster for
// every one of Phase 0's properties, with identical results on both
// backends (confirmed in `layout_probe.rs`'s own item 1c, item 2
// candidate 3, and every other property check). Chosen here,
// unconditionally, for every matrix combination.
fn use_fast_backend() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        // SAFETY: called once, from `run_matrix` before any `Simulator`
        // is constructed in this process, under the same
        // `ARAMA_DATA_HOME_TEST_LOCK` every App-constructing test in this
        // binary already takes (see that lock's own doc comment) - no
        // other test can be reading or writing this specific variable
        // concurrently.
        unsafe {
            std::env::set_var("ICED_TEST_BACKEND", "tiny-skia");
        }
    });
}

// ---------------------------------------------------------------------
// Combinations (RFC 053 §2.1)
// ---------------------------------------------------------------------

const WINDOW_SIZES: [(f32, f32); 3] = [
    (760.0, 520.0),   // the enforced minimum (app/src/core.rs)
    (1280.0, 720.0),  // the GUI rules' own default
    (1920.0, 1080.0), // one large size
];

#[derive(Clone, Copy, Debug)]
struct Combination {
    locale: arama_i18n::Locale,
    theme: arama_env::ThemePreset,
    window: (f32, f32),
}

impl std::fmt::Display for Combination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} / {:?} / {}x{}",
            self.locale.code(),
            self.theme,
            self.window.0,
            self.window.1
        )
    }
}

/// 2 locales x 4 theme presets x 3 window sizes = 24. Text size stays at
/// `Default` (RFC 053 §2.1: "until T063 lands: Default only, with the
/// dimension wired").
fn combinations() -> Vec<Combination> {
    let mut out = Vec::with_capacity(2 * 4 * 3);
    for &locale in arama_i18n::Locale::all() {
        for &theme in arama_env::ThemePreset::all() {
            for &window in &WINDOW_SIZES {
                out.push(Combination {
                    locale,
                    theme,
                    window,
                });
            }
        }
    }
    out
}

// ---------------------------------------------------------------------
// The kana-face gate (RFC 053 §6.4, review 176 condition 4)
// ---------------------------------------------------------------------

/// Fails, never skips, if the font database has no face covering a kana
/// glyph - the direct cosmic-text query `layout_probe.rs`'s item 4
/// establishes, not a name request (item 3b showed a name request
/// cannot distinguish real coverage from "fell back anyway"). Called
/// once per matrix test, before any `ja`-locale combination runs.
fn assert_a_japanese_face_is_available() {
    use iced::advanced::graphics::text::cosmic_text;

    let mut font_system = iced::advanced::graphics::text::font_system()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let raw = font_system.raw();

    let mut buffer = cosmic_text::Buffer::new(raw, cosmic_text::Metrics::new(16.0, 20.0));
    let attrs = cosmic_text::Attrs::new();
    let kana = "あ";
    buffer.set_text(raw, kana, &attrs, cosmic_text::Shaping::Advanced, None);

    let mut covered = false;
    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            if raw.db_mut().face(glyph.font_id).is_some() {
                covered = true;
            }
        }
    }

    assert!(
        covered,
        "no face in the font database covers a kana glyph (あ) - the layout matrix \
         cannot measure Japanese without one. Install a Japanese font: \
         fonts-noto-cjk on Linux (see .github/workflows/msrv.yaml), or the system \
         fonts macOS/Windows already carry."
    );
}

// ---------------------------------------------------------------------
// Known layout defects (RFC 053 §6.5)
// ---------------------------------------------------------------------

/// One entry per combination that genuinely fails property 1 today.
/// Each is asserted (in `run_matrix`) to still fail - removing a fixed
/// entry is required, not optional, so this list cannot go stale.
struct KnownDefect {
    state: &'static str,
    locale: arama_i18n::Locale,
    window: (f32, f32),
    /// A substring of the failing text's own content, to name exactly
    /// which label this entry covers - not "the whole combination."
    text_contains: &'static str,
    /// The task that owns the fix.
    task: &'static str,
}

/// Empty today: every screen state this delivery covers passed property
/// 1 at every combination, including the 760 x 520 minimum, on this
/// tree. If a future combination regresses, this is where its entry
/// goes - with the task that owns fixing it, not a workaround here.
const KNOWN_LAYOUT_DEFECTS: &[KnownDefect] = &[];

fn is_known_defect(state: &str, combo: Combination, content: &str) -> bool {
    KNOWN_LAYOUT_DEFECTS.iter().any(|defect| {
        defect.state == state
            && defect.locale.code() == combo.locale.code()
            && defect.window == combo.window
            && content.contains(defect.text_contains)
    })
}

// ---------------------------------------------------------------------
// Property 1 and the shared per-combination driver
// ---------------------------------------------------------------------

/// Sub-pixel rendering noise, not a layout defect: real renders of the
/// identical rectangle differ by thousandths of a logical pixel (e.g.
/// `20.8` vs `20.799995`), confirmed by an early run of this matrix
/// failing every single state on exactly that gap for the header's own
/// chrome. A quarter of a physical pixel at this harness's fixed 2x
/// scale factor (`iced_test::Simulator::snapshot`'s own constant) is a
/// generous, still-meaningful bound - anything a user could perceive as
/// clipped is far larger than this.
const VISIBILITY_EPSILON: f32 = 0.125;

fn rectangle_eq_within_epsilon(a: iced::Rectangle, b: iced::Rectangle) -> bool {
    (a.x - b.x).abs() <= VISIBILITY_EPSILON
        && (a.y - b.y).abs() <= VISIBILITY_EPSILON
        && (a.width - b.width).abs() <= VISIBILITY_EPSILON
        && (a.height - b.height).abs() <= VISIBILITY_EPSILON
}

/// RFC 053 §2.6: a failure names its combination, the text's own
/// content, and both rectangles.
fn assert_property_1(state: &str, combo: Combination, found: Vec<iced_test::selector::Target>) {
    for target in found {
        if let iced_test::selector::Target::Text {
            content,
            bounds,
            visible_bounds,
            ..
        } = target
        {
            // An empty label (a conditional placeholder with nothing to
            // show) has no content to be clipped - confirmed by its
            // zero-width `bounds` and `visible_bounds: None` (a
            // zero-area rectangle's intersection with any viewport is
            // empty by construction, not a sign anything is hidden).
            if content.is_empty() {
                continue;
            }
            let fully_visible =
                visible_bounds.is_some_and(|v| rectangle_eq_within_epsilon(v, bounds));
            if !fully_visible {
                if is_known_defect(state, combo, &content) {
                    continue;
                }
                panic!(
                    "{state}: \"{content}\" is not fully visible at {combo} - \
                     bounds={bounds:?} visible_bounds={visible_bounds:?}"
                );
            }
        }
    }

    for defect in KNOWN_LAYOUT_DEFECTS {
        if defect.state != state
            || defect.locale.code() != combo.locale.code()
            || defect.window != combo.window
        {
            continue;
        }
        // The entry's own text must still be present and still failing -
        // otherwise a fix landed and nobody removed the entry (§6.5).
        panic!(
            "{state}: KNOWN_LAYOUT_DEFECTS entry for \"{}\" ({}) is listed as still \
             failing at {combo}, but no text containing that substring was found \
             failing in this run - remove the entry, it looks fixed",
            defect.text_contains, defect.task
        );
    }
}

/// Isolation (RFC 053 §6.3/§7, mirroring `tests.rs`'s own convention):
/// every matrix test takes `ARAMA_DATA_HOME_TEST_LOCK` and points
/// `ARAMA_DATA_HOME` at a scratch directory - the same serialization
/// that already keeps global locale mutation safe here (see below), not
/// a second, narrower lock. Every test in this binary that constructs an
/// `App` already takes this same lock (confirmed: no counter-example in
/// `tests.rs` or `layout_probe.rs`), which is what makes mutating the
/// process-global locale and theme here safe rather than a third
/// instance of the exact race `tests.rs`'s own `walk_error_summary_...`
/// comment and `crates/ui/layout/src/footer/view.rs`'s test comment
/// both already document and work around.
fn run_matrix(state_name: &'static str, mut build: impl FnMut(&mut App, Combination)) {
    use_fast_backend();

    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous_env = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home(&format!("t074-matrix-{state_name}"));
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    let mut japanese_face_checked = false;

    for combo in combinations() {
        if combo.locale == arama_i18n::Locale::Ja && !japanese_face_checked {
            assert_a_japanese_face_is_available();
            japanese_face_checked = true;
        }
        arama_i18n::set_locale(combo.locale);
        arama_theme::set_theme(combo.theme);

        let mut app = App::new().0;
        build(&mut app, combo);
        let element = app.view();
        let found = find_all_text_with(App::settings(), combo.window, element);

        assert_property_1(state_name, combo, found);
    }

    arama_i18n::set_locale(arama_i18n::Locale::default());
    arama_theme::set_theme(arama_env::ThemePreset::default());
    unsafe {
        match &previous_env {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

// ---------------------------------------------------------------------
// Screen states (RFC 053 §2.1, Phase 0 item 5's survey)
// ---------------------------------------------------------------------

#[test]
fn setup_wizard() {
    // The default state: `App::new()` never finishes setup on its own.
    run_matrix("setup_wizard", |_app, _combo| {});
}

#[test]
fn gallery_empty() {
    run_matrix("gallery_empty", |app, _combo| {
        app.setup.finished = true;
    });
}

#[test]
fn settings_page() {
    run_matrix("settings_page", |app, _combo| {
        app.setup.finished = true;
        app.nav_page = NavPage::Settings;
    });
}

#[test]
fn cache_page() {
    run_matrix("cache_page", |app, _combo| {
        app.setup.finished = true;
        app.nav_page = NavPage::Cache;
    });
}

#[test]
fn media_focus_dialog() {
    use arama_env::cache_lookup_strategy::CacheLookupStrategy;
    use arama_ui_widgets::dialog::media_focus_dialog::MediaFocusDialog;

    run_matrix("media_focus_dialog", |app, _combo| {
        app.setup.finished = true;
        app.dialog = Some(Dialog::MediaFocusDialog(MediaFocusDialog::new(
            missing_test_path(),
            CacheLookupStrategy::default(),
            0.0,
            None,
        )));
    });
}

#[test]
fn confirm_dialog() {
    use arama_ui_widgets::dialog::confirm_dialog::ConfirmDialog;

    run_matrix("confirm_dialog", |app, _combo| {
        app.setup.finished = true;
        app.dialog = Some(Dialog::Confirm(ConfirmDialog::new(
            arama_i18n::t("toast.cache_delete_failed.title"),
            arama_i18n::t("toast.cache_delete_complete.body"),
            arama_i18n::t("toast.cache_delete_complete.title"),
        )));
    });
}

#[test]
fn toast_over_gallery() {
    run_matrix("toast_over_gallery", |app, _combo| {
        app.setup.finished = true;
        app.push_warning_toast("Indexed with warnings", "3 files skipped");
    });
}

#[test]
fn similar_pairs_scanning() {
    use arama_ui_widgets::dialog::similar_pairs_dialog::SimilarPairsDialog;
    use swdir::DirNode;

    run_matrix("similar_pairs_scanning", |app, _combo| {
        app.setup.finished = true;
        let dir_node = DirNode {
            path: missing_test_path(),
            sub_dirs: vec![],
            files: vec![],
        };
        app.dialog = Some(Dialog::SimilarPairsDialog(SimilarPairsDialog::new(
            dir_node, None, 0.0, None,
        )));
    });
}

#[test]
fn similar_pairs_results() {
    use arama_ui_widgets::dialog::similar_pairs_dialog::SimilarPairsDialog;
    use swdir::DirNode;

    run_matrix("similar_pairs_results", |app, _combo| {
        app.setup.finished = true;
        let dir_node = DirNode {
            path: missing_test_path(),
            sub_dirs: vec![],
            files: vec![],
        };
        app.dialog = Some(Dialog::SimilarPairsDialog(SimilarPairsDialog::new(
            dir_node,
            Some(vec![]),
            0.0,
            None,
        )));
    });
}

/// The three RFC 036 absence states - driven through the real message
/// path (`Message::SimilarPairsDialogMessage(..EmbeddingsReady..)`, the
/// same one `tests.rs`'s own
/// `stale_embeddings_ready_from_a_closed_scan_is_dropped_...` test
/// drives), not by poking a private field no other crate's test can
/// reach anyway.
/// `SimilarPair` itself stays private to `arama-ui-widgets` (only
/// `EmbeddingsReadyResult` was deliberately re-exported, for T061's
/// review 162) - every absence state's `items` is empty regardless, so
/// the three flags are enough to build each one without needing to name
/// that type here.
fn similar_pairs_absence_state(
    state_name: &'static str,
    had_errors: bool,
    nothing_indexed: bool,
    ffmpeg_missing_with_videos: bool,
) {
    use arama_ui_widgets::dialog::similar_pairs_dialog::{
        EmbeddingsReadyResult, SimilarPairsDialog, message as similar_pairs_dialog_message,
    };
    use swdir::DirNode;

    run_matrix(state_name, move |app, _combo| {
        app.setup.finished = true;
        let dir_node = DirNode {
            path: missing_test_path(),
            sub_dirs: vec![],
            files: vec![],
        };
        app.similar_pairs_scan_generation = 1;
        app.dialog = Some(Dialog::SimilarPairsDialog(SimilarPairsDialog::new(
            dir_node, None, 0.0, None,
        )));
        let mut result = EmbeddingsReadyResult::default();
        result.outcome.had_errors = had_errors;
        result.outcome.nothing_indexed = nothing_indexed;
        result.outcome.ffmpeg_missing_with_videos = ffmpeg_missing_with_videos;
        let _ = app.update(Message::SimilarPairsDialogMessage(
            similar_pairs_dialog_message::Message::EmbeddingsReady(1, result),
        ));
    });
}

#[test]
fn similar_pairs_nothing_indexed() {
    similar_pairs_absence_state("similar_pairs_nothing_indexed", false, true, false);
}

#[test]
fn similar_pairs_read_error() {
    similar_pairs_absence_state("similar_pairs_read_error", true, false, false);
}

#[test]
fn similar_pairs_ffmpeg_missing_with_videos() {
    similar_pairs_absence_state(
        "similar_pairs_ffmpeg_missing_with_videos",
        false,
        false,
        true,
    );
}
