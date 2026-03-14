use std::{path::PathBuf, process::Command};

use anyhow::Context;
use arama_env::{local_bin_dir, validate_dir};
use ffmpeg_sidecar::download::{download_ffmpeg_package, ffmpeg_download_url, unpack_ffmpeg};

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

        let parent_dir = path.parent().context(format!(
            "failed to get parent dir of {} bin",
            bin_name::FFMPEG
        ))?;

        validate_dir(parent_dir)?;

        // let version = check_latest_version()?;
        let download_url = ffmpeg_download_url()?;
        let archive_path = download_ffmpeg_package(&download_url, parent_dir)?;
        unpack_ffmpeg(&archive_path, &parent_dir)?;

        Ok(())
    }

    fn ffprobe_path() -> anyhow::Result<PathBuf> {
        Ok(local_bin_dir()?.join(bin_name::FFPROBE))
    }
}
