use std::cell::Cell;

use arama_env::ffmpeg_location::FfmpegLocationPreference;

use super::{
    SelectedReadyRoute, SelectionPickPlan, plan_selection_pick, rollback_revalidation_preference,
    route_selected_ready, run_current_picker, run_current_selection,
    state::{
        AuthorityStatus, AuthorityTerminal, FfmpegAuthority, RollbackAction, SelectionResolution,
    },
};
use crate::core::message::FfmpegRequestIntent;

#[test]
fn picker_cancel_is_silent_but_relative_preflight_is_actionable() {
    let current = FfmpegLocationPreference::Auto;
    assert_eq!(
        plan_selection_pick(&current, None),
        SelectionPickPlan::Cancelled
    );
    assert!(matches!(
        plan_selection_pick(&current, Some("relative".into())),
        SelectionPickPlan::Rejected { .. }
    ));
}

#[cfg(unix)]
#[test]
fn non_unicode_picker_preflight_is_actionable() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

    let path = PathBuf::from(OsString::from_vec(b"/tmp/ffmpeg-\xff".to_vec()));
    assert!(matches!(
        plan_selection_pick(&FfmpegLocationPreference::Auto, Some(path)),
        SelectionPickPlan::Rejected { .. }
    ));
}

#[test]
fn stale_selection_cannot_enter_the_app_persistence_operation() {
    let mut authority = FfmpegAuthority::<String>::new(FfmpegLocationPreference::Auto);
    let stale = authority.begin(
        FfmpegRequestIntent::Selection,
        FfmpegLocationPreference::SelectedDirectory("/a".into()),
    );
    let current = authority.begin(
        FfmpegRequestIntent::Selection,
        FfmpegLocationPreference::SelectedDirectory("/b".into()),
    );
    let save_calls = Cell::new(0);
    assert_eq!(
        run_current_selection(&authority, stale, || save_calls.set(save_calls.get() + 1)),
        None
    );
    assert_eq!(save_calls.get(), 0);
    assert_eq!(
        run_current_selection(&authority, current, || save_calls.set(save_calls.get() + 1)),
        Some(())
    );
    assert_eq!(save_calls.get(), 1);
}

#[test]
fn checking_rollback_is_mapped_to_a_fresh_revalidation_request() {
    let preference = FfmpegLocationPreference::SelectedDirectory("/prior".into());
    let resolution = SelectionResolution::Retained(RollbackAction::Revalidate(preference.clone()));
    assert_eq!(
        rollback_revalidation_preference(&resolution),
        Some(&preference)
    );
    assert_eq!(
        rollback_revalidation_preference(&SelectionResolution::Retained(RollbackAction::Restored)),
        None
    );
}

#[test]
fn second_picker_and_picker_during_candidate_validation_are_not_started() {
    let mut authority = FfmpegAuthority::<String>::new(FfmpegLocationPreference::Auto);
    let picker_a = authority.begin_picker().unwrap();
    assert_eq!(authority.begin_picker(), None);
    assert!(authority.accept_picker(picker_a));

    let selection_a = authority.begin(
        FfmpegRequestIntent::Selection,
        FfmpegLocationPreference::SelectedDirectory("/a".into()),
    );
    assert_eq!(authority.begin_picker(), None);
    assert!(authority.is_current(selection_a));
}

#[test]
fn clear_and_recheck_drop_late_picker_results_before_planning() {
    for intent in [
        FfmpegRequestIntent::ClearToAuto,
        FfmpegRequestIntent::Recheck,
    ] {
        let mut authority = FfmpegAuthority::<String>::new(FfmpegLocationPreference::Auto);
        let picker = authority.begin_picker().unwrap();
        authority.begin(intent, FfmpegLocationPreference::Auto);
        let plan_calls = Cell::new(0);
        assert_eq!(
            run_current_picker(&mut authority, picker, || {
                plan_calls.set(plan_calls.get() + 1);
                SelectionPickPlan::Cancelled
            }),
            None
        );
        assert_eq!(plan_calls.get(), 0);
    }
}

