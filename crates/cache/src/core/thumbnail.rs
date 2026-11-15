//! Thumbnail generation.
//!
//! - **Images**: resized with the `image` crate (224×224 JPEG, Lanczos3).
//! - **Videos**: a frame extracted with the `ffmpeg` command at the
//!   5-second mark, falling back to 0 seconds on failure.
//!
//! Thumbnails are named by a BLAKE3 hash of the source file's canonical
//! path: `<thumbnail_dir>/<hash16>.jpg`. (v1 named them by database row
//! id; row ids no longer exist under `localcache`.)

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use command_group::CommandGroup;

use super::engine::CacheError;

/// RFC 047 S4 / the external audit's B3: ffmpeg previously ran with no
/// timeout and no output cap at all - a hung or adversarial process
/// would block this call, and everything awaiting it, forever. Chosen
/// generously: a 224x224 single-frame extract from a local file is
/// ordinarily sub-second; this bounds the pathological case without
/// second-guessing a slow disk or codec on an otherwise-valid file.
const FFMPEG_TIMEOUT: Duration = Duration::from_secs(30);
const FFMPEG_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Standard CLIP model input size (px).
const THUMBNAIL_SIZE: u32 = 224;

/// Number of hex characters of the path hash used in the file name.
const THUMBNAIL_NAME_LEN: usize = 16;

// ---------------------------------------------------------------------------
// Image thumbnails
// ---------------------------------------------------------------------------

/// Generate a thumbnail from an image file and save it to `dest`.
///
/// - Format: JPEG
/// - Size: `THUMBNAIL_SIZE` × `THUMBNAIL_SIZE` (224 × 224)
/// - Resize filter: Lanczos3
pub(crate) fn generate_image_thumbnail(src: &Path, dest: &Path) -> Result<(), CacheError> {
    ensure_parent(dest)?;

    let img = image::open(src).map_err(|e| CacheError::ThumbnailGenerationFailed(e.to_string()))?;

    let thumb = img.resize(
        THUMBNAIL_SIZE,
        THUMBNAIL_SIZE,
        image::imageops::FilterType::Lanczos3,
    );

    thumb
        .save(dest)
        .map_err(|e| CacheError::ThumbnailGenerationFailed(e.to_string()))?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Video thumbnails
// ---------------------------------------------------------------------------

/// Generate a thumbnail from a video file and save it to `dest`.
///
/// Tries the frame at the 5-second mark; falls back to 0 seconds.
/// Returns `Err(CacheError::ThumbnailGenerationFailed)` when both fail.
pub(crate) fn generate_video_thumbnail(
    src: &Path,
    dest: &Path,
    ffmpeg_path: &Path,
) -> Result<(), CacheError> {
    ensure_parent(dest)?;

    // Try the 5-second mark first.
    if run_ffmpeg(ffmpeg_path, src, dest, "00:00:05").is_ok() && dest.exists() {
        return Ok(());
    }

    // Fall back to 0 seconds.
    if run_ffmpeg(ffmpeg_path, src, dest, "00:00:00").is_ok() && dest.exists() {
        return Ok(());
    }

    Err(CacheError::ThumbnailGenerationFailed(format!(
        "ffmpeg failed to extract frame from {}",
        src.display()
    )))
}

fn run_ffmpeg(
    ffmpeg_path: &Path,
    src: &Path,
    dest: &Path,
    timestamp: &str,
) -> Result<(), CacheError> {
    let mut command = Command::new(ffmpeg_path);
    command.args([
        "-ss",
        timestamp,
        "-i",
        src.to_str().unwrap_or(""),
        "-vframes",
        "1",
        "-vf",
        &format!("scale={THUMBNAIL_SIZE}:{THUMBNAIL_SIZE}:force_original_aspect_ratio=decrease"),
        "-y", // overwrite an existing file
        dest.to_str().unwrap_or(""),
    ]);

    run_command_with_timeout(command, FFMPEG_TIMEOUT).map_err(|err| {
        CacheError::ThumbnailGenerationFailed(match err {
            CommandTimeoutError::Spawn(message) => message,
            CommandTimeoutError::Failed(stderr) => stderr,
            CommandTimeoutError::TimedOut { timeout, stderr } => format!(
                "ffmpeg did not finish extracting a frame from {} within {}s and was killed; stderr: {stderr}",
                src.display(),
                timeout.as_secs()
            ),
        })
    })
}

#[derive(Debug)]
enum CommandTimeoutError {
    /// The process could not even be spawned (e.g. the binary vanished
    /// between discovery and use).
    Spawn(String),
    /// It ran to completion with a non-zero exit status.
    Failed(String),
    /// It did not exit within `timeout` and was killed.
    TimedOut { timeout: Duration, stderr: String },
}

/// Run `command` to completion, killing it if it has not exited after
/// `timeout`. `std::process::Command` has no built-in timeout - this is
/// the general mechanism `run_ffmpeg` uses it through, kept generic (and
/// so independently testable without a real media file or binary) rather
/// than private to ffmpeg specifically.
fn run_command_with_timeout(
    mut command: Command,
    timeout: Duration,
) -> std::result::Result<(), CommandTimeoutError> {
    // Review 161 #4: `group_spawn` (not `spawn`) so `kill()` below
    // terminates the whole process group/job object, not only this one
    // process - the same mechanism the sidecar already uses for Task
    // 021's Windows process-tree reaping
    // (`crates/engine/sidecar/src/media/video/video_engine.rs`). A
    // launcher shim (a Chocolatey or scoop ffmpeg wrapper, say) whose
    // real work happens in a child of the process we spawned would
    // otherwise be orphaned by a plain `child.kill()`.
    let mut child = command
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .group_spawn()
        .map_err(|e| CommandTimeoutError::Spawn(e.to_string()))?;

    // `Command::output()` drains stdout/stderr on background threads
    // while it waits, precisely so a chatty child cannot deadlock on a
    // full pipe. A manual poll loop (needed below for the timeout) does
    // not get that for free, so stderr is drained on its own thread here
    // for the same reason.
    let stderr_handle = child.inner().stderr.take().map(|mut pipe| {
        std::thread::spawn(move || {
            use std::io::Read;
            let mut buf = String::new();
            let _ = pipe.read_to_string(&mut buf);
            buf
        })
    });

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if Instant::now() >= deadline {
                    // Best-effort: either call failing just means the
                    // process was already gone by the time we got here.
                    let _ = child.kill();
                    let _ = child.wait();
                    break None;
                }
                std::thread::sleep(FFMPEG_POLL_INTERVAL.min(timeout));
            }
            Err(_) => break None,
        }
    };

    let stderr = stderr_handle
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();

    match status {
        Some(status) if status.success() => Ok(()),
        Some(_) => Err(CommandTimeoutError::Failed(stderr)),
        None => Err(CommandTimeoutError::TimedOut { timeout, stderr }),
    }
}

