use super::state::DownloadProgress;

#[derive(Debug, Clone)]
pub enum Message {
    CheckResources,
    MetadataChecked(usize, Option<u64>),
    RecheckFfmpeg(usize),
    ExternalFfmpegRequested,
    SelectFfmpegDirectory,
    FfmpegDirectorySelectRequested,
    StartDownloads,
    AiModelProgressUpdated(usize, DownloadProgress),
}
