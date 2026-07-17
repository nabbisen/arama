use std::path::PathBuf;

use arama_env::ffmpeg_location::FfmpegLocationPreference;
use arama_sidecar::media::video::video_engine::discovery::{
    FfmpegDiscoveryFailure, FfmpegDiscoveryOutcome, FilesystemIssue,
};

use super::{
    AuthorityStatus, AuthorityTerminal, FfmpegAuthority, RollbackAction, SelectionResolution,
};
use crate::core::message::FfmpegRequestIntent;

fn selected(name: &str) -> FfmpegLocationPreference {
    FfmpegLocationPreference::SelectedDirectory(PathBuf::from(format!("/{name}")))
}

fn ready_authority(
    preference: FfmpegLocationPreference,
    toolchain: &str,
) -> FfmpegAuthority<String> {
    let mut authority = FfmpegAuthority::new(preference.clone());
    let epoch = authority.begin(FfmpegRequestIntent::Startup, preference);
    assert!(authority.publish_terminal(epoch, AuthorityTerminal::Ready(toolchain.to_owned())));
    authority
}

fn failure(outcome: FfmpegDiscoveryOutcome) -> FfmpegDiscoveryFailure {
    FfmpegDiscoveryFailure::try_from(outcome).unwrap()
}

#[test]
fn invalid_selection_retains_auto_ready_authority() {
    let mut authority = ready_authority(FfmpegLocationPreference::Auto, "auto");
    let epoch = authority.begin(FfmpegRequestIntent::Selection, selected("candidate"));
    assert_eq!(
        authority.retain_prior(epoch),
        SelectionResolution::Retained(RollbackAction::Restored)
    );
    assert_eq!(authority.preference(), &FfmpegLocationPreference::Auto);
    assert_eq!(authority.toolchain().map(String::as_str), Some("auto"));
    assert_eq!(authority.status(), &AuthorityStatus::Ready);
}

#[test]
fn invalid_selection_retains_selected_ready_authority() {
    let prior = selected("prior");
    let mut authority = ready_authority(prior.clone(), "prior-pair");
    let epoch = authority.begin(FfmpegRequestIntent::Selection, selected("candidate"));
    assert_eq!(
        authority.retain_prior(epoch),
        SelectionResolution::Retained(RollbackAction::Restored)
    );
    assert_eq!(authority.preference(), &prior);
    assert_eq!(
        authority.toolchain().map(String::as_str),
        Some("prior-pair")
    );
    assert_eq!(authority.status(), &AuthorityStatus::Ready);
}

#[test]
fn save_failure_retains_auto_ready_authority() {
    let mut authority = ready_authority(FfmpegLocationPreference::Auto, "auto");
    let epoch = authority.begin(FfmpegRequestIntent::Selection, selected("candidate"));
    assert_eq!(
        authority.finish_selection_persistence::<()>(epoch, Err(())),
        SelectionResolution::Retained(RollbackAction::Restored)
    );
    assert_eq!(authority.preference(), &FfmpegLocationPreference::Auto);
    assert_eq!(authority.toolchain().map(String::as_str), Some("auto"));
    assert_eq!(authority.status(), &AuthorityStatus::Ready);
}

#[test]
fn save_failure_retains_selected_ready_authority() {
    let prior = selected("prior");
    let mut authority = ready_authority(prior.clone(), "prior-pair");
    let epoch = authority.begin(FfmpegRequestIntent::Selection, selected("candidate"));
    assert_eq!(
        authority.finish_selection_persistence::<()>(epoch, Err(())),
        SelectionResolution::Retained(RollbackAction::Restored)
    );
    assert_eq!(authority.preference(), &prior);
    assert_eq!(
        authority.toolchain().map(String::as_str),
        Some("prior-pair")
    );
    assert_eq!(authority.status(), &AuthorityStatus::Ready);
}

