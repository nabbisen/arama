use super::state::DownloadProgress;

#[derive(Debug, Clone)]
pub enum Message {
    StartDownloads,
    ProgressUpdated(usize, DownloadProgress),
}
