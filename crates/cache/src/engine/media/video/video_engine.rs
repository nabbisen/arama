use std::{path::PathBuf, process::Command};

use arama_env::{local_bin_dir, validate_dir};
use ffmpeg_sidecar::download::{check_latest_version, download_ffmpeg_package, unpack_ffmpeg};

#[cfg(not(windows))]
const BIN_NAME: &str = "ffmpeg";
#[cfg(windows)]
const BIN_NAME: &str = "ffmpeg.exe";

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
                Self::file_path().expect("failed to get ffmpeg in local bin"),
            )),
            FfmpegStatus::ExistsInPath => Some(Command::new("ffmpeg")),
            FfmpegStatus::NotExists => None,
        }
    }

    pub fn ready() -> FfmpegStatus {
        if Self::file_path().is_ok_and(|path| path.exists()) {
            return FfmpegStatus::ExistsInLocalBin;
        }

        if Command::new("ffmpeg")
            .args(["-version"])
            .output()
            .is_ok_and(|x| x.status.success())
        {
            return FfmpegStatus::ExistsInPath;
        }

        FfmpegStatus::NotExists
    }

    fn file_path() -> anyhow::Result<PathBuf> {
        Ok(local_bin_dir()?.join(BIN_NAME))
    }
}
