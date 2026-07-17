//! Validated ffmpeg/ffprobe discovery and verified managed installation.
//!
//! Linux and Windows may install pinned archives from
//! [yt-dlp/FFmpeg-Builds](https://github.com/yt-dlp/FFmpeg-Builds). macOS
//! executable acquisition is user-managed: arama only discovers a compatible
//! pair on `PATH` or in the native Homebrew prefix.

pub mod discovery;

use std::{
    fmt::Write as _,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

#[cfg(windows)]
use std::sync::mpsc;

use anyhow::Context;
use arama_env::{local_bin_dir, validate_dir};
use command_group::CommandGroup;
use reqwest::header::{ACCEPT, USER_AGENT};
use sha2::{Digest, Sha256};

#[cfg(not(windows))]
mod bin_name {
    pub const FFMPEG: &str = "ffmpeg";
    pub const FFPROBE: &str = "ffprobe";
}
#[cfg(windows)]
mod bin_name {
    pub const FFMPEG: &str = "ffmpeg.exe";
    pub const FFPROBE: &str = "ffprobe.exe";
}

const PROBE_TIMEOUT: Duration = Duration::from_secs(2);
const PROBE_POLL_INTERVAL: Duration = Duration::from_millis(10);
const MAX_PROBE_OUTPUT: u64 = 64 * 1024;
const MAX_ARTIFACT_SIZE: usize = 256 * 1024 * 1024;
const MANAGED_PAIR_DIRNAME: &str = "ffmpeg-managed";
static INSTALL_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ToolchainSource {
    Managed,
    System,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FfmpegStatus {
    ExistsInLocalBin,
    ExistsInPath,
    NotExists,
}

/// Platform ownership policy for the ffmpeg/ffprobe pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FfmpegDistribution {
    /// arama may install its pinned, authenticated pair.
    Managed,
    /// The user supplies the pair outside arama; UI actions may only discover it.
    External,
}

impl FfmpegDistribution {
    pub const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::External
        } else {
            Self::Managed
        }
    }
}

/// A concrete ffmpeg/ffprobe pair that passed compatibility validation.
///
/// Paths are private so callers cannot construct an authority around an
/// unvalidated or mixed pair.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FfmpegToolchain {
    ffmpeg: PathBuf,
    ffprobe: PathBuf,
    source: ToolchainSource,
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

pub struct VideoEngine {}

#[derive(Clone, Debug)]
pub struct DownloadArtifact {
    pub url: &'static str,
    pub file_name: &'static str,
    pub expected_sha256: &'static str,
    pub github_api_asset: bool,
}

impl DownloadArtifact {
    pub fn get(&self, client: &reqwest::Client) -> reqwest::RequestBuilder {
        let request = client.get(self.url);
        if self.github_api_asset {
            request
                .header(ACCEPT, "application/octet-stream")
                .header(USER_AGENT, "arama")
        } else {
            request
        }
    }
}

#[derive(Clone, Copy)]
struct ProbePolicy {
    timeout: Duration,
    poll_interval: Duration,
}

trait VersionProbe {
    fn version_token(&self, executable: &Path, tool_name: &str) -> anyhow::Result<String>;
}

struct ProcessVersionProbe {
    policy: ProbePolicy,
}

impl VersionProbe for ProcessVersionProbe {
    fn version_token(&self, executable: &Path, tool_name: &str) -> anyhow::Result<String> {
        let output = run_bounded_probe(executable, self.policy)?;
        parse_version_token(&output, tool_name)
    }
}

impl VideoEngine {
    /// Discover and validate one concrete ffmpeg/ffprobe pair.
    ///
    /// This performs bounded child-process probes and must therefore be called
    /// from background work rather than an iced render method.
    pub fn toolchain() -> Option<FfmpegToolchain> {
        let probe = ProcessVersionProbe {
            policy: ProbePolicy {
                timeout: PROBE_TIMEOUT,
                poll_interval: PROBE_POLL_INTERVAL,
            },
        };

        let legacy_directory = if cfg!(target_os = "macos") {
            local_bin_dir().ok()
        } else {
            None
        };
        discover_candidates(candidate_directories(), legacy_directory.as_deref(), &probe)
    }

    /// Discover the validated pair on a blocking worker.
    ///
    /// UI callers should use this entry point so process probing never runs on
    /// an iced render or async executor thread.
    pub async fn discover_toolchain() -> Option<FfmpegToolchain> {
        tokio::task::spawn_blocking(Self::toolchain)
            .await
            .ok()
            .flatten()
    }

    pub fn ffmpeg() -> Option<Command> {
        Self::toolchain().map(|toolchain| toolchain.ffmpeg())
    }

    pub fn ffprobe() -> Option<Command> {
        Self::toolchain().map(|toolchain| toolchain.ffprobe())
    }

    pub fn ready() -> FfmpegStatus {
        match Self::toolchain().map(|toolchain| toolchain.source) {
            Some(ToolchainSource::Managed) => FfmpegStatus::ExistsInLocalBin,
            Some(ToolchainSource::System) => FfmpegStatus::ExistsInPath,
            None => FfmpegStatus::NotExists,
        }
    }

