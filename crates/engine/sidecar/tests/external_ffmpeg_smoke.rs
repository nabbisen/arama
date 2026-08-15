use std::{path::PathBuf, process::Stdio, time::Instant};

use arama_env::{Settings, ffmpeg_location::FfmpegLocationPreference};
use arama_sidecar::media::video::video_engine::discovery::{
    DiscoverySource, FfmpegDiscoveryEvent, FfmpegDiscoveryOutcome, FfmpegDiscoveryRuntime,
    FfmpegLocatorPolicy, PreferenceTransition, StdCandidateFilesystem, normalize_auto_candidates,
    publish_validated_selection,
};

#[test]
#[ignore = "owner-run native smoke; set ARAMA_FFMPEG_SMOKE_DIR to a trusted pair"]
fn selected_external_pair_generates_probes_and_extracts_real_video() {
    let directory = std::env::var_os("ARAMA_FFMPEG_SMOKE_DIR")
        .map(PathBuf::from)
        .expect("set ARAMA_FFMPEG_SMOKE_DIR to the directory containing ffmpeg and ffprobe");
    assert!(directory.is_absolute());

    let preference = FfmpegLocationPreference::SelectedDirectory(directory);
    let runtime = FfmpegDiscoveryRuntime::default();
    let ticket = runtime.request(preference.clone());
    let async_runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build smoke runtime");

    assert!(matches!(
        async_runtime.block_on(ticket.next()),
        Some(FfmpegDiscoveryEvent::Started(1))
    ));
    let Some(FfmpegDiscoveryEvent::SelectedReady { validated, .. }) =
        async_runtime.block_on(ticket.next())
    else {
        panic!("selected external pair did not validate");
    };

    let mut settings = Settings {
        ffmpeg_location: FfmpegLocationPreference::Auto,
        ..Settings::default()
    };
    let PreferenceTransition::PublishedReady { outcome, .. } = publish_validated_selection(
        &mut settings,
        &FfmpegLocationPreference::Auto,
        validated,
        |_| Ok::<(), ()>(()),
    ) else {
        panic!("validated pair did not publish");
    };
    let FfmpegDiscoveryOutcome::Ready { toolchain, .. } = outcome else {
        panic!("published selection did not carry toolchain authority");
    };

    let root = std::env::current_dir()
        .expect("current directory")
        .join("target/external-ffmpeg-smoke")
        .join(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("create smoke directory");
    let video = root.join("fixture.mp4");
    let frame = root.join("frame.png");

    let generated = toolchain
        .ffmpeg()
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=64x64:d=1",
            "-c:v",
            "mpeg4",
            "-y",
        ])
        .arg(&video)
        .stdin(Stdio::null())
        .status()
        .expect("run captured ffmpeg for fixture generation");
    assert!(generated.success());

    let probe = toolchain
        .ffprobe()
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(&video)
        .stdin(Stdio::null())
        .output()
        .expect("run captured ffprobe");
    assert!(probe.status.success());
    let duration: f64 = String::from_utf8(probe.stdout)
        .expect("duration is UTF-8")
        .trim()
        .parse()
        .expect("duration is numeric");
    assert!(duration > 0.0);

    let extracted = toolchain
        .ffmpeg()
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(&video)
        .args(["-frames:v", "1", "-y"])
        .arg(&frame)
        .stdin(Stdio::null())
        .status()
        .expect("run captured ffmpeg for frame extraction");
    assert!(extracted.success());
    assert!(std::fs::metadata(&frame).is_ok_and(|metadata| metadata.len() > 0));

    std::fs::remove_dir_all(root).expect("remove smoke directory");
}

