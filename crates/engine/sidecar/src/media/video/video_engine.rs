//! Validated external `ffmpeg` / `ffprobe` authority and bounded probing.
//!
//! arama never downloads or installs these executables. Discovery validates a
//! user-managed compatible pair before producing [`FfmpegToolchain`].

pub mod discovery;

use std::{
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

#[cfg(windows)]
use std::sync::mpsc;

use anyhow::Context;
use command_group::CommandGroup;

#[cfg(not(windows))]
pub(super) mod bin_name {
    pub const FFMPEG: &str = "ffmpeg";
    pub const FFPROBE: &str = "ffprobe";
}
#[cfg(windows)]
pub(super) mod bin_name {
    pub const FFMPEG: &str = "ffmpeg.exe";
    pub const FFPROBE: &str = "ffprobe.exe";
}

pub(super) const PROBE_POLL_INTERVAL: Duration = Duration::from_millis(10);
const MAX_PROBE_OUTPUT: u64 = 64 * 1024;

/// A concrete pair that passed compatibility validation.
///
/// Paths are private so callers cannot manufacture authority around an
/// unvalidated or mixed pair. Long-running work clones this value once at task
/// creation and does not rediscover executables mid-task.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfmpegToolchain {
    pub(super) ffmpeg: PathBuf,
    pub(super) ffprobe: PathBuf,
}

impl FfmpegToolchain {
    pub fn ffmpeg(&self) -> Command {
        Command::new(&self.ffmpeg)
    }

    pub fn ffprobe(&self) -> Command {
        Command::new(&self.ffprobe)
    }

    pub fn ffmpeg_path(&self) -> &Path {
        &self.ffmpeg
    }

    pub fn ffprobe_path(&self) -> &Path {
        &self.ffprobe
    }
}

#[derive(Clone, Copy)]
pub(super) struct ProbePolicy {
    pub timeout: Duration,
    pub poll_interval: Duration,
}

pub(super) fn run_bounded_probe_with_cancellation(
    executable: &Path,
    policy: ProbePolicy,
    cancellation: Option<&std::sync::atomic::AtomicBool>,
) -> anyhow::Result<Vec<u8>> {
    let mut command = Command::new(executable);
    command
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let child = command
        .group_spawn()
        .with_context(|| format!("failed to start {}", executable.display()))?;
    let mut child = ProbeGroup { child };
    let stdout = child
        .inner()
        .stdout
        .take()
        .context("failed to capture version output")?;

    #[cfg(unix)]
    return run_bounded_probe_unix(executable, policy, child, stdout, cancellation);

    #[cfg(windows)]
    return run_bounded_probe_windows(executable, policy, child, stdout, cancellation);
}

#[cfg(unix)]
fn run_bounded_probe_unix(
    executable: &Path,
    policy: ProbePolicy,
    mut child: ProbeGroup,
    mut stdout: std::process::ChildStdout,
    cancellation: Option<&std::sync::atomic::AtomicBool>,
) -> anyhow::Result<Vec<u8>> {
    use std::os::fd::AsRawFd;

    let descriptor = stdout.as_raw_fd();
    // SAFETY: the documented fcntl commands operate on the live descriptor
    // owned by this function.
    let flags = unsafe { libc::fcntl(descriptor, libc::F_GETFL) };
    if flags == -1
        || unsafe { libc::fcntl(descriptor, libc::F_SETFL, flags | libc::O_NONBLOCK) } == -1
    {
        anyhow::bail!("failed to make version-output pipe nonblocking");
    }

    let deadline = Instant::now() + policy.timeout;
    let mut output = Vec::new();
    let mut status = None;
    let mut output_closed = false;
    let mut group_terminated = false;
    let mut buffer = [0_u8; 4096];

    loop {
        if cancellation.is_some_and(|token| token.load(Ordering::Acquire)) {
            terminate_probe_group(&mut child);
            anyhow::bail!("version probe cancelled");
        }
        loop {
            match stdout.read(&mut buffer) {
                Ok(0) => {
                    output_closed = true;
                    break;
                }
                Ok(read) => {
                    if output.len().saturating_add(read) > MAX_PROBE_OUTPUT as usize {
                        terminate_probe_group(&mut child);
                        anyhow::bail!("version output exceeded limit for {}", executable.display());
                    }
                    output.extend_from_slice(&buffer[..read]);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error).context("failed to read version output"),
            }
        }

        if status.is_none() {
            status = child.try_wait().context("failed to poll version probe")?;
        }
        if status.is_some() && !group_terminated {
            let _ = child.kill();
            let _ = child.wait();
            group_terminated = true;
        }
        if output_closed && let Some(status) = status {
            if !status.success() {
                anyhow::bail!("version probe failed for {}", executable.display());
            }
            return Ok(output);
        }
        if Instant::now() >= deadline {
            terminate_probe_group(&mut child);
            anyhow::bail!("version probe timed out for {}", executable.display());
        }
        std::thread::sleep(policy.poll_interval);
    }
}