    /// Pinned, digest-authenticated artifact for managed installation.
    ///
    /// macOS intentionally has no artifact: users install a paired toolchain
    /// outside arama and discovery validates it before use.
    pub fn download_artifact() -> anyhow::Result<DownloadArtifact> {
        let [linux_x86_64, linux_aarch64, windows_x86_64] = supported_artifacts();
        if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            Ok(linux_x86_64)
        } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
            Ok(linux_aarch64)
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            Ok(windows_x86_64)
        } else if cfg!(target_os = "macos") {
            anyhow::bail!(
                "automatic ffmpeg download is unavailable on macOS; install ffmpeg and ffprobe (recommended: brew install ffmpeg), then re-check"
            )
        } else {
            anyhow::bail!(
                "unsupported platform; install ffmpeg and ffprobe and ensure both are on PATH"
            )
        }
    }

    /// Download, authenticate, extract, validate, and install a managed pair.
    ///
    /// Extraction is private to this operation, so no production caller can
    /// install a pre-positioned archive that bypassed digest verification.
    pub async fn download_and_install() -> anyhow::Result<()> {
        let artifact = Self::download_artifact()?;
        validate_digest_format(artifact.expected_sha256)?;

        let client = reqwest::Client::new();
        let response = artifact
            .get(&client)
            .send()
            .await
            .with_context(|| format!("failed to fetch {}", artifact.url))?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error {}: {}", response.status(), artifact.url);
        }

        let bytes = read_bounded_response(response).await?;
        verify_and_install_archive(&bytes, artifact.expected_sha256)
            .context("failed to install authenticated ffmpeg archive")
    }
}

async fn read_bounded_response(response: reqwest::Response) -> anyhow::Result<Vec<u8>> {
    read_bounded_response_with_limit(response, MAX_ARTIFACT_SIZE).await
}

async fn read_bounded_response_with_limit(
    mut response: reqwest::Response,
    limit: usize,
) -> anyhow::Result<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        anyhow::bail!("ffmpeg download exceeds the configured size limit");
    }

    let mut bytes = Vec::with_capacity(
        response
            .content_length()
            .unwrap_or_default()
            .min(limit as u64) as usize,
    );
    while let Some(chunk) = response
        .chunk()
        .await
        .context("failed to read ffmpeg download")?
    {
        extend_bounded(&mut bytes, &chunk, limit)?;
    }
    Ok(bytes)
}

fn extend_bounded(destination: &mut Vec<u8>, chunk: &[u8], limit: usize) -> anyhow::Result<()> {
    if destination.len().saturating_add(chunk.len()) > limit {
        anyhow::bail!("ffmpeg download exceeds the configured size limit");
    }
    destination
        .try_reserve(chunk.len())
        .context("failed to reserve memory for ffmpeg download")?;
    destination.extend_from_slice(chunk);
    Ok(())
}

fn supported_artifacts() -> [DownloadArtifact; 3] {
    [
        DownloadArtifact {
            url: "https://api.github.com/repos/yt-dlp/FFmpeg-Builds/releases/assets/470447732",
            file_name: "ffmpeg-master-latest-linux64-gpl.tar.xz",
            expected_sha256: "4aa9b01ab7d2a4a2fd86d243816b7b08d10336dc594b8ffb9555f8c54d28416c",
            github_api_asset: true,
        },
        DownloadArtifact {
            url: "https://api.github.com/repos/yt-dlp/FFmpeg-Builds/releases/assets/470447730",
            file_name: "ffmpeg-master-latest-linuxarm64-gpl.tar.xz",
            expected_sha256: "e8fc5de3a7562d3770175a20845dc1d0693994a689731165f624b87efa15cc14",
            github_api_asset: true,
        },
        DownloadArtifact {
            url: "https://api.github.com/repos/yt-dlp/FFmpeg-Builds/releases/assets/470447768",
            file_name: "ffmpeg-master-latest-win64-gpl.zip",
            expected_sha256: "d3baa79e66e77d095a1f7440008be49983656563576014e5b5c02728af5d3795",
            github_api_asset: true,
        },
    ]
}