// ---------------------------------------------------------------------------
// Shared utilities
// ---------------------------------------------------------------------------

/// Thumbnail destination for a source file:
/// `<thumbnail_dir>/<blake3(canonical_path)[..16]>.jpg`.
///
/// The source file must exist (its path is canonicalized to make the name
/// stable across relative/absolute spellings).
pub(crate) fn thumbnail_dest(thumbnail_dir: &Path, src: &Path) -> Result<PathBuf, CacheError> {
    let canonical = src.canonicalize().map_err(|e| CacheError::io(src, e))?;
    Ok(thumbnail_dest_for_canonical(
        thumbnail_dir,
        &canonical.to_string_lossy(),
    ))
}

/// Same as [`thumbnail_dest`] for an already-canonicalized path string.
/// Used by the v1 migration, where the source file's existence has
/// already been verified.
pub(crate) fn thumbnail_dest_for_canonical(thumbnail_dir: &Path, canonical: &str) -> PathBuf {
    let hash = blake3::hash(canonical.as_bytes());
    let hex = hash.to_hex();
    thumbnail_dir.join(format!("{}.jpg", &hex.as_str()[..THUMBNAIL_NAME_LEN]))
}

/// Create the parent directory of `path` when missing.
fn ensure_parent(path: &Path) -> Result<(), CacheError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| CacheError::io(parent, e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 047 S4 / audit B3: the central property - a process that
    /// never exits on its own is killed within bounds, not awaited
    /// forever. No real ffmpeg or media file involved: the timeout
    /// mechanism is generic over any `Command` (`run_ffmpeg` is one
    /// caller of it, not the thing under test here).
    #[cfg(unix)]
    #[test]
    fn a_process_that_never_exits_is_killed_within_the_timeout() {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 30"]);

        let started = std::time::Instant::now();
        let result = run_command_with_timeout(command, Duration::from_millis(300));
        let elapsed = started.elapsed();

        assert!(
            matches!(result, Err(CommandTimeoutError::TimedOut { .. })),
            "expected TimedOut, got a different outcome"
        );
        assert!(
            elapsed < Duration::from_secs(5),
            "killing a hung process took {elapsed:?}, expected well under the 30s it would \
             otherwise have run for"
        );
    }

    /// Review 161 #4: the point of `group_spawn` over plain `spawn` -
    /// killing the direct child must not orphan a grandchild it spawned
    /// itself, the shape of a launcher shim (a Chocolatey or scoop
    /// `ffmpeg` wrapper) whose real work happens one process down.
    /// Proven without a PID-existence check (no `libc` dependency
    /// needed): the grandchild repeatedly appends one line to a heartbeat
    /// file while alive; if it is still running after the parent is
    /// killed, the file keeps growing, and if it is dead, it stops -
    /// checked by comparing the file's length at two reads a beat apart,
    /// after the kill.
    ///
    /// Review 162: the first version of this test compared `date +%s%N`
    /// timestamps instead of a length. BSD `date` (macOS) has no `%N`, so
    /// the stamp there only changes once a second - two reads 250 ms
    /// apart would usually land in the same second and match **whether
    /// or not the grandchild was actually still alive**, making the
    /// assertion vacuous on that platform. Appending a line per beat and
    /// comparing length instead needs nothing from `date` at all, so it
    /// means the same thing on every `sh`.
    #[cfg(unix)]
    #[test]
    fn killing_the_group_also_reaps_a_grandchild_it_spawned() {
        let heartbeat = std::env::temp_dir().join(format!(
            "arama-t061-rework-grandchild-heartbeat-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&heartbeat);

        let mut command = Command::new("sh");
        command.args([
            "-c",
            &format!("(while true; do echo x >> {heartbeat:?}; sleep 0.05; done) & wait",),
        ]);

        // Long enough that the grandchild has definitely written at
        // least once before the timeout fires.
        let result = run_command_with_timeout(command, Duration::from_millis(200));
        assert!(
            matches!(result, Err(CommandTimeoutError::TimedOut { .. })),
            "expected TimedOut, got {result:?}"
        );
        assert!(
            heartbeat.exists(),
            "the grandchild never got to write even once - the fixture itself is broken, \
             not the property under test"
        );

        let first_len = std::fs::metadata(&heartbeat).unwrap().len();
        std::thread::sleep(Duration::from_millis(250));
        let second_len = std::fs::metadata(&heartbeat).unwrap().len();

        let _ = std::fs::remove_file(&heartbeat);

        assert_eq!(
            first_len, second_len,
            "the heartbeat file kept growing after the parent was killed - the grandchild is \
             still alive, orphaned rather than reaped"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_process_that_exits_zero_well_within_the_timeout_succeeds() {
        let command = Command::new("true");

        let result = run_command_with_timeout(command, Duration::from_secs(5));

        assert!(result.is_ok(), "expected success, got {result:?}");
    }

    #[cfg(unix)]
    #[test]
    fn a_process_that_exits_nonzero_well_within_the_timeout_fails_without_a_timeout_error() {
        let command = Command::new("false");

        let result = run_command_with_timeout(command, Duration::from_secs(5));

        assert!(
            matches!(result, Err(CommandTimeoutError::Failed(_))),
            "expected Failed (a clean non-zero exit, not a timeout), got {result:?}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn a_process_that_never_exits_is_killed_within_the_timeout() {
        let mut command = Command::new("cmd");
        command.args(["/C", "ping -n 60 127.0.0.1 >NUL"]);

        let started = std::time::Instant::now();
        let result = run_command_with_timeout(command, Duration::from_millis(300));
        let elapsed = started.elapsed();

        assert!(
            matches!(result, Err(CommandTimeoutError::TimedOut { .. })),
            "expected TimedOut, got a different outcome"
        );
        assert!(
            elapsed < Duration::from_secs(10),
            "killing a hung process took {elapsed:?}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn a_process_that_exits_zero_well_within_the_timeout_succeeds() {
        let mut command = Command::new("cmd");
        command.args(["/C", "exit 0"]);

        let result = run_command_with_timeout(command, Duration::from_secs(5));

        assert!(result.is_ok(), "expected success, got {result:?}");
    }

    #[cfg(windows)]
    #[test]
    fn a_process_that_exits_nonzero_well_within_the_timeout_fails_without_a_timeout_error() {
        let mut command = Command::new("cmd");
        command.args(["/C", "exit 1"]);

        let result = run_command_with_timeout(command, Duration::from_secs(5));

        assert!(
            matches!(result, Err(CommandTimeoutError::Failed(_))),
            "expected Failed (a clean non-zero exit, not a timeout), got {result:?}"
        );
    }

    /// Not platform-gated: proves the stderr-draining thread does not
    /// itself deadlock a chatty child against a full pipe, which a
    /// manual poll loop does not get for free the way `Command::output`
    /// does - this is the specific failure mode that motivated draining
    /// stderr on its own thread rather than reading it after the fact.
    #[cfg(unix)]
    #[test]
    fn a_chatty_process_does_not_deadlock_the_poll_loop() {
        // A pipe buffer is typically 64 KiB; this writes well past that
        // before exiting, so a reader that isn't actively draining would
        // leave the child blocked on `write()` and `try_wait` spinning
        // forever on `None`.
        let mut command = Command::new("sh");
        command.args(["-c", "yes 'filling the pipe' | head -c 200000 1>&2; exit 0"]);

        let started = std::time::Instant::now();
        let result = run_command_with_timeout(command, Duration::from_secs(5));
        let elapsed = started.elapsed();

        assert!(result.is_ok(), "expected success, got {result:?}");
        assert!(
            elapsed < Duration::from_secs(5),
            "a drained chatty child should finish almost immediately, took {elapsed:?}"
        );
    }
}
