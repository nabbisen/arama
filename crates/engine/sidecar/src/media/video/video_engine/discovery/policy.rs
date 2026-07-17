use std::time::Duration;

/// Centralized RFC 032 discovery bounds shared by Auto and Selected modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FfmpegLocatorPolicy {
    pub max_raw_path_entries: usize,
    pub max_path_candidates: usize,
    pub probe_timeout: Duration,
    pub attempt_timeout: Duration,
}

impl Default for FfmpegLocatorPolicy {
    fn default() -> Self {
        Self {
            max_raw_path_entries: 64,
            max_path_candidates: 32,
            probe_timeout: Duration::from_secs(2),
            attempt_timeout: Duration::from_secs(6),
        }
    }
}

#[cfg(test)]
mod tests;