#[cfg(windows)]
fn run_bounded_probe_windows(
    executable: &Path,
    policy: ProbePolicy,
    mut child: ProbeGroup,
    stdout: std::process::ChildStdout,
    cancellation: Option<&std::sync::atomic::AtomicBool>,
) -> anyhow::Result<Vec<u8>> {
    let (output_sender, output_receiver) = mpsc::sync_channel(1);
    let reader = std::thread::spawn(move || {
        let mut output = Vec::new();
        let result = stdout
            .take(MAX_PROBE_OUTPUT + 1)
            .read_to_end(&mut output)
            .map(|_| output);
        let _ = output_sender.send(result);
    });

    let deadline = Instant::now() + policy.timeout;
    let mut captured_output = None;
    let mut status = None;
    loop {
        if cancellation.is_some_and(|token| token.load(Ordering::Acquire)) {
            terminate_probe_group(&mut child);
            anyhow::bail!("version probe cancelled");
        }
        if captured_output.is_none()
            && let Ok(result) = output_receiver.try_recv()
        {
            let output = result.context("failed to read version output")?;
            if output.len() as u64 > MAX_PROBE_OUTPUT {
                terminate_probe_group(&mut child);
                let _ = reader.join();
                anyhow::bail!("version output exceeded limit for {}", executable.display());
            }
            captured_output = Some(output);
        }
        if status.is_none() {
            status = child.try_wait().context("failed to poll version probe")?;
        }
        if status.is_some() && captured_output.is_some() {
            break;
        }
        if Instant::now() >= deadline {
            terminate_probe_group(&mut child);
            let _ = output_receiver.recv_timeout(Duration::from_millis(100));
            if reader.is_finished() {
                let _ = reader.join();
            }
            anyhow::bail!("version probe timed out for {}", executable.display());
        }
        std::thread::sleep(policy.poll_interval);
    }

    terminate_probe_group(&mut child);
    let output = captured_output.context("version-output reader stopped unexpectedly")?;
    let _ = reader.join();
    let status = status.context("version probe exited without a status")?;
    if !status.success() {
        anyhow::bail!("version probe failed for {}", executable.display());
    }
    if output.len() as u64 > MAX_PROBE_OUTPUT {
        anyhow::bail!("version output exceeded limit for {}", executable.display());
    }
    Ok(output)
}

fn terminate_probe_group(child: &mut ProbeGroup) {
    let _ = child.kill();
    let _ = child.wait();
}

struct ProbeGroup {
    child: command_group::GroupChild,
}

impl ProbeGroup {
    fn inner(&mut self) -> &mut std::process::Child {
        self.child.inner()
    }

    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }

    fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill()
    }

    fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.child.wait()
    }
}

impl Drop for ProbeGroup {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub(super) fn parse_version_token(output: &[u8], tool_name: &str) -> anyhow::Result<String> {
    let output = std::str::from_utf8(output).context("version output was not UTF-8")?;
    let first_line = output.lines().next().context("version output was empty")?;
    let prefix = format!("{tool_name} version ");
    first_line
        .strip_prefix(&prefix)
        .and_then(|rest| rest.split_whitespace().next())
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .context("version output had an unexpected format")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_parser_requires_exact_tool_prefix() {
        assert_eq!(
            parse_version_token(b"ffmpeg version 8.1-static\n", "ffmpeg").unwrap(),
            "8.1-static"
        );
        assert!(parse_version_token(b"wrapper ffmpeg version 8.1\n", "ffmpeg").is_err());
        assert!(parse_version_token(b"ffprobe version 8.1\n", "ffmpeg").is_err());
    }

    #[cfg(unix)]
    fn probe_fixture(name: &str, script: &[u8]) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::current_dir()
            .unwrap()
            .join("target/video-engine-probes")
            .join(format!("{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let executable = root.join("probe");
        std::fs::write(&executable, script).unwrap();
        let mut permissions = std::fs::metadata(&executable).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&executable, permissions).unwrap();
        executable
    }

    #[cfg(unix)]
    #[test]
    fn process_probe_times_out_without_waiting_for_child_completion() {
        let executable = probe_fixture("timeout", b"#!/bin/sh\nsleep 2\n");
        let started = Instant::now();
        let result = run_bounded_probe_with_cancellation(
            &executable,
            ProbePolicy {
                timeout: Duration::from_millis(30),
                poll_interval: Duration::from_millis(5),
            },
            None,
        );
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(1));
        std::fs::remove_dir_all(executable.parent().unwrap()).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn process_probe_rejects_output_over_limit_before_timeout() {
        let executable = probe_fixture(
            "output-limit",
            b"#!/bin/sh\nwhile :; do printf '0123456789abcdef'; done\n",
        );
        let started = Instant::now();
        let result = run_bounded_probe_with_cancellation(
            &executable,
            ProbePolicy {
                timeout: Duration::from_secs(2),
                poll_interval: Duration::from_millis(5),
            },
            None,
        );
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(1));
        std::fs::remove_dir_all(executable.parent().unwrap()).unwrap();
    }
}