#[test]
fn rejected_overlapping_pick_cannot_exist_or_supersede_selection_a() {
    let mut authority = FfmpegAuthority::<String>::new(FfmpegLocationPreference::Auto);
    let selection_a = authority.begin(
        FfmpegRequestIntent::Selection,
        FfmpegLocationPreference::SelectedDirectory("/a".into()),
    );
    assert_eq!(authority.begin_picker(), None);

    let save_calls = Cell::new(0);
    assert_eq!(
        run_current_selection(&authority, selection_a, || save_calls
            .set(save_calls.get() + 1)),
        Some(())
    );
    assert_eq!(save_calls.get(), 1);
}

// --- SelectedReady dispatch seam (review 067 Finding 1 / task 005) --------
//
// These tests exercise `route_selected_ready`, the exact function
// `handle_ffmpeg_discovery_event`'s `SelectedReady` arm calls to decide its
// branch, plus the same `publish_terminal` sequence the Terminal branch runs
// in production. Task 005 required covering the dispatch seam itself, not
// only `FfmpegAuthority`'s state machine in isolation (which was already
// correct and already covered above/in `state/tests.rs`).

#[test]
fn selected_ready_routes_startup_and_recheck_to_terminal() {
    for intent in [FfmpegRequestIntent::Startup, FfmpegRequestIntent::Recheck] {
        let mut authority = FfmpegAuthority::<String>::new(FfmpegLocationPreference::Auto);
        let epoch = authority.begin(
            intent,
            FfmpegLocationPreference::SelectedDirectory("/usr/bin".into()),
        );
        assert_eq!(
            route_selected_ready(&authority, epoch),
            SelectedReadyRoute::Terminal,
            "{intent:?} must route to Terminal, not be silently dropped"
        );
    }
}

#[test]
fn selected_ready_routes_selection_unchanged() {
    let mut authority = FfmpegAuthority::<String>::new(FfmpegLocationPreference::Auto);
    let epoch = authority.begin(
        FfmpegRequestIntent::Selection,
        FfmpegLocationPreference::SelectedDirectory("/usr/bin".into()),
    );
    assert_eq!(
        route_selected_ready(&authority, epoch),
        SelectedReadyRoute::Selection
    );
}

#[test]
fn selected_ready_routes_clear_to_auto_and_stale_epoch_to_stale() {
    let mut authority = FfmpegAuthority::<String>::new(FfmpegLocationPreference::Auto);
    let clear_epoch = authority.begin(
        FfmpegRequestIntent::ClearToAuto,
        FfmpegLocationPreference::Auto,
    );
    assert_eq!(
        route_selected_ready(&authority, clear_epoch),
        SelectedReadyRoute::Stale
    );

    // A stale epoch (superseded by a newer transaction) has no live intent.
    let superseded = clear_epoch;
    authority.begin(FfmpegRequestIntent::Recheck, FfmpegLocationPreference::Auto);
    assert_eq!(
        route_selected_ready(&authority, superseded),
        SelectedReadyRoute::Stale
    );
}

#[test]
fn selected_ready_under_startup_reaches_ready_terminal_not_stuck_checking() {
    let preference = FfmpegLocationPreference::SelectedDirectory("/usr/bin".into());
    let mut authority = FfmpegAuthority::<String>::new(preference.clone());
    let epoch = authority.begin(FfmpegRequestIntent::Startup, preference.clone());

    // Matches the Started arm's behaviour: startup enters Checking first.
    assert_eq!(authority.status(), &AuthorityStatus::Checking);

    // What the fixed Terminal branch does: derive the toolchain from the
    // validated outcome (already proven correct via review 068's approved
    // `ValidatedSelection::outcome()` widening) and publish it directly.
    assert_eq!(
        route_selected_ready(&authority, epoch),
        SelectedReadyRoute::Terminal
    );
    let published =
        authority.publish_terminal(epoch, AuthorityTerminal::Ready("toolchain-stub".to_owned()));

    assert!(
        published,
        "a live epoch's terminal publication must succeed"
    );
    assert_eq!(authority.status(), &AuthorityStatus::Ready);
    assert_ne!(
        authority.status(),
        &AuthorityStatus::Checking,
        "review 067 Finding 1: a valid persisted Selected preference must not hang on Checking"
    );
    assert_eq!(
        authority.toolchain().map(String::as_str),
        Some("toolchain-stub")
    );
    assert_eq!(authority.preference(), &preference);
}

