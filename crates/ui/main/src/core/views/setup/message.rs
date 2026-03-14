use crate::components::setup::downloader;

#[derive(Debug, Clone)]
pub enum Message {
    Download,
    Skip,
    DownloaderMessage(downloader::message::Message),
}
