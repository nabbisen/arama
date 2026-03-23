use super::config::DownloaderConfig;

#[derive(Debug, Clone)]
pub struct DownloaderState {
    pub config: DownloaderConfig,
    pub file_size: Option<u64>,
    pub download_state: DownloadState,
}

#[derive(Debug, Clone)]
pub enum DownloadProgress {
    Downloading(f32),
    Finished,
    Errored(String),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum DownloadState {
    #[default]
    Idle,
    Downloading(f32),
    Finished,
    Errored(String),
    NotRequired,
}
