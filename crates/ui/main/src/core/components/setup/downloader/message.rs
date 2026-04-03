use super::state::DownloadProgress;

#[derive(Debug, Clone)]
pub enum Message {
    StartDownloads,
    AiModelProgressUpdated(usize, DownloadProgress),
    GeneralProgressUpdated(usize, DownloadProgress),
}