fn candidate_directories() -> Vec<(PathBuf, ToolchainSource)> {
    let mut candidates = Vec::new();

    #[cfg(not(target_os = "macos"))]
    if let Ok(local) = local_bin_dir() {
        candidates.push((local.join(MANAGED_PAIR_DIRNAME), ToolchainSource::Managed));
        candidates.push((local, ToolchainSource::Managed));
    }

    if let Some(path) = std::env::var_os("PATH") {
        candidates.extend(
            std::env::split_paths(&path).map(|directory| (directory, ToolchainSource::System)),
        );
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    candidates.push((PathBuf::from("/opt/homebrew/bin"), ToolchainSource::System));

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    candidates.push((PathBuf::from("/usr/local/bin"), ToolchainSource::System));

    candidates
}

fn discover_candidates(
    candidates: Vec<(PathBuf, ToolchainSource)>,
    excluded_legacy_directory: Option<&Path>,
    probe: &dyn VersionProbe,
) -> Option<FfmpegToolchain> {
    candidates.into_iter().find_map(|candidate| {
        if excluded_legacy_directory.is_some_and(|legacy| directory_is_within(&candidate.0, legacy))
        {
            return None;
        }
        validate_candidate(candidate, probe)
    })
}

fn directory_is_within(candidate: &Path, excluded_root: &Path) -> bool {
    match (
        std::fs::canonicalize(candidate),
        std::fs::canonicalize(excluded_root),
    ) {
        (Ok(candidate), Ok(excluded_root)) => candidate.starts_with(excluded_root),
        _ => match (absolute_path(candidate), absolute_path(excluded_root)) {
            (Ok(candidate), Ok(excluded_root)) => candidate.starts_with(excluded_root),
            _ => false,
        },
    }
}

fn absolute_path(path: &Path) -> std::io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn validate_candidate(
    candidate: (PathBuf, ToolchainSource),
    probe: &dyn VersionProbe,
) -> Option<FfmpegToolchain> {
    let (directory, source) = candidate;
    let ffmpeg = directory.join(bin_name::FFMPEG);
    let ffprobe = directory.join(bin_name::FFPROBE);
    if !ffmpeg.is_file() || !ffprobe.is_file() {
        return None;
    }

    let ffmpeg_version = probe.version_token(&ffmpeg, "ffmpeg").ok()?;
    let ffprobe_version = probe.version_token(&ffprobe, "ffprobe").ok()?;
    if ffmpeg_version != ffprobe_version {
        return None;
    }

    Some(FfmpegToolchain {
        ffmpeg,
        ffprobe,
        source,
    })
}

fn run_bounded_probe(executable: &Path, policy: ProbePolicy) -> anyhow::Result<Vec<u8>> {
    run_bounded_probe_with_cancellation(executable, policy, None)
}

fn run_bounded_probe_with_cancellation(
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
    // SAFETY: fcntl is called with the live stdout descriptor owned by this
    // function and the documented F_GETFL/F_SETFL commands.
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
            // `stdout` is owned here and is dropped on return. An escaped
            // process retaining the write end therefore observes EPIPE rather
            // than extending discovery beyond the deadline.
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

fn parse_version_token(output: &[u8], tool_name: &str) -> anyhow::Result<String> {
    let output = std::str::from_utf8(output).context("version output was not UTF-8")?;
    let first_line = output.lines().next().context("version output was empty")?;
    let prefix = format!("{tool_name} version ");
    let token = first_line
        .strip_prefix(&prefix)
        .and_then(|rest| rest.split_whitespace().next())
        .filter(|token| !token.is_empty())
        .context("version output had an unexpected format")?;
    Ok(token.to_owned())
}

fn validate_digest_format(digest: &str) -> anyhow::Result<()> {
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        anyhow::bail!("expected SHA-256 must be 64 lowercase hexadecimal characters");
    }
    Ok(())
}

fn verify_sha256(bytes: &[u8], expected_sha256: &str) -> anyhow::Result<()> {
    validate_digest_format(expected_sha256)?;
    let actual = sha256_hex(bytes);
    if actual != expected_sha256 {
        anyhow::bail!("expected SHA-256 {expected_sha256}, got {actual}");
    }
    Ok(())
}

fn verify_and_install_archive(bytes: &[u8], expected_sha256: &str) -> anyhow::Result<()> {
    let binary_folder = managed_binary_folder()?;
    verify_and_install_archive_into(bytes, expected_sha256, &binary_folder)
}

fn verify_and_install_archive_into(
    bytes: &[u8],
    expected_sha256: &str,
    binary_folder: &Path,
) -> anyhow::Result<()> {
    verify_sha256(bytes, expected_sha256).context("ffmpeg checksum verification failed")?;
    install_verified_archive(bytes, binary_folder)
        .context("failed to install verified ffmpeg archive")
}

fn install_verified_archive(bytes: &[u8], binary_folder: &Path) -> anyhow::Result<()> {
    let id = INSTALL_ID.fetch_add(1, Ordering::Relaxed);
    let staging = binary_folder.join(format!(".ffmpeg-install-{}-{id}", std::process::id()));
    std::fs::create_dir(&staging).context("failed to create ffmpeg staging directory")?;

    let result = install_from_staging(bytes, binary_folder, &staging, id);
    if staging.exists() {
        let _ = std::fs::remove_dir_all(&staging);
    }
    result
}

fn install_from_staging(
    bytes: &[u8],
    binary_folder: &Path,
    staging: &Path,
    id: u64,
) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    {
        let decompressed = xz2::read::XzDecoder::new(bytes);
        let mut archive = tar::Archive::new(decompressed);
        archive
            .unpack(staging)
            .context("failed to extract verified tar.xz")?;
    }

    #[cfg(not(target_os = "linux"))]
    {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
            .context("failed to read verified zip")?;
        archive
            .extract(staging)
            .context("failed to extract verified zip")?;
    }

    let inner = std::fs::read_dir(staging)?
        .next()
        .context("extraction produced an empty directory")?
        .context("failed to read extraction directory")?
        .path();
    let staged_ffmpeg = inner.join("bin").join(bin_name::FFMPEG);
    let staged_ffprobe = inner.join("bin").join(bin_name::FFPROBE);
    if !staged_ffmpeg.is_file() || !staged_ffprobe.is_file() {
        anyhow::bail!("verified archive did not contain a complete ffmpeg/ffprobe pair");
    }

    #[cfg(unix)]
    for executable in [&staged_ffmpeg, &staged_ffprobe] {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(executable)?.permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(executable, permissions)?;
    }

    let probe = ProcessVersionProbe {
        policy: ProbePolicy {
            timeout: PROBE_TIMEOUT,
            poll_interval: PROBE_POLL_INTERVAL,
        },
    };
    validate_candidate((inner.join("bin"), ToolchainSource::Managed), &probe)
        .context("verified archive contained an incompatible tool pair")?;

    activate_pair_directory(binary_folder, &inner.join("bin"), id, &StdPairFilesystem)
}

