use std::{sync::Arc, time::Duration};

use arama_env::ffmpeg_location::FfmpegLocationPreference;

use super::{FfmpegDiscoveryEvent, FfmpegDiscoveryRuntime};
use crate::media::video::video_engine::discovery::{
    FfmpegDiscoveryOutcome, FfmpegLocatorPolicy, SearchLimit,
};

#[test]
fn actor_publishes_deadline_and_draining_before_blocked_worker_returns() {
    let runtime = FfmpegDiscoveryRuntime::new_with_worker(
        FfmpegLocatorPolicy {
            attempt_timeout: Duration::from_millis(5),
            ..FfmpegLocatorPolicy::default()
        },
        Arc::new(|_, _| {
            std::thread::sleep(Duration::from_millis(40));
            super::super::worker::WorkerCompletion::Outcome(FfmpegDiscoveryOutcome::Missing)
        }),
    );
    let first = runtime.request(FfmpegLocationPreference::Auto);
    assert!(matches!(
        first.next_blocking(),
        Some(FfmpegDiscoveryEvent::Started(1))
    ));
    let FfmpegDiscoveryEvent::Published(first_deadline) = first.next_blocking().unwrap() else {
        panic!("first request should publish its deadline")
    };
    assert_eq!(
        first_deadline.outcome,
        FfmpegDiscoveryOutcome::SearchLimitReached(SearchLimit::WholeAttempt)
    );

    let second = runtime.request(FfmpegLocationPreference::SelectedDirectory(
        std::env::temp_dir().join("selected"),
    ));
    let FfmpegDiscoveryEvent::Published(draining) = second.next_blocking().unwrap() else {
        panic!("queued request should publish draining")
    };
    assert_eq!(
        draining.outcome,
        FfmpegDiscoveryOutcome::SearchLimitReached(SearchLimit::WorkerDraining)
    );
    assert!(matches!(
        second.next_blocking(),
        Some(FfmpegDiscoveryEvent::Started(2))
    ));
}

#[test]
fn superseded_active_ticket_closes_when_latest_pending_starts() {
    let runtime = FfmpegDiscoveryRuntime::new_with_worker(
        FfmpegLocatorPolicy {
            attempt_timeout: Duration::from_secs(1),
            ..FfmpegLocatorPolicy::default()
        },
        Arc::new(|_, _| {
            std::thread::sleep(Duration::from_millis(10));
            super::super::worker::WorkerCompletion::Outcome(FfmpegDiscoveryOutcome::Missing)
        }),
    );
    let first = runtime.request(FfmpegLocationPreference::Auto);
    assert!(matches!(
        first.next_blocking(),
        Some(FfmpegDiscoveryEvent::Started(1))
    ));
    let second = runtime.request(FfmpegLocationPreference::Auto);

    assert_eq!(
        first.next_blocking(),
        Some(FfmpegDiscoveryEvent::Superseded)
    );
    assert!(matches!(
        second.next_blocking(),
        Some(FfmpegDiscoveryEvent::Started(2))
    ));
}

#[cfg(unix)]
#[test]
fn selected_symlink_executes_and_publishes_lexical_validated_authority() {
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
    };

    let root = std::env::current_dir()
        .unwrap()
        .join("target/runtime-selected-symlink")
        .join(std::process::id().to_string());
    let real = root.join("real");
    let selected = root.join("selected");
    fs::create_dir_all(&real).unwrap();
    for name in ["ffmpeg", "ffprobe"] {
        let executable = real.join(name);
        fs::write(
            &executable,
            format!("#!/bin/sh\nprintf '{name} version test-build\\n'\n"),
        )
        .unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    }
    symlink(&real, &selected).unwrap();
    let runtime = FfmpegDiscoveryRuntime::default();
    let ticket = runtime.request(FfmpegLocationPreference::SelectedDirectory(
        selected.clone(),
    ));
    assert!(matches!(
        ticket.next_blocking(),
        Some(FfmpegDiscoveryEvent::Started(1))
    ));
    let FfmpegDiscoveryEvent::SelectedReady {
        generation,
        validated,
    } = ticket.next_blocking().unwrap()
    else {
        panic!("selected validation should publish")
    };
    assert_eq!(generation, 1);
    let FfmpegDiscoveryOutcome::Ready { toolchain, .. } = validated.outcome() else {
        panic!("selected pair should be ready")
    };
    assert_eq!(toolchain.ffmpeg_path(), selected.join("ffmpeg"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unexpected_worker_completion_does_not_clear_active_bookkeeping() {
    let runtime = FfmpegDiscoveryRuntime::new_with_worker(
        FfmpegLocatorPolicy {
            attempt_timeout: Duration::from_secs(1),
            ..FfmpegLocatorPolicy::default()
        },
        Arc::new(|_, _| {
            std::thread::sleep(Duration::from_millis(20));
            super::super::worker::WorkerCompletion::Outcome(FfmpegDiscoveryOutcome::Missing)
        }),
    );
    let ticket = runtime.request(FfmpegLocationPreference::Auto);
    assert!(matches!(
        ticket.next_blocking(),
        Some(FfmpegDiscoveryEvent::Started(1))
    ));
    runtime
        .sender
        .send(super::ActorEvent::WorkerCompleted {
            generation: 999,
            completion: super::super::worker::WorkerCompletion::Outcome(
                FfmpegDiscoveryOutcome::Missing,
            ),
        })
        .unwrap();
    let FfmpegDiscoveryEvent::Published(publication) = ticket.next_blocking().unwrap() else {
        panic!("the real active completion must still publish")
    };
    assert_eq!(publication.generation, 1);
    assert_eq!(publication.outcome, FfmpegDiscoveryOutcome::Missing);
}
