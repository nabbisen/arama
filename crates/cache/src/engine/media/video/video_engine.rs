use std::{path::PathBuf, process::Command};

use arama_env::{local_bin_dir, validate_dir};
use ffmpeg_sidecar::download::{check_latest_version, download_ffmpeg_package, unpack_ffmpeg};

#[cfg(not(windows))]
const BIN_NAME: &str = "ffmpeg";
#[cfg(windows)]
const BIN_NAME: &str = "ffmpeg.exe";

pub struct VideoEngine {}

impl VideoEngine {
    pub fn ready() -> bool {
        if Self::file_path().is_ok_and(|path| path.exists()) {
            return true;
        }

        Command::new("ffmpeg")
            .args(["-version"])
            .output()
            .is_ok_and(|x| x.status.success())
    }

    fn file_path() -> anyhow::Result<PathBuf> {
        Ok(local_bin_dir()?.join(BIN_NAME))
    }
}