/// RFC 038 Variant 2 — a second entry point rather than a parameterised one
/// (handoff §3.2), so a failure names which mode broke.
///
/// The workflow, not this test, is responsible for installing a real pair
/// and proving it is absent from `PATH` before this runs (handoff §5: "log
/// the effective PATH and assert ffmpeg/ffprobe are not resolvable on it" —
/// that check belongs to the shell step invoking `cargo test`, since this
/// integration-test crate has no visibility into `PATH` resolution
/// semantics beyond what the process it runs in already has).
///
/// This test's own job is narrower and platform-neutral: report exactly
/// where discovery found the pair (or that it found nothing), and hard-fail
/// only on the one universal claim — that a `Ready` outcome must not have
/// come from a bare `PATH` scan when the workflow went to the trouble of
/// stripping `PATH` first. A `Ready` outcome via `AutoPath` here would mean
/// the workflow's own precondition was not actually met, which proves
/// nothing about any fallback and must not be reported as a pass.
///
/// It deliberately does **not** require `Ready` universally: RFC 032's own
/// design table (`rfcs/done/032-cross-platform-external-ffmpeg.md`) gives
/// Windows no automatic off-`PATH` fallback at all — the Windows story for
/// an off-`PATH` pair is explicit Selected-directory mode (Variant 1,
/// above), not Auto discovery. `Missing` on Windows is therefore the
/// *correct*, designed outcome, not a defect this test should flag; only
/// the workflow's per-platform log interpretation (see `native-smoke.yaml`)
/// decides what a given outcome means for that runner.
#[test]
#[ignore = "owner-run native smoke; requires a valid ffmpeg/ffprobe pair installed but excluded from PATH"]
fn discovery_finds_a_pair_off_path() {
    let runtime = FfmpegDiscoveryRuntime::default();
    let ticket = runtime.request(FfmpegLocationPreference::Auto);
    let async_runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build smoke runtime");

    assert!(matches!(
        async_runtime.block_on(ticket.next()),
        Some(FfmpegDiscoveryEvent::Started(1))
    ));
    let Some(FfmpegDiscoveryEvent::Published(publication)) = async_runtime.block_on(ticket.next())
    else {
        panic!("auto discovery did not publish a terminal outcome");
    };

    // Printed (not just logged on failure): the workflow greps this line
    // regardless of whether the test itself passes, per handoff §5's "a
    // reviewer must be able to see, from the log alone" requirement. Run
    // with `--nocapture` or this is invisible on a passing test.
    match &publication.outcome {
        FfmpegDiscoveryOutcome::Ready { toolchain, source } => {
            println!("NATIVE_SMOKE_DISCOVERY_OUTCOME=Ready");
            println!("NATIVE_SMOKE_DISCOVERY_SOURCE={source:?}");
            println!(
                "NATIVE_SMOKE_DISCOVERY_PATH={}",
                toolchain.ffmpeg_path().display()
            );
            assert_ne!(
                *source,
                DiscoverySource::AutoPath,
                "resolved via a bare PATH scan despite the workflow excluding the \
                 pair from PATH beforehand - this proves nothing about any fallback \
                 and means the workflow's PATH-exclusion precondition was not met"
            );
        }
        other => {
            println!("NATIVE_SMOKE_DISCOVERY_OUTCOME={other:?}");
        }
    }
}

