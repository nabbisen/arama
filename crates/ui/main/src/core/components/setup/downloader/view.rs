use iced::widget::{column, container, progress_bar, text};
use iced::{Element, Length, alignment};

use crate::components::setup::downloader::config::DownloaderConfig;

use super::{Downloader, message::Message, state::DownloadState};

impl Downloader {
    pub fn view(&self) -> Element<'_, Message> {
        let c = self
            .states
            .iter()
            .enumerate()
            .fold(column![].spacing(20), |col, (_id, state)| {
                let name = match &state.config {
                    DownloaderConfig::AiModel(model_container) => {
                        let safetensors_path = model_container
                            .safetensors_path()
                            .expect("failed to get safetensors path");

                        let parent_dir_name = if let Some(x) = safetensors_path.parent() {
                            x.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string()
                        } else {
                            String::new()
                        };

                        if parent_dir_name.contains("clip") {
                            "Image analysis ai model (clip)".to_owned()
                        } else if parent_dir_name.contains("wav2vec2") {
                            "Audio analysis ai model (wav2vec2)".to_owned()
                        } else {
                            panic!("Unknown donwload config");
                        }
                    }
                    DownloaderConfig::Ffmepg => "Video manipulator (ffmpeg)".to_owned(),
                };

                let (status, val) = match &state.download_state {
                    DownloadState::Idle => ("Missing".to_string(), 0.0),
                    DownloadState::Downloading(p) => (format!("Downloading... {:.1}%", *p), *p),
                    DownloadState::Finished => ("Ready".to_string(), 100.0),
                    DownloadState::Errored(e) => (format!("Error: {}", e), 0.0),
                };

                col.push(
                    column![
                        text(format!("{} : {}", name, status)).size(14),
                        container(progress_bar(0.0..=100.0, val)).height(12),
                    ]
                    .spacing(5),
                )
            });
        let downloads_column = c.max_width(400).align_x(alignment::Horizontal::Center);

        container(downloads_column)
            .padding(40)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
