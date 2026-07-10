//! FFmpeg binary management: detect, download, unpack, and run.
//!
//! ## Download sources
//!
//! Linux and Windows binaries are downloaded from
//! [yt-dlp/FFmpeg-Builds](https://github.com/yt-dlp/FFmpeg-Builds) on
//! GitHub CDN — fast, well-maintained, statically-linked GPL builds.
//! The previous sources (johnvansickle.com on Linux, gyan.dev on Windows)
//! were personal servers with considerably lower throughput.
//!
//! macOS binaries continue to come from evermeet.cx (x86_64) and
//! osxexperts.net (aarch64), the same sources used industry-wide since
//! no GitHub-hosted static macOS builds are currently available.
//!
//! ## Archive layout (yt-dlp builds)
//!
//! ```text
//! ffmpeg-master-latest-<platform>-gpl[.tar.xz | .zip]
//!   └─ ffmpeg-master-latest-<platform>-gpl/
//!        └─ bin/
//!             ├─ ffmpeg[.exe]
//!             └─ ffprobe[.exe]
//! ```
//!
//! `unpack_archive` extracts to a temporary directory, moves the two
//! binaries to their final location, then cleans up.

use std::{fmt::Write as _, path::PathBuf, process::Command};

use anyhow::Context;
use arama_env::{local_bin_dir, validate_dir};
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

/// Temporary directory name used during archive extraction.
const UNPACK_DIRNAME: &str = "ffmpeg_release_temp";

#[derive(PartialEq)]
pub enum FfmpegStatus {
    ExistsInLocalBin,
    ExistsInPath,
    NotExists,
}

pub struct VideoEngine {}

#[derive(Clone, Debug)]
pub struct DownloadArtifact {
    pub url: &'static str,
    pub file_name: &'static str,
    pub expected_sha256: Option<&'static str>,
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

impl VideoEngine {
    pub fn ffmpeg_path() -> anyhow::Result<PathBuf> {
        Ok(local_bin_dir()?.join(bin_name::FFMPEG))
    }

    pub fn ffmpeg() -> Option<Command> {
        match Self::ready() {
            FfmpegStatus::ExistsInLocalBin => Self::ffmpeg_path().ok().map(Command::new),
            FfmpegStatus::ExistsInPath => Some(Command::new(bin_name::FFMPEG)),
            FfmpegStatus::NotExists => None,
        }
    }

    pub fn ffprobe() -> Option<Command> {
        match Self::ready() {
            FfmpegStatus::ExistsInLocalBin => Self::ffprobe_path().ok().map(Command::new),
            FfmpegStatus::ExistsInPath => Some(Command::new(bin_name::FFPROBE)),
            FfmpegStatus::NotExists => None,
        }
    }

    pub fn ready() -> FfmpegStatus {
        if Self::ffmpeg_path().is_ok_and(|path| path.exists()) {
            return FfmpegStatus::ExistsInLocalBin;
        }

        if Command::new(bin_name::FFMPEG)
            .args(["-version"])
            .output()
            .is_ok_and(|x| x.status.success())
        {
            return FfmpegStatus::ExistsInPath;
        }

        FfmpegStatus::NotExists
    }

