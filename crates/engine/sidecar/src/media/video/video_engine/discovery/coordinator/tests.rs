use arama_env::ffmpeg_location::FfmpegLocationPreference;

use super::FfmpegDiscoveryCoordinator;
use crate::media::video::video_engine::discovery::{FfmpegDiscoveryOutcome, SearchLimit};

#[test]
fn one_active_and_latest_pending_are_serialized() {
    let mut coordinator = FfmpegDiscoveryCoordinator::default();
    let first = coordinator
        .request(FfmpegLocationPreference::Auto)
        .work
        .unwrap();
    let second = coordinator.request(FfmpegLocationPreference::Auto);
    assert!(first.is_cancelled());
    assert!(second.work.is_none());
    let third = coordinator.request(FfmpegLocationPreference::Auto);
    assert!(third.work.is_none());

    let drained = coordinator.worker_completed(first.generation, FfmpegDiscoveryOutcome::Missing);
    let newest = drained.work.expect("latest pending work should start");
    assert_eq!(newest.generation, 3);
    assert!(drained.publication.is_none());
}

#[test]
fn auto_or_selected_deadline_publishes_before_worker_drains() {
    for preference in [
        FfmpegLocationPreference::Auto,
        FfmpegLocationPreference::SelectedDirectory("/controlled/bin".into()),
    ] {
        let mut coordinator = FfmpegDiscoveryCoordinator::default();
        let work = coordinator.request(preference).work.unwrap();
        let publication = coordinator.deadline_elapsed(work.generation).unwrap();
        assert_eq!(
            publication.outcome,
            FfmpegDiscoveryOutcome::SearchLimitReached(SearchLimit::WholeAttempt)
        );
        assert!(work.is_cancelled());

        let queued = coordinator.request(FfmpegLocationPreference::Auto);
        assert_eq!(
            queued.publication.unwrap().outcome,
            FfmpegDiscoveryOutcome::SearchLimitReached(SearchLimit::WorkerDraining)
        );
        assert!(queued.work.is_none());
    }
}

#[test]
fn deadline_for_superseded_worker_publishes_latest_as_draining() {
    let mut coordinator = FfmpegDiscoveryCoordinator::default();
    let active = coordinator
        .request(FfmpegLocationPreference::Auto)
        .work
        .unwrap();
    coordinator.request(FfmpegLocationPreference::SelectedDirectory(
        "/controlled/bin".into(),
    ));

    let publication = coordinator.deadline_elapsed(active.generation).unwrap();

    assert_eq!(publication.generation, 2);
    assert_eq!(
        publication.outcome,
        FfmpegDiscoveryOutcome::SearchLimitReached(SearchLimit::WorkerDraining)
    );
}

#[test]
fn stale_completion_cannot_publish() {
    let mut coordinator = FfmpegDiscoveryCoordinator::default();
    let work = coordinator
        .request(FfmpegLocationPreference::Auto)
        .work
        .unwrap();
    coordinator.request(FfmpegLocationPreference::Auto);
    let completion = coordinator.worker_completed(work.generation, FfmpegDiscoveryOutcome::Missing);
    assert!(completion.publication.is_none());
    assert_eq!(completion.work.unwrap().generation, 2);
}

#[test]
fn latest_completion_publishes_without_starting_another_worker() {
    let mut coordinator = FfmpegDiscoveryCoordinator::default();
    let work = coordinator
        .request(FfmpegLocationPreference::Auto)
        .work
        .unwrap();

    let completion = coordinator.worker_completed(work.generation, FfmpegDiscoveryOutcome::Missing);

    assert!(completion.work.is_none());
    assert_eq!(
        completion.publication.unwrap().outcome,
        FfmpegDiscoveryOutcome::Missing
    );
    assert_eq!(coordinator.active_generation(), None);
}
