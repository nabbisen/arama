//! T074 Phase 0 (RFC 053 §3): before the headless layout matrix (RFC
//! 053 §2.1) is built, this probe establishes - each by a planted
//! defect shown failing before it is trusted to pass - what
//! `iced_test::Simulator` can actually see. The matrix (a later
//! delivery) checks only the properties this file demonstrates it can
//! detect; a property not shown failing here is not claimed there.
//!
//! Every finding below was reached by reading the exact source cited,
//! not assumed from the crate's own doc comments, which in one case
//! (`cosmic_text::FontSystem::new()`, see item 4) turned out to describe
//! behaviour the function does not actually have in 0.15.0.

use super::*;

// ---------------------------------------------------------------------
// Shared infrastructure: finding every `Target::Text` in a tree.
// ---------------------------------------------------------------------
//
// `Simulator::find<S>` always runs `S`'s selector through the `One`
// strategy (`Selector::find`'s default body - confirmed by reading
// `iced_test-0.14.0/src/simulator.rs:104-129` and
// `iced_selector-0.14.0/src/lib.rs:36-41`), which stops traversing the
// widget tree as soon as `select()` returns `Some` once
// (`iced_selector-0.14.0/src/find.rs`'s `One::is_done`). `Selector::
// find_all()` exists and would visit every match, but nothing in
// `Simulator`'s public API ever calls it - `find` is the only operation-
// running method exposed, and its signature is hardcoded to `Find<S>`
// (`Finder<One<S>>`), not `FindAll<S>` (`Finder<All<S>>`): a trait
// method's return type cannot be widened by overriding it.
//
// The matrix needs every text in the tree, every time (RFC 053 §2.3
// property 1). The working technique, confirmed below: give the
// selector a shared, `Send` accumulator (`Arc<Mutex<_>>`), push every
// match into it from inside `select()`, and always return `None` from
// `select()` itself. `is_done()` then never becomes true, so traversal
// covers the whole tree; `Simulator::find` ends in `Err
// (SelectorNotFound)` (expected - the one-shot `output` was never set),
// but the accumulator, read back through the still-held `Arc` clone,
// has everything.
#[derive(Clone)]
struct CollectAllText {
    collected: std::sync::Arc<std::sync::Mutex<Vec<iced_test::selector::Target>>>,
}

impl iced_test::Selector for CollectAllText {
    type Output = ();

    fn select(&mut self, candidate: iced_test::selector::Candidate<'_>) -> Option<Self::Output> {
        if matches!(candidate, iced_test::selector::Candidate::Text { .. }) {
            self.collected
                .lock()
                .unwrap()
                .push(iced_test::selector::Target::from(candidate));
        }
        None
    }

    fn description(&self) -> String {
        "collect every Target::Text in the tree".to_owned()
    }
}

/// Runs [`CollectAllText`] over `element` and returns every text target
/// found, in traversal order. Generic over `Message` so it accepts both
/// a throwaway element built directly in a test (`Message = ()`) and a
/// real `App::view()` (`Message = core::message::Message`).
fn find_all_text<'a, Message: Clone + 'a>(
    element: impl Into<iced::Element<'a, Message, iced::Theme, iced::Renderer>>,
) -> Vec<iced_test::selector::Target> {
    let collected = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let selector = CollectAllText {
        collected: collected.clone(),
    };
    let mut simulator = iced_test::Simulator::new(element.into());
    // `Err(SelectorNotFound)` is the expected, permanent result of this
    // selector - see the module doc above.
    let _ = simulator.find(selector);
    std::sync::Arc::try_unwrap(collected)
        .expect("no other reference survives Simulator::find")
        .into_inner()
        .unwrap()
}

#[test]
fn infrastructure_find_all_text_sees_every_label_not_just_the_first() {
    use iced::widget::{column, container, text};

    let element: iced::Element<'_, (), iced::Theme, iced::Renderer> = container(column![
        text("alpha"),
        text("beta"),
        container(text("gamma")),
    ])
    .into();

    let found: Vec<String> = find_all_text(element)
        .into_iter()
        .filter_map(|target| match target {
            iced_test::selector::Target::Text { content, .. } => Some(content),
            _ => None,
        })
        .collect();

    assert_eq!(
        found,
        vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()],
        "the shared-accumulator technique must see every text in the tree, including \
         the one nested a container deeper, not stop after the first match"
    );
}

