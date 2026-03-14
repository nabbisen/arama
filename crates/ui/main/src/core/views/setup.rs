pub mod message;
mod update;
pub mod util;
mod view;

use std::io::Result;

use crate::components::setup::downloader::{Downloader, config::DownloaderConfig};
use arama_ai::model::model_container::{clip, wav2vec2};

pub struct Setup {
    pub finished: bool,
    downloader: Downloader,
}

impl Setup {
    pub fn default() -> Result<Self> {
        let clip = clip::model();
        let wav2vec2 = wav2vec2::model();
        let configs = vec![
            DownloaderConfig::AiModel(clip.source_url.clone(), clip.safetensors_path()?),
            DownloaderConfig::AiModel(wav2vec2.source_url.clone(), wav2vec2.safetensors_path()?),
            DownloaderConfig::Ffmepg,
        ];

        Ok(Self {
            finished: false,
            downloader: Downloader::new(configs),
        })
    }
}
