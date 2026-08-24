use super::config::DownloaderConfig;

#[derive(Debug, Clone)]
pub struct DownloaderState {
    pub config: DownloaderConfig,
    pub file_size: Option<u64>,
    pub download_state: DownloadState,
}

#[derive(Debug, Clone)]
pub enum DownloadProgress {
    Downloading(DownloadBytes),
    Finished(DownloaderConfig),
    Errored(String),
}

/// Task 036: real bytes written to disk so far, and the real total if
/// one is known - never a computed or animated stand-in. `total` is
/// `None` whenever `ModelContainer::download_with_progress` could not
/// derive an honest one (no file has reported a length yet, or some
/// file in this generation never did); the view must show bytes-so-far
/// rather than invent a percentage in that case.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DownloadBytes {
    pub downloaded: u64,
    pub total: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum DownloadState {
    #[default]
    Idle,
    Checking,
    WorkerDraining,
    Downloading(DownloadBytes),
    Finished,
    Errored(String),
    NotRequired,
    ExternalRequired,
}