// ---------------------------------------------------------------------
// Item 1 (RFC 053 §3.1): clipping.
// ---------------------------------------------------------------------

#[test]
fn item_1_a_scrollable_viewport_shrinks_visible_bounds_below_a_clipped_labels_bounds() {
    use iced::widget::{container, scrollable, text};

    // A label far wider than a tiny scrollable viewport around it.
    let element: iced::Element<'_, (), iced::Theme, iced::Renderer> = container(
        scrollable(text("this label is much wider than the box that clips it"))
            .width(20.0)
            .height(20.0),
    )
    .into();

    let mut simulator = iced_test::Simulator::new(element);
    let found = simulator
        .find("this label is much wider than the box that clips it")
        .expect("the text must still be found even though it is clipped");

    let bounds = found.bounds();
    let visible = found.visible_bounds();
    assert!(
        visible.is_none_or(|v| v.width < bounds.width || v.height < bounds.height),
        "a label inside a too-small scrollable viewport must report a smaller \
         visible_bounds than bounds (or None) - bounds={bounds:?} visible={visible:?}"
    );
}

#[test]
fn item_1_b_container_clip_alone_does_not_shrink_visible_bounds() {
    use iced::widget::{container, text};

    // Finding: `iced_selector`'s `Finder` only narrows its tracked
    // `viewport` inside `Finder::scrollable` (confirmed by reading
    // `iced_selector-0.14.0/src/find.rs:190-216`); `Finder::container`
    // only intersects *against* whatever `viewport` already is, never
    // narrows it for its own bounds. A plain `Container::clip(true)`
    // visually clips when drawn, but is invisible to `visible_bounds` -
    // only a `scrollable` ancestor (item 1a) is seen by this harness.
    let element: iced::Element<'_, (), iced::Theme, iced::Renderer> =
        container(text("this label is much wider than the box that clips it"))
            .width(20.0)
            .height(20.0)
            .clip(true)
            .into();

    let mut simulator = iced_test::Simulator::new(element);
    let found = simulator
        .find("this label is much wider than the box that clips it")
        .expect("the text must still be found even though it visually clips");

    let bounds = found.bounds();
    let visible = found.visible_bounds();
    assert_eq!(
        visible,
        Some(bounds),
        "Container::clip(true) alone is NOT seen by visible_bounds - if this ever \
         changes (an iced upgrade), this is the property to re-check before the matrix \
         starts relying on plain Container::clip for anything - bounds={bounds:?} \
         visible={visible:?}"
    );
}

// ---------------------------------------------------------------------
// Item 2 (RFC 053 §3.2): non-clipping overflow - the Japanese-footer
// shape. Three candidates, as RFC 053 names them.
// ---------------------------------------------------------------------

