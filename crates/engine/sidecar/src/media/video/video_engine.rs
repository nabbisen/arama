use std::{path::PathBuf, process::Command};

use anyhow::Context;
use arama_env::{local_bin_dir, validate_dir};
use ffmpeg_sidecar::download::{ffmpeg_download_url, unpack_ffmpeg};

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

    pub fn download_url() -> anyhow::Result<String> {
        let download_url = ffmpeg_download_url()?;
        Ok(download_url.to_owned())
    }

    pub fn download_dest_path() -> anyhow::Result<PathBuf> {
        let download_url = VideoEngine::download_url()?;
        let file_name = download_url.split("/").last().unwrap();

        let parent_dir = VideoEngine::parent_dir()?;

        let download_dest_path = parent_dir.join(file_name);

        Ok(download_dest_path)
    }

    pub fn unpack_archive() -> anyhow::Result<()> {
        let download_dest_path = VideoEngine::download_dest_path()?;
        let parent_dir = VideoEngine::parent_dir()?;

        unpack_ffmpeg(&download_dest_path, &parent_dir)?;

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
