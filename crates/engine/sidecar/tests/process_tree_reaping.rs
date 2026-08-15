//! Task 021 — the last undischarged risk from
//! `rfcs/notes/native-smoke-risk-acceptance.md`: whether `command-group`'s
//! Windows job-object path reaps a probe's entire process tree on timeout,
//! the way its Unix process-group path is already known to (every other
//! test in this workspace exercises Unix).
//!
//! `run_bounded_probe_with_cancellation` (`video_engine.rs`) is `pub(super)`
//! and deliberately not widened for this test - per the task, that would be
//! a design decision, not an implementation detail. This test goes through
//! the public path instead: Selected-directory discovery
//! (`FfmpegDiscoveryRuntime::new(policy).request(SelectedDirectory(dir))`),
//! pointed at a hanging stub, so the real production probe/kill path is what
//! gets exercised end to end.
//!
//! The stub (`tests/fixtures/hang_stub.rs`) is not part of the Cargo
//! workspace - it is a standalone fixture the workflow compiles directly
//! with `rustc` before running this test, one binary reused for both the
//! `ffmpeg` and `ffprobe` names (only `ffmpeg`'s probe ever actually runs;
//! `validate_candidate` returns on its failure before `ffprobe` is probed,
//! so `ffprobe` only needs to exist as a file):
//!
//! ```sh
//! mkdir -p "$STUB_DIR"
//! rustc crates/engine/sidecar/tests/fixtures/hang_stub.rs -o "$STUB_DIR/ffmpeg"      # unix
//! cp "$STUB_DIR/ffmpeg" "$STUB_DIR/ffprobe"
//! # or, on Windows:
//! rustc crates/engine/sidecar/tests/fixtures/hang_stub.rs -o "$STUB_DIR/ffmpeg.exe"
//! cp "$STUB_DIR/ffmpeg.exe" "$STUB_DIR/ffprobe.exe"
//! ```
//!
//! Invoked as the probe target, the stub spawns a grandchild of itself (which
//! writes its own PID to `grandchild.pid` next to the stub, then hangs) and
//! then also hangs itself, without ever answering `-version` - forcing the
//! real `probe_timeout` path. The risk under test is specifically the
//! grandchild: `ProbeGroup::kill()` terminating only the direct child would
//! prove nothing that was in question.

use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};

use arama_env::ffmpeg_location::FfmpegLocationPreference;
use arama_sidecar::media::video::video_engine::discovery::{
    FfmpegDiscoveryEvent, FfmpegDiscoveryOutcome, FfmpegDiscoveryRuntime, FfmpegLocatorPolicy,
};

#[test]
#[ignore = "owner-run native smoke; requires ARAMA_HANG_STUB_DIR pointing at a compiled hang_stub.rs pair"]
fn process_tree_reaping_kills_grandchild_on_probe_timeout() {
    let directory = std::env::var_os("ARAMA_HANG_STUB_DIR")
        .map(PathBuf::from)
        .expect("set ARAMA_HANG_STUB_DIR to a directory containing the compiled hang-stub pair");
    assert!(directory.is_absolute());

    let marker = directory.join("grandchild.pid");
    let _ = std::fs::remove_file(&marker);

    // Short, test-local timeouts - not a change to the production default in
    // policy.rs, just what this call site asks for, the same pattern RFC
    // 039's Phase 0 timing tests used.
    let policy = FfmpegLocatorPolicy {
        probe_timeout: Duration::from_millis(500),
        attempt_timeout: Duration::from_secs(5),
        ..FfmpegLocatorPolicy::default()
    };
    let runtime = FfmpegDiscoveryRuntime::new(policy);
    let ticket = runtime.request(FfmpegLocationPreference::SelectedDirectory(
        directory.clone(),
    ));
    let async_runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build smoke runtime");

    assert!(matches!(
        async_runtime.block_on(ticket.next()),
        Some(FfmpegDiscoveryEvent::Started(1))
    ));

    // The grandchild must exist and be recorded *before* the probe times
    // out, or this test would prove nothing - a race where the grandchild
    // never actually started would let a broken reaping path "pass" by
    // accident.
    let grandchild_pid = wait_for_marker(&marker, Duration::from_millis(450))
        .expect("grandchild did not record its PID before the probe timeout - test setup is broken, not the thing under test");
    println!("NATIVE_SMOKE_REAPING_GRANDCHILD_PID={grandchild_pid}");
    assert!(
        process_is_alive(grandchild_pid),
        "grandchild {grandchild_pid} must be alive before the timeout for this test to prove anything"
    );
    println!("NATIVE_SMOKE_REAPING_GRANDCHILD_ALIVE_BEFORE_TIMEOUT=true");

    let Some(FfmpegDiscoveryEvent::Published(publication)) = async_runtime.block_on(ticket.next())
    else {
        panic!("selected-directory probe did not publish a terminal outcome");
    };
    println!(
        "NATIVE_SMOKE_REAPING_TERMINAL_OUTCOME={:?}",
        publication.outcome
    );
    assert_eq!(
        publication.outcome,
        FfmpegDiscoveryOutcome::ProbeTimedOut,
        "expected the hang stub to produce a probe timeout, not some other outcome"
    );

    // The terminal event fires after ProbeGroup::kill()+wait() return, but
    // OS-level process table cleanup can lag that by a beat on some
    // platforms - one short, bounded wait before asserting absence, not a
    // retry loop that would hide a slow-but-eventually-correct teardown.
    std::thread::sleep(Duration::from_millis(300));

    let alive_after = process_is_alive(grandchild_pid);
    println!("NATIVE_SMOKE_REAPING_GRANDCHILD_ALIVE_AFTER_TIMEOUT={alive_after}");
    assert!(
        !alive_after,
        "grandchild {grandchild_pid} survived the probe timeout - process-tree reaping did not reach it"
    );

    let _ = std::fs::remove_file(&marker);
}

fn wait_for_marker(marker: &Path, budget: Duration) -> Option<u32> {
    let deadline = Instant::now() + budget;
    while Instant::now() < deadline {
        if let Ok(text) = std::fs::read_to_string(marker)
            && let Ok(pid) = text.trim().parse::<u32>()
        {
            return Some(pid);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    None
}

#[cfg(unix)]
fn process_is_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(windows)]
fn process_is_alive(pid: u32) -> bool {
    let output = Command::new("tasklist")
        .args(["/fi", &format!("PID eq {pid}"), "/nh"])
        .output()
        .expect("run tasklist");
    String::from_utf8_lossy(&output.stdout).contains(&pid.to_string())
}
