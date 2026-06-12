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
    /// Create a fallback Setup that reports itself as already finished,
    /// bypassing the setup wizard. Used when `Setup::default()` fails.
    pub fn fallback() -> Self {
        Self {
            finished: true,
            downloader: Downloader::new(vec![]),
        }
    }

    pub fn default() -> Result<Self> {
        let configs = vec![
            DownloaderConfig::AiModel(clip::model()),
            DownloaderConfig::AiModel(wav2vec2::model()),
            DownloaderConfig::Ffmepg,
        ];

        Ok(Self {
            finished: false,
            downloader: Downloader::new(configs),
        })
    }
}
