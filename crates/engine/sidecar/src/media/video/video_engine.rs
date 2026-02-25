use std::{path::PathBuf, process::Command};

use arama_env::{local_bin_dir, validate_dir};
use ffmpeg_sidecar::download::{check_latest_version, download_ffmpeg_package, unpack_ffmpeg};

#[cfg(not(windows))]
mod bin_name {
    pub const FFMPEG: &str = "ffmpeg";
}
#[cfg(windows)]
mod bin_name {
    pub const FFMPEG: &str = "ffmpeg.exe";
}

#[derive(PartialEq)]
pub enum FfmpegStatus {
    ExistsInLocalBin,
    ExistsInPath,
    NotExists,
}

pub struct VideoEngine {}

impl VideoEngine {
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

    fn ffmpeg_path() -> anyhow::Result<PathBuf> {
        Ok(local_bin_dir()?.join(bin_name::FFMPEG))
    }
}