trait PairFilesystem {
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    fn remove_dir_all(&self, path: &Path) -> std::io::Result<()>;
    fn create_dir(&self, path: &Path) -> std::io::Result<()>;
    fn copy_file(&self, from: &Path, to: &Path) -> std::io::Result<()>;
}

struct StdPairFilesystem;

impl PairFilesystem for StdPairFilesystem {
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        std::fs::rename(from, to)
    }

    fn remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
        std::fs::remove_dir_all(path)
    }

    fn create_dir(&self, path: &Path) -> std::io::Result<()> {
        std::fs::create_dir(path)
    }

    fn copy_file(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        std::fs::copy(from, to).map(|_| ())
    }
}

fn activate_pair_directory(
    binary_folder: &Path,
    staged_pair: &Path,
    id: u64,
    filesystem: &dyn PairFilesystem,
) -> anyhow::Result<()> {
    let next = binary_folder.join(format!(".ffmpeg-next-{id}"));
    let active = binary_folder.join(MANAGED_PAIR_DIRNAME);
    let backup = binary_folder.join(format!(".ffmpeg-backup-{id}"));
    let recovery = binary_folder.join(format!(".ffmpeg-recovery-{id}"));

    filesystem
        .rename(staged_pair, &next)
        .context("failed to stage complete ffmpeg pair for activation")?;

    let had_active = active.exists();
    if had_active && let Err(error) = filesystem.rename(&active, &backup) {
        let cleanup = filesystem.remove_dir_all(&next).err();
        return Err(anyhow::anyhow!(format_install_error(
            "failed to preserve active ffmpeg pair",
            &error,
            cleanup.as_ref(),
        )));
    }

    if let Err(error) = filesystem.rename(&next, &active) {
        let mut rollback_error = None;
        if had_active && let Err(first_restore_error) = filesystem.rename(&backup, &active) {
            match publish_recovery_copy(filesystem, &backup, &recovery, &active) {
                Ok(()) => {
                    rollback_error = filesystem.remove_dir_all(&backup).err();
                }
                Err(copy_error) => {
                    rollback_error = Some(std::io::Error::other(format!(
                        "backup rename failed: {first_restore_error}; atomic copy recovery failed: {copy_error}"
                    )));
                    // One final rename retry can recover a transient failure
                    // without ever publishing a partial active directory.
                    if filesystem.rename(&backup, &active).is_ok() {
                        rollback_error = Some(std::io::Error::other(format!(
                            "atomic copy recovery failed: {copy_error}; backup rename retry restored the active pair"
                        )));
                    }
                }
            }
        }
        let _ = filesystem.remove_dir_all(&next);
        return Err(anyhow::anyhow!(format_install_error(
            "failed to activate complete ffmpeg pair",
            &error,
            rollback_error.as_ref(),
        )));
    }

    if had_active && let Err(error) = filesystem.remove_dir_all(&backup) {
        return Err(error).context(format!(
            "new ffmpeg pair is active, but old pair remains at {}",
            backup.display()
        ));
    }
    Ok(())
}

fn publish_recovery_copy(
    filesystem: &dyn PairFilesystem,
    backup: &Path,
    recovery: &Path,
    active: &Path,
) -> std::io::Result<()> {
    filesystem.create_dir(recovery)?;
    for name in [bin_name::FFMPEG, bin_name::FFPROBE] {
        if let Err(copy_error) = filesystem.copy_file(&backup.join(name), &recovery.join(name)) {
            let cleanup_error = filesystem.remove_dir_all(recovery).err();
            return Err(std::io::Error::other(format_install_error(
                "failed to copy complete recovery pair",
                &copy_error,
                cleanup_error.as_ref(),
            )));
        }
    }
    if !recovery.join(bin_name::FFMPEG).is_file() || !recovery.join(bin_name::FFPROBE).is_file() {
        let validation_error = std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "recovery pair is incomplete",
        );
        let cleanup_error = filesystem.remove_dir_all(recovery).err();
        return Err(std::io::Error::other(format_install_error(
            "failed to validate recovery pair",
            &validation_error,
            cleanup_error.as_ref(),
        )));
    }
    if let Err(rename_error) = filesystem.rename(recovery, active) {
        let cleanup_error = filesystem.remove_dir_all(recovery).err();
        return Err(std::io::Error::other(format_install_error(
            "failed to publish complete recovery pair",
            &rename_error,
            cleanup_error.as_ref(),
        )));
    }
    Ok(())
}

