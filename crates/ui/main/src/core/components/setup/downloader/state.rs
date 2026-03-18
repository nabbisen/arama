use super::config::DownloaderConfig;

#[derive(Debug, Clone)]
pub struct DownloaderState {
    pub config: DownloaderConfig,
    pub download_state: DownloadState,
}

#[derive(Debug, Clone)]
pub enum DownloadProgress {
    Downloading(f32),
    Finished,
    Errored(String),
}

#[derive(Debug, Clone, Default)]
pub enum DownloadState {
    #[default]
    Idle,
    Downloading(f32),
    Finished,
    Errored(String),
}
