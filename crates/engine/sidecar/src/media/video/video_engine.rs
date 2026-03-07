use std::{path::PathBuf, process::Command};

use arama_env::{local_bin_dir, validate_dir};
use ffmpeg_sidecar::download::{check_latest_version, download_ffmpeg_package, unpack_ffmpeg};

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

#[derive(PartialEq)]
pub enum FfmpegStatus {
    ExistsInLocalBin,
    ExistsInPath,
    NotExists,
}

pub struct VideoEngine {}

impl VideoEngine {
    pub fn ffmpeg_path() -> anyhow::Result<PathBuf> {
        Ok(local_bin_dir()?.join(bin_name::FFMPEG))
    }

    pub fn ffmpeg() -> Option<Command> {
        match Self::ready() {
            FfmpegStatus::ExistsInLocalBin => Some(Command::new(
                Self::ffmpeg_path()
                    .expect(&format!("failed to get {} in local bin", bin_name::FFMPEG)),
            )),
            FfmpegStatus::ExistsInPath => Some(Command::new(bin_name::FFMPEG)),
            FfmpegStatus::NotExists => None,
        }
    }

    pub fn ffprobe() -> Option<Command> {
        match Self::ready() {
            FfmpegStatus::ExistsInLocalBin => Some(Command::new(
                Self::ffprobe_path()
                    .expect(&format!("failed to get {} in local bin", bin_name::FFPROBE)),
            )),
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

    pub fn download() -> anyhow::Result<()> {
        let path = VideoEngine::ffmpeg_path()?;

        // todo: skip or delete existing file ?
        // if path.exists() {
        //     return Ok(());
        // }

        validate_dir(path.as_path())?;

        let dir = path.parent().expect(&format!(
            "failed to get parent dir of {} bin",
            bin_name::FFMPEG
        ));

        let version = check_latest_version()?;

        let archive_path = download_ffmpeg_package(&version, dir)?;
        unpack_ffmpeg(&archive_path, dir)?;

        Ok(())
    }

    fn ffprobe_path() -> anyhow::Result<PathBuf> {
        Ok(local_bin_dir()?.join(bin_name::FFPROBE))
    }
}
