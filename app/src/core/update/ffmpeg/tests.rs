use std::cell::Cell;

use arama_env::ffmpeg_location::FfmpegLocationPreference;

use super::{
    SelectionPickPlan, plan_selection_pick, rollback_revalidation_preference, run_current_picker,
    run_current_selection,
    state::{FfmpegAuthority, RollbackAction, SelectionResolution},
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
