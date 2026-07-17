use super::state::DownloadProgress;

#[derive(Debug, Clone)]
pub enum Message {
    CheckResources,
    MetadataChecked(usize, Option<u64>),
    FfmpegChecked(usize, Result<(bool, Option<u64>), String>),
    RecheckFfmpeg(usize),
    ExternalFfmpegRequested,
    StartDownloads,
    AiModelProgressUpdated(usize, DownloadProgress),
    GeneralProgressUpdated(usize, DownloadProgress),
}