#[test]
fn selected_ready_under_recheck_reaches_ready_terminal_not_stuck_checking() {
    let preference = FfmpegLocationPreference::SelectedDirectory("/usr/bin".into());
    let mut authority = FfmpegAuthority::<String>::new(preference.clone());
    // Recheck begins from an already-published Ready authority, unlike
    // Startup which begins from a fresh authority -- exercised separately so
    // a fix that only works from the initial state is not mistaken for
    // covering both call sites named in review 067 Finding 1.
    let first = authority.begin(FfmpegRequestIntent::Startup, preference.clone());
    assert!(authority.publish_terminal(first, AuthorityTerminal::Ready("initial".to_owned())));

    let epoch = authority.begin(FfmpegRequestIntent::Recheck, preference.clone());
    assert_eq!(authority.status(), &AuthorityStatus::Checking);
    assert_eq!(
        route_selected_ready(&authority, epoch),
        SelectedReadyRoute::Terminal
    );
    let published =
        authority.publish_terminal(epoch, AuthorityTerminal::Ready("revalidated".to_owned()));

    assert!(published);
    assert_eq!(authority.status(), &AuthorityStatus::Ready);
    assert_eq!(
        authority.toolchain().map(String::as_str),
        Some("revalidated")
    );
}

// Test 4 from task 005 §4 ("no settings save occurs for Startup/Recheck",
// asserted positively rather than via an unchanged-value check) is not
// exercised as a dynamic test here. `handle_ffmpeg_discovery_event`'s
// Terminal branch (see `ffmpeg.rs`) contains no `ConfigManager`/save call at
// all for that route -- confirmed by reading the diff, not merely inferred
// -- so there is no save path to invoke in the first place. No test in this
// suite constructs a full `App` for it, consistent with this file's own
// scope (`FfmpegAuthority` and the free functions in `ffmpeg.rs`, not
// `App::new()`'s wider startup sequence). This is the "if the current
// structure makes that unobservable, say so" case task 005 anticipated.
//
// (RFC 041, historical note: this comment previously cited `App::new()`
// resolving settings against the test process's CWD -- `app/`, where a
// real `app/settings.json` could live -- as the specific reason to avoid
// constructing an `App` here. That specific risk no longer applies:
// `App::new()` no longer touches the CWD at all. `core::view::tests`
// demonstrates constructing an `App` safely via the `ARAMA_DATA_HOME`
// override. The absence of a dynamic test for this case is still correct,
// just for the narrower, file-scope reason above, not the wider one this
// comment used to give.)

// Test 5 from task 005 §4 ("an invalid persisted Selected preference still
// reports its typed failure") is a `Published`-event path, not
// `SelectedReady` -- an invalid pair fails validation inside the worker,
// which emits `Published` with a non-Ready outcome, never `SelectedReady`
// (see review 067 Finding 1, "the probe-absence observation was a red
// herring": a failed validation is handled by the unchanged `Published`
// arm). That arm was not touched by this fix. Coverage for a Failed
// terminal published under a non-Selection intent already exists at
// `state/tests.rs::invalid_and_save_failure_restore_exact_selected_failure`
// (`authority.begin(FfmpegRequestIntent::Startup, ...)` followed by
// `publish_terminal(_, AuthorityTerminal::Failed(_))`), so it is not
// duplicated here.
