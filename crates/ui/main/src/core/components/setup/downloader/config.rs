use std::path::PathBuf;

use arama_ai::model::model_container::SourceUrl;

#[derive(Debug, Clone)]
pub enum DownloaderConfig {
    AiModel(SourceUrl, PathBuf),
    Ffmepg,
}