/// RFC 039 Phase 0 — blocking measurement, not a regression guard. Times the
/// filesystem-only collection phase (`normalize_auto_candidates`) over the
/// runner's real, unstripped `PATH`, with the raw-entry and candidate caps
/// raised far past anything realistic so nothing here is capped by the value
/// this RFC is about to change — the point is to measure the uncapped cost,
/// not to exercise the current bound.
///
/// This phase never spawns a subprocess (`normalize_auto_candidates` only
/// canonicalizes and stats directories), so its timing is the filesystem
/// half of the two-part cost `max_raw_path_entries` gates: 2 syscalls per
/// raw entry during collection, independent of whether a later validation
/// pass would find a match.
#[test]
#[ignore = "owner-run native smoke; RFC 039 Phase 0 timing, not a pass/fail check"]
fn discovery_collection_phase_timing_over_real_path() {
    let path = std::env::var_os("PATH");
    let raw_entry_count = path
        .as_deref()
        .map(|value| std::env::split_paths(value).count())
        .unwrap_or(0);

    let policy = FfmpegLocatorPolicy {
        max_raw_path_entries: 10_000,
        max_path_candidates: 10_000,
        ..FfmpegLocatorPolicy::default()
    };
    let mut control = || Ok(());
    let mut filesystem = StdCandidateFilesystem;

    let started = Instant::now();
    let result =
        normalize_auto_candidates(path.as_deref(), None, policy, &mut control, &mut filesystem);
    let elapsed = started.elapsed();

    println!("NATIVE_SMOKE_TIMING_RAW_ENTRY_COUNT={raw_entry_count}");
    println!(
        "NATIVE_SMOKE_TIMING_CANDIDATE_COUNT={}",
        result.candidates.len()
    );
    println!(
        "NATIVE_SMOKE_TIMING_REJECTED_ENTRIES={}",
        result.rejected_entries
    );
    println!("NATIVE_SMOKE_TIMING_RAW_TRUNCATED={}", result.raw_truncated);
    println!(
        "NATIVE_SMOKE_TIMING_CANDIDATE_TRUNCATED={}",
        result.candidate_truncated
    );
    println!(
        "NATIVE_SMOKE_TIMING_FILESYSTEM_DIAGNOSTIC={:?}",
        result.filesystem_diagnostic
    );
    // Per-entry pass, deliberately not sharing normalize_auto_candidates'
    // early-return-on-error path, so a single failing entry does not stop
    // enumeration of the rest - this is diagnostic-only, characterizing a
    // finding, not exercising the production candidate-collection code.
    if let Some(path) = path.as_deref() {
        for (index, entry) in std::env::split_paths(path).enumerate() {
            match std::fs::canonicalize(&entry) {
                Ok(canonical) => match std::fs::metadata(&canonical) {
                    Ok(_) => {}
                    Err(error) => {
                        println!(
                            "NATIVE_SMOKE_TIMING_ENTRY_METADATA_ERROR index={index} kind={:?} entry={}",
                            error.kind(),
                            entry.display()
                        );
                    }
                },
                Err(error) => {
                    println!(
                        "NATIVE_SMOKE_TIMING_ENTRY_CANONICALIZE_ERROR index={index} kind={:?} entry={}",
                        error.kind(),
                        entry.display()
                    );
                }
            }
        }
    }
    println!(
        "NATIVE_SMOKE_TIMING_COLLECTION_PHASE_MS={}",
        elapsed.as_secs_f64() * 1000.0
    );
    if raw_entry_count > 0 {
        println!(
            "NATIVE_SMOKE_TIMING_PER_RAW_ENTRY_US={}",
            elapsed.as_secs_f64() * 1_000_000.0 / raw_entry_count as f64
        );
    }
}

/// RFC 039 Phase 0 — the full-pipeline counterpart to the collection-phase
/// measurement above. Uses the same PATH-stripped precondition as
/// `discovery_finds_a_pair_off_path` (the workflow strips the installed
/// pair from PATH before this runs), so every candidate takes the cheap
/// "not found" path through validation (filesystem only, no subprocess) —
/// this measures the worst-case-for-timeout shape a raised cap would
/// produce: scanning every real raw PATH entry to a confident `Missing`
/// rather than truncating early, with `max_raw_path_entries` raised high
/// enough that the current default cannot mask the true entry count.
#[test]
#[ignore = "owner-run native smoke; RFC 039 Phase 0 timing, requires PATH stripped like discovery_finds_a_pair_off_path"]
fn discovery_full_attempt_timing_over_real_path() {
    let policy = FfmpegLocatorPolicy {
        max_raw_path_entries: 512,
        ..FfmpegLocatorPolicy::default()
    };
    let runtime = FfmpegDiscoveryRuntime::new(policy);
    let ticket = runtime.request(FfmpegLocationPreference::Auto);
    let async_runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build smoke runtime");

    let started = Instant::now();
    assert!(matches!(
        async_runtime.block_on(ticket.next()),
        Some(FfmpegDiscoveryEvent::Started(1))
    ));
    let Some(FfmpegDiscoveryEvent::Published(publication)) = async_runtime.block_on(ticket.next())
    else {
        panic!("auto discovery did not publish a terminal outcome");
    };
    let elapsed = started.elapsed();

    println!(
        "NATIVE_SMOKE_TIMING_FULL_ATTEMPT_MS={}",
        elapsed.as_secs_f64() * 1000.0
    );
    println!(
        "NATIVE_SMOKE_TIMING_FULL_ATTEMPT_OUTCOME={:?}",
        publication.outcome
    );
    if let FfmpegDiscoveryOutcome::Ready { source, .. } = &publication.outcome {
        assert_ne!(
            *source,
            DiscoverySource::AutoPath,
            "resolved via a bare PATH scan despite PATH being stripped - the \
             precondition this timing measurement depends on was not met"
        );
    }
}