#[test]
fn clear_and_recheck_invalidate_capture_synchronously() {
    for intent in [
        FfmpegRequestIntent::ClearToAuto,
        FfmpegRequestIntent::Recheck,
    ] {
        let mut authority = ready_authority(selected("prior"), "pair");
        authority.begin(intent, FfmpegLocationPreference::Auto);
        assert!(authority.toolchain().is_none());
        assert_eq!(authority.status(), &AuthorityStatus::Checking);
    }
}

#[test]
fn clear_after_queued_selection_rejects_the_stale_selection_epoch() {
    let mut authority = ready_authority(FfmpegLocationPreference::Auto, "auto");
    let selection_epoch = authority.begin(FfmpegRequestIntent::Selection, selected("a"));
    let clear_epoch = authority.begin(
        FfmpegRequestIntent::ClearToAuto,
        FfmpegLocationPreference::Auto,
    );
    assert_eq!(
        authority.finish_selection_persistence::<()>(
            selection_epoch,
            Ok((selected("a"), "a".to_owned()))
        ),
        SelectionResolution::Stale
    );
    assert!(
        authority.publish_terminal(clear_epoch, AuthorityTerminal::Ready("new-auto".to_owned()))
    );
    assert_eq!(authority.toolchain().map(String::as_str), Some("new-auto"));
}

#[test]
fn selection_b_rejects_a_queued_selection_a_completion() {
    let mut authority = ready_authority(FfmpegLocationPreference::Auto, "auto");
    let epoch_a = authority.begin(FfmpegRequestIntent::Selection, selected("a"));
    let epoch_b = authority.begin(FfmpegRequestIntent::Selection, selected("b"));
    assert_eq!(
        authority.finish_selection_persistence::<()>(epoch_a, Ok((selected("a"), "a".to_owned()))),
        SelectionResolution::Stale
    );
    assert_eq!(
        authority.finish_selection_persistence::<()>(epoch_b, Ok((selected("b"), "b".to_owned()))),
        SelectionResolution::Published
    );
    assert_eq!(authority.preference(), &selected("b"));
    assert_eq!(authority.toolchain().map(String::as_str), Some("b"));
}

#[test]
fn invalid_and_save_failure_restore_exact_selected_failure() {
    for save_failure in [false, true] {
        let prior = selected("prior");
        let prior_failure = failure(FfmpegDiscoveryOutcome::LegacyLocationExcluded);
        let mut authority = FfmpegAuthority::<String>::new(prior.clone());
        let initial = authority.begin(FfmpegRequestIntent::Startup, prior.clone());
        assert!(
            authority.publish_terminal(initial, AuthorityTerminal::Failed(prior_failure.clone()))
        );
        let epoch = authority.begin(FfmpegRequestIntent::Selection, selected("candidate"));
        let resolution = if save_failure {
            authority.finish_selection_persistence::<()>(epoch, Err(()))
        } else {
            authority.retain_prior(epoch)
        };
        assert_eq!(
            resolution,
            SelectionResolution::Retained(RollbackAction::Restored)
        );
        assert_eq!(authority.preference(), &prior);
        assert_eq!(authority.status(), &AuthorityStatus::Failed(prior_failure));
        assert!(authority.toolchain().is_none());
    }
}

#[test]
fn invalid_and_save_failure_restore_exact_auto_filesystem_failure() {
    for save_failure in [false, true] {
        let prior_failure = failure(FfmpegDiscoveryOutcome::FilesystemUnavailable(
            FilesystemIssue::Access,
        ));
        let mut authority = FfmpegAuthority::<String>::new(FfmpegLocationPreference::Auto);
        let initial = authority.begin(FfmpegRequestIntent::Startup, FfmpegLocationPreference::Auto);
        assert!(
            authority.publish_terminal(initial, AuthorityTerminal::Failed(prior_failure.clone()))
        );
        let epoch = authority.begin(FfmpegRequestIntent::Selection, selected("candidate"));
        let resolution = if save_failure {
            authority.finish_selection_persistence::<()>(epoch, Err(()))
        } else {
            authority.retain_prior(epoch)
        };
        assert_eq!(
            resolution,
            SelectionResolution::Retained(RollbackAction::Restored)
        );
        assert_eq!(authority.preference(), &FfmpegLocationPreference::Auto);
        assert_eq!(authority.status(), &AuthorityStatus::Failed(prior_failure));
    }
}