    /// Download artifact for the current platform.
    ///
    /// Linux and Windows use yt-dlp/FFmpeg-Builds and carry SHA-256 values
    /// from the GitHub release asset metadata. macOS upstreams currently do
    /// not publish stable checksums for these moving URLs.
    pub fn download_artifact() -> anyhow::Result<DownloadArtifact> {
        if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            Ok(DownloadArtifact {
                url: "https://api.github.com/repos/yt-dlp/FFmpeg-Builds/releases/assets/470447732",
                file_name: "ffmpeg-master-latest-linux64-gpl.tar.xz",
                expected_sha256: Some(
                    "4aa9b01ab7d2a4a2fd86d243816b7b08d10336dc594b8ffb9555f8c54d28416c",
                ),
                github_api_asset: true,
            })
        } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
            Ok(DownloadArtifact {
                url: "https://api.github.com/repos/yt-dlp/FFmpeg-Builds/releases/assets/470447730",
                file_name: "ffmpeg-master-latest-linuxarm64-gpl.tar.xz",
                expected_sha256: Some(
                    "e8fc5de3a7562d3770175a20845dc1d0693994a689731165f624b87efa15cc14",
                ),
                github_api_asset: true,
            })
        } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
            Ok(DownloadArtifact {
                url: "https://api.github.com/repos/yt-dlp/FFmpeg-Builds/releases/assets/470447768",
                file_name: "ffmpeg-master-latest-win64-gpl.zip",
                expected_sha256: Some(
                    "d3baa79e66e77d095a1f7440008be49983656563576014e5b5c02728af5d3795",
                ),
                github_api_asset: true,
            })
        } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
            Ok(DownloadArtifact {
                url: "https://evermeet.cx/ffmpeg/getrelease/zip",
                file_name: "zip",
                expected_sha256: None,
                github_api_asset: false,
            })
        } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
            Ok(DownloadArtifact {
                url: "https://www.osxexperts.net/ffmpeg80arm.zip",
                file_name: "ffmpeg80arm.zip",
                expected_sha256: None,
                github_api_asset: false,
            })
        } else {
            anyhow::bail!(
                "Unsupported platform; please install ffmpeg manually and ensure it is on PATH"
            )
        }
    }

    /// Download URL for the current platform.
    pub fn download_url() -> anyhow::Result<String> {
        Ok(Self::download_artifact()?.url.to_owned())
    }

    pub fn download_dest_path() -> anyhow::Result<PathBuf> {
        let artifact = VideoEngine::download_artifact()?;
        let parent_dir = VideoEngine::parent_dir()?;
        Ok(parent_dir.join(artifact.file_name))
    }

    /// Extract the downloaded archive and place the ffmpeg (and ffprobe)
    /// binaries in [`VideoEngine::parent_dir`]. Cleans up the archive
    /// and temporary extraction directory on success.
    pub fn unpack_archive() -> anyhow::Result<()> {
        let from_archive = VideoEngine::download_dest_path()?;
        let binary_folder = VideoEngine::parent_dir()?;

        let temp_dir = binary_folder.join(UNPACK_DIRNAME);
        std::fs::create_dir_all(&temp_dir)?;

        // ── Extract ──────────────────────────────────────────────────────
        #[cfg(target_os = "linux")]
        {
            let file =
                std::fs::File::open(&from_archive).context("failed to open ffmpeg archive")?;
            let decompressed = xz2::read::XzDecoder::new(file);
            let mut archive = tar::Archive::new(decompressed);
            archive
                .unpack(&temp_dir)
                .context("failed to extract tar.xz")?;
        }

        #[cfg(not(target_os = "linux"))]
        {
            let file =
                std::fs::File::open(&from_archive).context("failed to open ffmpeg archive")?;
            let mut archive = zip::ZipArchive::new(file).context("failed to read zip archive")?;
            archive
                .extract(&temp_dir)
                .context("failed to extract zip")?;
        }

        // ── Locate binaries ──────────────────────────────────────────────
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        let (ffmpeg_src, ffprobe_src) = {
            let inner = std::fs::read_dir(&temp_dir)?
                .next()
                .context("extraction produced an empty directory")?
                .context("failed to read extraction directory")?
                .path();
            (
                inner.join("bin").join(bin_name::FFMPEG),
                inner.join("bin").join(bin_name::FFPROBE),
            )
        };

        #[cfg(target_os = "macos")]
        let (ffmpeg_src, ffprobe_src) = (
            temp_dir.join(bin_name::FFMPEG),
            temp_dir.join(bin_name::FFPROBE),
        );

        // ── Move to final location ───────────────────────────────────────
        std::fs::rename(&ffmpeg_src, binary_folder.join(bin_name::FFMPEG))
            .context("failed to move ffmpeg binary")?;

        if ffprobe_src.exists() {
            std::fs::rename(&ffprobe_src, binary_folder.join(bin_name::FFPROBE))
                .context("failed to move ffprobe binary")?;
        }

        // Set executable bit on Unix.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let dest = binary_folder.join(bin_name::FFMPEG);
            let mut perms = std::fs::metadata(&dest)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&dest, perms)?;
        }

        // ── Cleanup ──────────────────────────────────────────────────────
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir)?;
        }
        if from_archive.exists() {
            std::fs::remove_file(&from_archive)?;
        }

        Ok(())
    }

    /// Download and install the ffmpeg binary in one step.
    ///
    /// Fetches the archive from [`VideoEngine::download_url()`], saves it
    /// to [`VideoEngine::download_dest_path()`], then calls
    /// [`VideoEngine::unpack_archive()`]. The binary is buffered in memory
    /// during download (~80 MB); acceptable for a one-time re-installation
    /// from the Settings → AI page.
    pub async fn download_and_install() -> anyhow::Result<()> {
        let artifact = VideoEngine::download_artifact()?;
        let dest = VideoEngine::download_dest_path()?;

        let client = reqwest::Client::new();
        let response = artifact
            .get(&client)
            .send()
            .await
            .with_context(|| format!("failed to fetch {}", artifact.url))?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error {}: {}", response.status(), artifact.url);
        }

        let bytes = response
            .bytes()
            .await
            .context("failed to read ffmpeg download")?;

        verify_sha256(&bytes, artifact.expected_sha256)
            .context("ffmpeg checksum verification failed")?;

        tokio::fs::write(&dest, &bytes)
            .await
            .with_context(|| format!("failed to write {}", dest.display()))?;

        VideoEngine::unpack_archive().context("failed to unpack ffmpeg archive")?;
        Ok(())
    }

    fn parent_dir() -> anyhow::Result<PathBuf> {
        let path = VideoEngine::ffmpeg_path()?;

        let parent_dir = path.parent().context(format!(
            "failed to get parent dir of {} bin",
            bin_name::FFMPEG
        ))?;

        validate_dir(parent_dir)?;

        Ok(parent_dir.to_path_buf())
    }

    fn ffprobe_path() -> anyhow::Result<PathBuf> {
        Ok(local_bin_dir()?.join(bin_name::FFPROBE))
    }
}

fn verify_sha256(bytes: &[u8], expected_sha256: Option<&str>) -> anyhow::Result<()> {
    let Some(expected_sha256) = expected_sha256 else {
        return Ok(());
    };

    let actual = sha256_hex(bytes);
    if actual != expected_sha256 {
        anyhow::bail!("expected SHA-256 {expected_sha256}, got {actual}");
    }

    Ok(())
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
    use super::sha256_hex;

    #[test]
    fn sha256_hex_formats_digest() {
        assert_eq!(
            sha256_hex(b"arama"),
            "0d22554a4efcf5eb5aa3bef02fa51ce1a1c8ba77fe45d6d959148250c1211702"
        );
    }
}
