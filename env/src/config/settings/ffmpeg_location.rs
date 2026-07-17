use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// User-owned location policy for the external ffmpeg/ffprobe pair.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FfmpegLocationPreference {
    /// Search the bounded platform candidate list.
    #[default]
    Auto,
    /// Validate only this directory until the user clears the selection.
    SelectedDirectory(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfmpegPreferenceError {
    EmptyOrRelative,
    NonUnicode,
}

impl FfmpegLocationPreference {
    /// Validate the portion of the preference required for JSON persistence.
    pub fn validate_persistable(&self) -> Result<(), FfmpegPreferenceError> {
        let Self::SelectedDirectory(path) = self else {
            return Ok(());
        };
        if path.as_os_str().is_empty() || !path.is_absolute() {
            return Err(FfmpegPreferenceError::EmptyOrRelative);
        }
        if path.to_str().is_none() {
            return Err(FfmpegPreferenceError::NonUnicode);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