#[test]
fn item_2_candidates_1_and_2_bounds_based_checks_cannot_see_non_clipping_overflow() {
    use iced::widget::Id;
    use iced::widget::{container, text};

    // Finding: a widget's `bounds`, as reported through `Candidate`/
    // `Target`, is the box its parent ALLOCATED to it during layout, not
    // its intrinsic/measured content size - true with or without
    // `Wrapping::None` (ruling out "it only looks this way because it
    // wrapped into a taller box"). The overflow this produces at paint
    // time is real (confirmed visually by item 2's candidate 3 below)
    // but the layout tree's own geometry never represents it, so a
    // widget whose content overflows its box reports the SAME bounds as
    // that box - candidate 1 (visible_bounds alone) and candidate 2
    // (comparing the text's bounds against its container's) can never
    // detect this, not merely in this one example.
    let container_id = Id::new("item-2-fixed-width-container");
    let element: iced::Element<'_, (), iced::Theme, iced::Renderer> = container(
        text("this label is much wider than the fixed-width box around it")
            .wrapping(iced::widget::text::Wrapping::None),
    )
    .id(container_id.clone())
    .width(20.0)
    .into();

    let all = find_all_text(element);
    let text_target = all
        .into_iter()
        .find(|target| matches!(target, iced_test::selector::Target::Text { .. }))
        .expect("the label must be found");
    let text_bounds = text_target.bounds();
    let text_visible = text_target.visible_bounds();

    // The container's own bounds, found the same way `id()` selection
    // works in production code (`iced_test::selector::id`).
    let container_bounds = {
        let element: iced::Element<'_, (), iced::Theme, iced::Renderer> = container(
            text("this label is much wider than the fixed-width box around it")
                .wrapping(iced::widget::text::Wrapping::None),
        )
        .id(container_id)
        .width(20.0)
        .into();
        let mut simulator = iced_test::Simulator::new(element);
        simulator
            .find(iced_test::selector::id("item-2-fixed-width-container"))
            .expect("the container must be found by its id")
            .bounds()
    };

    let candidate_1 = text_visible.is_some_and(|v| v.width < text_bounds.width);
    let candidate_2 = text_bounds.width > container_bounds.width;

    assert!(
        !candidate_1 && !candidate_2,
        "recording the expected negative finding: neither visible_bounds alone \
         (candidate_1={candidate_1}) nor a text-vs-container bounds comparison \
         (candidate_2={candidate_2}) detects a non-clipping overflow - \
         text_bounds={text_bounds:?} container_bounds={container_bounds:?} \
         text_visible={text_visible:?}; see candidate 3 for what does work"
    );
}

#[test]
fn item_2_candidate_3_a_pixel_check_past_the_edge_sees_non_clipping_overflow() {
    use iced::widget::{container, text};

    let scratch = std::env::temp_dir().join(format!(
        "arama-t074-phase0-pixel-check-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();

    // `Snapshot` exposes no public accessor for its raw pixels (its
    // `screenshot` field is private to `iced_test` - confirmed by
    // reading `iced_test-0.14.0/src/simulator.rs:252-257`), so this goes
    // through the one public door that does touch them: `matches_image`
    // writes a PNG to a scratch path on its first call (no reference
    // exists yet) and returns `Ok(true)`. Decode that PNG back with
    // `png` (already in Cargo.lock - `iced_test` itself depends on it
    // for the identical comparison, confirmed by reading
    // `iced_test-0.14.0/src/simulator.rs:270-277` - this adds a usage
    // edge to an existing lockfile entry, not a new dependency) and
    // inspect pixels past the container's right edge directly.
    const CONTAINER_WIDTH: f32 = 20.0;
    let element: iced::Element<'_, (), iced::Theme, iced::Renderer> = container(
        text("this label is much wider than the fixed-width box around it")
            .wrapping(iced::widget::text::Wrapping::None)
            .color(iced::Color::WHITE),
    )
    .width(CONTAINER_WIDTH)
    .style(|_theme| container::Style {
        background: Some(iced::Color::BLACK.into()),
        ..container::Style::default()
    })
    .into();

    let mut simulator = iced_test::Simulator::with_size(
        iced::Settings::default(),
        iced::Size::new(200.0, 60.0),
        element,
    );
    let theme = arama_theme::iced_theme();
    let snapshot = simulator
        .snapshot(&theme)
        .expect("Simulator::snapshot must render successfully on this hardware");

    let image_path = scratch.join("overflow.png");
    let first_call = snapshot
        .matches_image(&image_path)
        .expect("writing the reference PNG must not fail");
    assert!(first_call, "a freshly created reference must match itself");

    // `Snapshot::matches_image` suffixes the path with `-<renderer>`
    // (confirmed by reading `Simulator::path`) - find whatever it
    // actually wrote rather than assuming the exact name.
    let written = std::fs::read_dir(&scratch)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "png"))
        .expect("matches_image must have written exactly one PNG");

    let file = std::io::BufReader::new(std::fs::File::open(&written).unwrap());
    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().unwrap();
    let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buffer).unwrap();
    let rgba = &buffer[..info.buffer_size()];
    let width = info.width as usize;
    let height = info.height as usize;

    // `Simulator::snapshot` always renders at a fixed 2x scale factor
    // (confirmed by reading its source: `let scale_factor = 2.0;`, not
    // configurable), so the container's right edge in physical pixels
    // is at `CONTAINER_WIDTH * 2.0`.
    let edge_x = (CONTAINER_WIDTH * 2.0).round() as usize;

    let mut found_overflow_pixel = false;
    'scan: for y in 0..height {
        for x in edge_x..width {
            let i = (y * width + x) * 4;
            let (r, g, b, a) = (rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]);
            // The label is white text on a black background; anything
            // but fully-transparent or black past the edge is the
            // label's own ink overflowing it.
            if a > 0 && (r, g, b) != (0, 0, 0) {
                found_overflow_pixel = true;
                break 'scan;
            }
        }
    }

    assert!(
        found_overflow_pixel,
        "a pixel check past the container's right edge (x >= {edge_x} of {width}) must \
         detect the label's own ink overflowing it, where item 2's bounds-based \
         candidates cannot - checked {written:?}"
    );

    let _ = std::fs::remove_dir_all(&scratch);
}