#[test]
fn selection_rollback_from_startup_or_recheck_requires_live_revalidation() {
    for (prior_intent, save_failure) in [
        (FfmpegRequestIntent::Startup, false),
        (FfmpegRequestIntent::Startup, true),
        (FfmpegRequestIntent::Recheck, false),
        (FfmpegRequestIntent::Recheck, true),
    ] {
        let prior = selected("prior");
        let mut authority = FfmpegAuthority::<String>::new(prior.clone());
        authority.begin(prior_intent, prior.clone());
        let selection = authority.begin(FfmpegRequestIntent::Selection, selected("candidate"));
        let resolution = if save_failure {
            authority.finish_selection_persistence::<()>(selection, Err(()))
        } else {
            authority.retain_prior(selection)
        };
        assert_eq!(
            resolution,
            SelectionResolution::Retained(RollbackAction::Revalidate(prior.clone()))
        );
        assert_eq!(authority.preference(), &prior);
        assert_eq!(authority.status(), &AuthorityStatus::Checking);
        assert!(authority.toolchain().is_none());
    }
}

#[test]
fn one_picker_or_candidate_is_enforced_by_authority_state() {
    let mut authority = FfmpegAuthority::<String>::new(FfmpegLocationPreference::Auto);
    let picker_a = authority.begin_picker().expect("first picker starts");
    assert!(authority.picker_active());
    assert_eq!(authority.begin_picker(), None);
    assert!(authority.accept_picker(picker_a));

    let selection = authority.begin(FfmpegRequestIntent::Selection, selected("a"));
    assert_eq!(authority.begin_picker(), None);
    assert!(authority.is_current(selection));
}

#[test]
fn clear_and_recheck_invalidate_open_picker_results() {
    for intent in [
        FfmpegRequestIntent::ClearToAuto,
        FfmpegRequestIntent::Recheck,
    ] {
        let mut authority = FfmpegAuthority::<String>::new(FfmpegLocationPreference::Auto);
        let picker = authority.begin_picker().unwrap();
        authority.begin(intent, FfmpegLocationPreference::Auto);
        assert!(!authority.accept_picker(picker));
        assert!(!authority.picker_active());
    }
}

#[test]
fn picker_cancellation_consumes_only_picker_and_preserves_current_operation() {
    let mut authority = FfmpegAuthority::<String>::new(FfmpegLocationPreference::Auto);
    let recheck = authority.begin(FfmpegRequestIntent::Recheck, FfmpegLocationPreference::Auto);
    let picker = authority.begin_picker().unwrap();
    assert!(authority.accept_picker(picker));
    assert!(authority.is_current(recheck));
    assert_eq!(authority.status(), &AuthorityStatus::Checking);
}

#[test]
fn explicit_clear_intent_can_invalidate_picker_and_unpublished_selection() {
    let mut authority = ready_authority(FfmpegLocationPreference::Auto, "auto");
    let picker = authority.begin_picker().unwrap();
    authority.invalidate_picker();
    assert!(!authority.accept_picker(picker));

    let selection = authority.begin(FfmpegRequestIntent::Selection, selected("candidate"));
    assert!(authority.is_current(selection));
    assert_eq!(
        authority.abandon_selection(),
        Some(SelectionResolution::Retained(RollbackAction::Restored))
    );
    assert!(!authority.is_current(selection));
    assert_eq!(authority.toolchain().map(String::as_str), Some("auto"));
}