fn format_install_error(
    operation: &str,
    primary: &std::io::Error,
    recovery: Option<&std::io::Error>,
) -> String {
    match recovery {
        Some(recovery) => format!("{operation}: {primary}; recovery also failed: {recovery}"),
        None => format!("{operation}: {primary}"),
    }
}

fn managed_binary_folder() -> anyhow::Result<PathBuf> {
    let directory = local_bin_dir()?;
    validate_dir(&directory)?;
    Ok(directory)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut hex, "{byte:02x}").expect("writing to string cannot fail");
    }
    hex
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, collections::HashMap, fs, io::Write, net::TcpListener};

    use super::*;

    fn test_root(name: &str) -> PathBuf {
        std::env::current_dir()
            .unwrap()
            .join("target")
            .join("sidecar-test-temp")
            .join(format!(
                "{name}-{}-{}",
                std::process::id(),
                INSTALL_ID.fetch_add(1, Ordering::Relaxed)
            ))
    }

    struct FakeProbe {
        tokens: HashMap<PathBuf, anyhow::Result<String>>,
    }

    impl VersionProbe for FakeProbe {
        fn version_token(&self, executable: &Path, _tool_name: &str) -> anyhow::Result<String> {
            match self.tokens.get(executable) {
                Some(Ok(token)) => Ok(token.clone()),
                Some(Err(error)) => anyhow::bail!(error.to_string()),
                None => anyhow::bail!("unexpected executable"),
            }
        }
    }

    fn candidate_fixture(ffmpeg_version: &str, ffprobe_version: &str) -> (PathBuf, FakeProbe) {
        let root = test_root("toolchain");
        fs::create_dir_all(&root).unwrap();
        let ffmpeg = root.join(bin_name::FFMPEG);
        let ffprobe = root.join(bin_name::FFPROBE);
        fs::write(&ffmpeg, b"fixture").unwrap();
        fs::write(&ffprobe, b"fixture").unwrap();
        let tokens = HashMap::from([
            (ffmpeg, Ok(ffmpeg_version.to_owned())),
            (ffprobe, Ok(ffprobe_version.to_owned())),
        ]);
        (root, FakeProbe { tokens })
    }

    fn write_pair(directory: &Path, marker: &[u8]) {
        fs::create_dir_all(directory).unwrap();
        fs::write(directory.join(bin_name::FFMPEG), marker).unwrap();
        fs::write(directory.join(bin_name::FFPROBE), marker).unwrap();
    }

    fn assert_pair(directory: &Path, marker: &[u8]) {
        assert_eq!(fs::read(directory.join(bin_name::FFMPEG)).unwrap(), marker);
        assert_eq!(fs::read(directory.join(bin_name::FFPROBE)).unwrap(), marker);
    }

    fn serve_once(response: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 512];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = stream.read(&mut buffer).unwrap();
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
            }
            stream.write_all(response).unwrap();
            stream.flush().unwrap();
        });
        format!("http://{address}")
    }

    fn test_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    struct FaultRenameFilesystem {
        rename_calls: Cell<usize>,
        fail_calls: Vec<usize>,
        copy_calls: Cell<usize>,
        fail_copy_calls: Vec<usize>,
        remove_calls: Cell<usize>,
        fail_remove_calls: Vec<usize>,
    }

    impl FaultRenameFilesystem {
        fn new(fail_calls: Vec<usize>) -> Self {
            Self {
                rename_calls: Cell::new(0),
                fail_calls,
                copy_calls: Cell::new(0),
                fail_copy_calls: Vec::new(),
                remove_calls: Cell::new(0),
                fail_remove_calls: Vec::new(),
            }
        }

        fn with_copy_failures(fail_calls: Vec<usize>, fail_copy_calls: Vec<usize>) -> Self {
            Self {
                rename_calls: Cell::new(0),
                fail_calls,
                copy_calls: Cell::new(0),
                fail_copy_calls,
                remove_calls: Cell::new(0),
                fail_remove_calls: Vec::new(),
            }
        }

        fn with_failures(
            fail_calls: Vec<usize>,
            fail_copy_calls: Vec<usize>,
            fail_remove_calls: Vec<usize>,
        ) -> Self {
            Self {
                rename_calls: Cell::new(0),
                fail_calls,
                copy_calls: Cell::new(0),
                fail_copy_calls,
                remove_calls: Cell::new(0),
                fail_remove_calls,
            }
        }
    }

    impl PairFilesystem for FaultRenameFilesystem {
        fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
            let call = self.rename_calls.get() + 1;
            self.rename_calls.set(call);
            if self.fail_calls.contains(&call) {
                return Err(std::io::Error::other(format!(
                    "injected rename failure {call}"
                )));
            }
            fs::rename(from, to)
        }

        fn remove_dir_all(&self, path: &Path) -> std::io::Result<()> {
            let call = self.remove_calls.get() + 1;
            self.remove_calls.set(call);
            if self.fail_remove_calls.contains(&call) {
                return Err(std::io::Error::other(format!(
                    "injected remove failure {call}"
                )));
            }
            fs::remove_dir_all(path)
        }

        fn create_dir(&self, path: &Path) -> std::io::Result<()> {
            fs::create_dir(path)
        }

        fn copy_file(&self, from: &Path, to: &Path) -> std::io::Result<()> {
            let call = self.copy_calls.get() + 1;
            self.copy_calls.set(call);
            if self.fail_copy_calls.contains(&call) {
                return Err(std::io::Error::other(format!(
                    "injected copy failure {call}"
                )));
            }
            fs::copy(from, to).map(|_| ())
        }
    }

    #[test]
    fn matching_pair_produces_authority() {
        let (root, probe) = candidate_fixture("8.1", "8.1");
        let toolchain = validate_candidate((root.clone(), ToolchainSource::System), &probe)
            .expect("matching pair should validate");
        assert_eq!(toolchain.ffmpeg_path(), root.join(bin_name::FFMPEG));
        assert_eq!(toolchain.ffprobe_path(), root.join(bin_name::FFPROBE));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn mismatched_pair_is_rejected() {
        let (root, probe) = candidate_fixture("8.1", "8.0");
        assert!(validate_candidate((root.clone(), ToolchainSource::System), &probe).is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn excluded_legacy_candidate_is_skipped_in_favor_of_valid_system_pair() {
        let (legacy, mut probe) = candidate_fixture("8.1", "8.1");
        let (system, system_probe) = candidate_fixture("8.1", "8.1");
        probe.tokens.extend(system_probe.tokens);

        let toolchain = discover_candidates(
            vec![
                (legacy.clone(), ToolchainSource::System),
                (system.clone(), ToolchainSource::System),
            ],
            Some(&legacy),
            &probe,
        )
        .expect("non-legacy candidate should remain available");

        assert_eq!(toolchain.ffmpeg_path(), system.join(bin_name::FFMPEG));
        fs::remove_dir_all(legacy).unwrap();
        fs::remove_dir_all(system).unwrap();
    }

    #[test]
    fn legacy_only_discovery_fails_closed() {
        let (legacy, probe) = candidate_fixture("8.1", "8.1");
        assert!(
            discover_candidates(
                vec![(legacy.clone(), ToolchainSource::System)],
                Some(&legacy),
                &probe,
            )
            .is_none()
        );
        fs::remove_dir_all(legacy).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn legacy_directory_alias_is_also_excluded() {
        use std::os::unix::fs::symlink;

        let (legacy, mut probe) = candidate_fixture("8.1", "8.1");
        let alias = legacy.with_extension("alias");
        symlink(&legacy, &alias).unwrap();
        probe
            .tokens
            .insert(alias.join(bin_name::FFMPEG), Ok("8.1".to_owned()));
        probe
            .tokens
            .insert(alias.join(bin_name::FFPROBE), Ok("8.1".to_owned()));
        assert!(
            discover_candidates(
                vec![(alias.clone(), ToolchainSource::System)],
                Some(&legacy),
                &probe,
            )
            .is_none()
        );
        fs::remove_file(alias).unwrap();
        fs::remove_dir_all(legacy).unwrap();
    }

    #[test]
    fn version_parser_requires_exact_tool_prefix() {
        assert_eq!(
            parse_version_token(b"ffmpeg version 8.1 Copyright\n", "ffmpeg").unwrap(),
            "8.1"
        );
        assert!(parse_version_token(b"ffprobe version 8.1\n", "ffmpeg").is_err());
        assert!(parse_version_token(&[0xff], "ffmpeg").is_err());
    }

    #[test]
    fn digest_is_required_and_lowercase_hex() {
        assert!(validate_digest_format(&"a".repeat(64)).is_ok());
        assert!(validate_digest_format(&"A".repeat(64)).is_err());
        assert!(validate_digest_format("abc").is_err());
        assert!(verify_sha256(b"arama", &"0".repeat(64)).is_err());
    }

    #[test]
    fn digest_mismatch_cannot_reach_extraction_or_replace_existing_pair() {
        let root = test_root("install-auth");
        fs::create_dir_all(&root).unwrap();
        let ffmpeg = root.join(bin_name::FFMPEG);
        let ffprobe = root.join(bin_name::FFPROBE);
        fs::write(&ffmpeg, b"existing ffmpeg").unwrap();
        fs::write(&ffprobe, b"existing ffprobe").unwrap();

        let result = verify_and_install_archive_into(b"not an archive", &"0".repeat(64), &root);

        assert!(result.is_err());
        assert_eq!(fs::read(&ffmpeg).unwrap(), b"existing ffmpeg");
        assert_eq!(fs::read(&ffprobe).unwrap(), b"existing ffprobe");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn authenticated_archive_installs_a_valid_complete_pair() {
        let root = test_root("install-happy");
        fs::create_dir_all(&root).unwrap();
        let encoder = xz2::write::XzEncoder::new(Vec::new(), 6);
        let mut archive = tar::Builder::new(encoder);
        for (name, contents) in [
            (
                bin_name::FFMPEG,
                b"#!/bin/sh\nprintf 'ffmpeg version 8.1\\n'\n".as_slice(),
            ),
            (
                bin_name::FFPROBE,
                b"#!/bin/sh\nprintf 'ffprobe version 8.1\\n'\n".as_slice(),
            ),
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            archive
                .append_data(&mut header, format!("fixture/bin/{name}"), contents)
                .unwrap();
        }
        archive.finish().unwrap();
        let encoder = archive.into_inner().unwrap();
        let bytes = encoder.finish().unwrap();
        let digest = sha256_hex(&bytes);

        verify_and_install_archive_into(&bytes, &digest, &root).unwrap();

        let active = root.join(MANAGED_PAIR_DIRNAME);
        assert!(active.join(bin_name::FFMPEG).is_file());
        assert!(active.join(bin_name::FFPROBE).is_file());
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn directory_activation_switches_complete_pair() {
        let root = test_root("activate-success");
        fs::create_dir_all(&root).unwrap();
        let active = root.join(MANAGED_PAIR_DIRNAME);
        let staged = root.join("staged");
        write_pair(&active, b"old");
        write_pair(&staged, b"new");

        activate_pair_directory(&root, &staged, 1, &FaultRenameFilesystem::new(vec![])).unwrap();

        assert_pair(&active, b"new");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn backup_failure_preserves_complete_old_pair() {
        let root = test_root("activate-backup-failure");
        fs::create_dir_all(&root).unwrap();
        let active = root.join(MANAGED_PAIR_DIRNAME);
        let staged = root.join("staged");
        write_pair(&active, b"old");
        write_pair(&staged, b"new");

        let result =
            activate_pair_directory(&root, &staged, 2, &FaultRenameFilesystem::new(vec![2]));

        assert!(result.is_err());
        assert_pair(&active, b"old");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn activation_failure_rolls_back_complete_old_pair() {
        let root = test_root("activate-failure");
        fs::create_dir_all(&root).unwrap();
        let active = root.join(MANAGED_PAIR_DIRNAME);
        let staged = root.join("staged");
        write_pair(&active, b"old");
        write_pair(&staged, b"new");

        let result =
            activate_pair_directory(&root, &staged, 3, &FaultRenameFilesystem::new(vec![3]));

        assert!(result.is_err());
        assert_pair(&active, b"old");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rollback_rename_failure_uses_complete_pair_copy_recovery() {
        let root = test_root("activate-rollback-failure");
        fs::create_dir_all(&root).unwrap();
        let active = root.join(MANAGED_PAIR_DIRNAME);
        let staged = root.join("staged");
        write_pair(&active, b"old");
        write_pair(&staged, b"new");

        let result =
            activate_pair_directory(&root, &staged, 4, &FaultRenameFilesystem::new(vec![3, 4]));

        assert!(result.is_err());
        assert_pair(&active, b"old");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn first_copy_failure_never_publishes_partial_active_pair() {
        assert_copy_failure_recovers_old_pair(1, "activate-copy-first-failure");
    }

    #[test]
    fn second_copy_failure_never_publishes_partial_active_pair() {
        assert_copy_failure_recovers_old_pair(2, "activate-copy-second-failure");
    }

    #[test]
    fn recovery_cleanup_failure_is_reported_without_partial_active_pair() {
        let root = test_root("activate-recovery-cleanup-failure");
        fs::create_dir_all(&root).unwrap();
        let active = root.join(MANAGED_PAIR_DIRNAME);
        let staged = root.join("staged");
        write_pair(&active, b"old");
        write_pair(&staged, b"new");

        let result = activate_pair_directory(
            &root,
            &staged,
            8,
            &FaultRenameFilesystem::with_failures(vec![3, 4], vec![1], vec![1]),
        )
        .unwrap_err();

        assert!(format!("{result:#}").contains("recovery also failed"));
        assert_pair(&active, b"old");
        fs::remove_dir_all(root).unwrap();
    }

    fn assert_copy_failure_recovers_old_pair(copy_call: usize, fixture_name: &str) {
        let root = test_root(fixture_name);
        fs::create_dir_all(&root).unwrap();
        let active = root.join(MANAGED_PAIR_DIRNAME);
        let staged = root.join("staged");
        write_pair(&active, b"old");
        write_pair(&staged, b"new");

        let result = activate_pair_directory(
            &root,
            &staged,
            5 + copy_call as u64,
            &FaultRenameFilesystem::with_copy_failures(vec![3, 4], vec![copy_call]),
        );

        assert!(result.is_err());
        assert_pair(&active, b"old");
        assert!(
            !root
                .join(format!(".ffmpeg-recovery-{}", 5 + copy_call as u64))
                .exists()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bounded_buffer_rejects_oversized_stream_without_appending() {
        let mut destination = vec![1, 2, 3];
        assert!(extend_bounded(&mut destination, &[4, 5], 4).is_err());
        assert_eq!(destination, vec![1, 2, 3]);
    }

    #[test]
    fn response_rejects_oversized_declared_content_length() {
        test_runtime().block_on(async {
            let url = serve_once(
                b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\nabcde",
            );
            let response = reqwest::get(url).await.unwrap();
            assert!(read_bounded_response_with_limit(response, 4).await.is_err());
        });
    }

    #[test]
    fn response_rejects_oversized_chunked_body_without_declared_length() {
        test_runtime().block_on(async {
            let url = serve_once(
                b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n3\r\nabc\r\n3\r\ndef\r\n0\r\n\r\n",
            );
            let response = reqwest::get(url).await.unwrap();
            assert!(read_bounded_response_with_limit(response, 4).await.is_err());
        });
    }

    #[test]
    fn every_automatic_artifact_has_a_valid_digest() {
        for artifact in supported_artifacts() {
            validate_digest_format(artifact.expected_sha256).unwrap();
        }
    }

    #[test]
    fn sha256_hex_formats_digest() {
        assert_eq!(
            sha256_hex(b"arama"),
            "0d22554a4efcf5eb5aa3bef02fa51ce1a1c8ba77fe45d6d959148250c1211702"
        );
    }

    #[cfg(unix)]
    #[test]
    fn process_probe_times_out_without_waiting_for_child_completion() {
        use std::os::unix::fs::PermissionsExt;

        let root = test_root("probe-timeout");
        fs::create_dir_all(&root).unwrap();
        let executable = root.join("slow-probe");
        fs::write(&executable, b"#!/bin/sh\nsleep 2\n").unwrap();
        let mut permissions = fs::metadata(&executable).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).unwrap();

        let started = Instant::now();
        let result = run_bounded_probe(
            &executable,
            ProbePolicy {
                timeout: Duration::from_millis(30),
                poll_interval: Duration::from_millis(5),
            },
        );
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(1));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn process_probe_rejects_output_over_limit_before_timeout() {
        use std::os::unix::fs::PermissionsExt;

        let root = test_root("probe-output-limit");
        fs::create_dir_all(&root).unwrap();
        let executable = root.join("loud-probe");
        fs::write(
            &executable,
            b"#!/bin/sh\nwhile :; do printf '0123456789abcdef'; done\n",
        )
        .unwrap();
        let mut permissions = fs::metadata(&executable).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).unwrap();

        let started = Instant::now();
        let result = run_bounded_probe(
            &executable,
            ProbePolicy {
                timeout: Duration::from_secs(2),
                poll_interval: Duration::from_millis(5),
            },
        );
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(1));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn process_probe_terminates_successful_parents_writer_descendant() {
        use std::os::unix::fs::PermissionsExt;

        let root = test_root("probe-descendant");
        fs::create_dir_all(&root).unwrap();
        let executable = root.join("descendant-probe");
        let marker = root.join("descendant-output");
        fs::write(
            &executable,
            format!(
                "#!/bin/sh\n( exec 3>&1; while :; do printf x >> '{}'; done ) &\nprintf 'ffmpeg version 8.1\\n'\nexit 0\n",
                marker.display()
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(&executable).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).unwrap();

        let result = run_bounded_probe(
            &executable,
            ProbePolicy {
                timeout: Duration::from_millis(50),
                poll_interval: Duration::from_millis(5),
            },
        );
        assert_eq!(result.unwrap(), b"ffmpeg version 8.1\n");
        let size_after_return = fs::metadata(&marker).map(|m| m.len()).unwrap_or_default();
        std::thread::sleep(Duration::from_millis(50));
        let size_later = fs::metadata(&marker).map(|m| m.len()).unwrap_or_default();
        assert_eq!(size_after_return, size_later);
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn process_probe_deadline_closes_pipe_held_by_escaped_session() {
        use std::os::unix::fs::PermissionsExt;

        let root = test_root("probe-escaped-session");
        fs::create_dir_all(&root).unwrap();
        let executable = root.join("escaped-session-probe");
        let marker = root.join("pipe-closed");
        fs::write(
            &executable,
            format!(
                "#!/bin/sh\nsetsid /bin/sh -c \"trap '' PIPE; while printf x; do sleep 0.01; done; printf closed > '{}'\" &\nprintf 'ffmpeg version 8.1\\n'\nexit 0\n",
                marker.display()
            ),
        )
        .unwrap();
        let mut permissions = fs::metadata(&executable).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).unwrap();

        let started = Instant::now();
        let result = run_bounded_probe(
            &executable,
            ProbePolicy {
                timeout: Duration::from_millis(75),
                poll_interval: Duration::from_millis(5),
            },
        );
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(1));

        let marker_deadline = Instant::now() + Duration::from_millis(500);
        while fs::read(&marker).ok().as_deref() != Some(b"closed")
            && Instant::now() < marker_deadline
        {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(fs::read(&marker).unwrap(), b"closed");
        fs::remove_dir_all(root).unwrap();
    }
}