// ---------------------------------------------------------------------
// Item 3 (RFC 053 §3.3): fonts - the icon font and the Japanese font.
// ---------------------------------------------------------------------

#[test]
fn item_3_a_icon_measured_width_differs_with_and_without_the_lucide_font() {
    use lucide_icons::LUCIDE_FONT_BYTES;

    fn measured_width(settings: iced::Settings) -> f32 {
        let icon = lucide_icons::iced::icon_database();
        let element: iced::Element<'_, (), iced::Theme, iced::Renderer> = icon.into();
        let mut simulator = iced_test::Simulator::with_settings(settings, element);
        let found = simulator
            .find("\u{e0ad}")
            .expect("the icon's own text content must be findable either way");
        found.bounds().width
    }

    // Order matters, and is itself a finding: `iced_graphics::text::
    // font_system()` is a process-wide `OnceLock` singleton (confirmed
    // by reading `iced_graphics-0.14.0/src/text.rs:115-134`), and
    // `Settings::fonts` bytes are loaded into it permanently -
    // `FontSystem::load_font` has no corresponding unload. "Without"
    // must run first in this process, or any earlier test in the same
    // binary that already loaded Lucide would make this comparison
    // measure nothing (an earlier draft of this test got exactly that
    // false negative by measuring "with" first).
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

#[test]
fn item_3_b_measuring_japanese_by_requesting_a_font_name_does_not_distinguish_coverage() {
    use iced::Font;

    fn measured_width(font: Font) -> f32 {
        let element: iced::Element<'_, (), iced::Theme, iced::Renderer> =
            iced::widget::text("設定を保存できませんでした")
                .font(font)
                .wrapping(iced::widget::text::Wrapping::None)
                .into();
        let mut simulator = iced_test::Simulator::new(element);
        let found = simulator
            .find("設定を保存できませんでした")
            .expect("the label must be findable regardless of which font renders it");
        found.bounds().width
    }

    // "Fira Sans" is `iced_test::Simulator`'s own substitute for
    // `Font::DEFAULT` (confirmed by reading `Simulator::with_size`) - a
    // Latin font with no CJK coverage. Requesting it, or requesting
    // "Noto Sans CJK JP" by name (present on this machine, confirmed via
    // `fc-list :lang=ja family`, matching the handoff's claim), produces
    // the SAME measured width. Finding: neither name is ever loaded into
    // cosmic-text's font database (only `Settings::fonts` bytes and the
    // bundled Iced-Icons font are - see item 3a), so an unmatched family
    // name does not change which fallback path runs; it is the SAME
    // fallback either way. This candidate cannot tell "a real font was
    // used" from "nothing matched, fell back anyway" apart - item 4
    // answers the actual question by querying cosmic-text's shaping
    // directly instead of by name.
    let with_latin_name = measured_width(Font::with_name("Fira Sans"));
    let with_japanese_name = measured_width(Font::with_name("Noto Sans CJK JP"));

    assert_eq!(
        with_latin_name, with_japanese_name,
        "recording the expected finding: requesting an unmatched font family by name \
         does not change the measured width (Fira Sans={with_latin_name}, \
         Noto Sans CJK JP={with_japanese_name}) - both fall through to the same \
         fallback. Measuring 'with vs. without a font' this way is not a usable \
         signal; see item 4 for the direct check."
    );
}

// ---------------------------------------------------------------------
// Item 4 (RFC 053 §3.4, §2.4): which font face draws a kanji vs. a
// kana, queried directly through cosmic-text's own shaping API.
// ---------------------------------------------------------------------

/// `iced::advanced::graphics::text::{font_system, cosmic_text}` are
/// confirmed public re-exports reaching `iced_graphics::text::{
/// font_system, cosmic_text}` - no second `cosmic-text` copy enters the
/// lockfile (handoff §6.1 item 4's explicit instruction).
#[test]
fn item_4_kanji_and_kana_in_one_label_select_different_font_faces() {
    use iced::advanced::graphics::text::cosmic_text;

    let mut font_system = iced::advanced::graphics::text::font_system()
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let raw = font_system.raw();

    let mut buffer = cosmic_text::Buffer::new(raw, cosmic_text::Metrics::new(16.0, 20.0));
    let attrs = cosmic_text::Attrs::new();
    // 設 (kanji) and あ (kana) in one label - the same shape as a real
    // Japanese toast or menu string arama already ships.
    let text = "設あ";
    buffer.set_text(raw, text, &attrs, cosmic_text::Shaping::Advanced, None);

    let mut faces: Vec<(String, Option<String>)> = Vec::new();
    for run in buffer.layout_runs() {
        for glyph in run.glyphs {
            let source_char = text[glyph.start..glyph.end].to_owned();
            let family = raw.db_mut().face(glyph.font_id).map(|face| {
                face.families
                    .first()
                    .map(|(name, _lang)| name.clone())
                    .unwrap_or_else(|| format!("{:?}", face.id))
            });
            faces.push((source_char, family));
        }
    }
    let locale = raw.locale().to_owned();

    // RFC 053 §2.4's finding, confirmed rather than left "not yet
    // observed": on this machine, 設 selects "Noto Sans CJK SC"
    // (Simplified Chinese) while あ, in the same label, selects "Noto
    // Sans CJK JP" - one label, two font faces. `raw.locale()` reports
    // "en-US" (the system locale `sys_locale` reads), never "ja" and
    // never arama's own `arama_i18n::set_locale` - confirming the RFC's
    // exact claim that the Japanese-specific branch of cosmic-text's
    // fallback-by-script never fires for this process, regardless of
    // which language arama itself is displaying.
    eprintln!("=== T074 Phase 0 item 4: font face selected per character: {faces:?} ===");
    eprintln!("=== T074 Phase 0 item 4: locale cosmic-text reports: {locale:?} ===");

    assert!(
        !faces.is_empty(),
        "shaping a non-empty Japanese string must produce at least one glyph"
    );
    assert!(
        faces.iter().all(|(_, family)| family.is_some()),
        "every glyph should resolve to *some* face - a glyph with no face at all would \
         mean cosmic-text could not shape this string here at all: {faces:?}"
    );
    // Recorded, not asserted pass/fail on which specific face is chosen
    // (RFC 053 §3 item 4: "recorded, not asserted - it is evidence for
    // §2.4's finding, not a property of the layout"). Phase 0's own job
    // is only to confirm the query mechanism works and to capture what
    // it currently reports, here and on each CI runner (native-smoke.yaml's
    // `--nocapture` default already surfaces this eprintln on macOS and
    // Windows for the architect to read after pushing).
}

// ---------------------------------------------------------------------
// Item 5 (RFC 053 §3, §2.1): which screen states can be built headless.
// ---------------------------------------------------------------------
//
// Not a test - a survey, reported here since it has no separate home.
// Every state below is constructed the same way every other test in
// `app/src/core/tests.rs` already does: direct field mutation on a real
// `App::new()`, no new seam needed.
//
//   - Setup wizard: the default state (`setup.finished == false`).
//   - Gallery, empty: `setup.finished = true`, no `dir_node`.
//   - Gallery, populated: `setup.finished = true` plus a real `dir_node`
//     and a seeded cache - heavier (the same fixture shape
//     `run_thumbnail_phase`'s own tests already build), not attempted
//     in this probe; reported as buildable, not yet built.
//   - Settings: `nav_page = NavPage::Settings` (renders `settings_page`
//     directly - confirmed by reading `app/src/core/view.rs:142`; there
//     is no separate `Dialog::SettingsDialog` variant).
//   - Cache page: `nav_page = NavPage::Cache`.
//   - Media Focus dialog: `dialog = Some(Dialog::MediaFocusDialog(..))`
//     (already exercised by `tier_2_escape_closes_a_real_open_dialog`
//     in `tests.rs`).
//   - Similar Pairs dialog, with results and its absence states:
//     `dialog = Some(Dialog::SimilarPairsDialog(SimilarPairsDialog::new(..)))`
//     (already exercised in `tests.rs`); the absence-state variants need
//     constructing `SimilarPairsDialog` with different inputs, not a new
//     seam.
//   - Confirm dialog: `dialog = Some(Dialog::Confirm(ConfirmDialog::new(..)))`.
//   - A toast: `app.toasts.push(..)` (already exercised in `tests.rs`).
//
// All eight are buildable headless through state already public enough
// within this crate for a test to reach. None needs a new constructor
// or test-only seam on `App` itself.

// ---------------------------------------------------------------------
// Item 6 (RFC 053 §2.5): per-combination run time, projected against
// the matrix's 60 s Linux-runner budget.
// ---------------------------------------------------------------------

#[test]
fn item_6_per_combination_run_time_projection() {
    let _guard = ARAMA_DATA_HOME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os(arama_env::DATA_HOME_ENV_VAR);
    let scratch = scratch_data_home("t074-phase0-run-time-projection");
    unsafe {
        std::env::set_var(arama_env::DATA_HOME_ENV_VAR, &scratch);
    }

    const ITERATIONS: usize = 10;

    let mut app = App::new().0;
    app.setup.finished = true;
    let started = std::time::Instant::now();
    for _ in 0..ITERATIONS {
        let element = app.view();
        let _ = find_all_text(element);
    }
    let elapsed = started.elapsed();
    let per_combination = elapsed / ITERATIONS as u32;

    // Finding: the default backend (`wgpu`, first in `Simulator`'s own
    // fallback candidate list) costs roughly 100x a plain CPU render
    // here - confirmed by re-running this same test with
    // `ICED_TEST_BACKEND=tiny-skia` set (≈1ms/combination vs.
    // ≈110-125ms/combination), with every other Phase 0 property above
    // (clipping, the pixel check, both font checks) producing identical
    // results under either backend. 72 combinations x even a handful of
    // screen states risks the 60s budget under the default backend;
    // under `tiny-skia` it does not. This is the concrete number the
    // matrix delivery should act on (RFC 053 §2.5: split, never trim, if
    // over budget) - recorded here, not resolved, since choosing the
    // matrix's own backend is that delivery's decision to make and test.
    eprintln!(
        "=== T074 Phase 0 item 6: {ITERATIONS} combinations (gallery, empty, 760x520), \
         default backend, App reused: {elapsed:?} ({per_combination:?} each) ==="
    );
    eprintln!(
        "=== T074 Phase 0 item 6: projected for 72 combinations x 1 screen state: {:?} ===",
        per_combination * 72
    );

    unsafe {
        match &previous {
            Some(value) => std::env::set_var(arama_env::DATA_HOME_ENV_VAR, value),
            None => std::env::remove_var(arama_env::DATA_HOME_ENV_VAR),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}
