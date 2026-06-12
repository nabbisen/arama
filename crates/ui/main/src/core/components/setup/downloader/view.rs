use arama_env::local_dir;
use arama_i18n::t;
use disk_space::DiskSpace;
use iced::Length::Fill;
use iced::widget::{column, container, progress_bar, row, text};
use iced::{Element, Length, alignment};

use crate::components::setup::downloader::config::DownloaderConfig;

use super::{Downloader, message::Message, state::DownloadState};

impl Downloader {
    pub fn view(&self) -> Element<'_, Message> {
        let download_requires = self
            .states
            .iter()
            .filter(|x| x.download_state != DownloadState::NotRequired)
            .fold(
                column![text(t("setup.not_ready"))].max_width(400).spacing(10),
                |col, state| {
                    let (status, progress) = match &state.download_state {
                        DownloadState::Idle => (t("setup.status.missing"), 0.0),
                        DownloadState::Downloading(p) => {
                            (format!("{} {:.1}%", t("setup.status.downloading"), *p), *p)
                        }
                        DownloadState::Finished => (t("setup.status.ready"), 100.0),
                        DownloadState::Errored(e) => {
                            (format!("{}: {}", t("setup.status.error"), e), 0.0)
                        }
                        DownloadState::NotRequired => unreachable!(),
                    };

                    let size_str = if let Some(x) = state.file_size {
                        x.to_string()
                    } else {
                        t("setup.item.size_unknown")
                    };
                    let name = format!(
                        "{} ({} MB)",
                        state_name(&state.config),
                        size_str,
                    );

                    col.push(
                        column![
                            text(format!("{} : {}", name, status)).size(14),
                            container(progress_bar(0.0..=100.0, progress)).height(12),
                        ]
                        .spacing(5),
                    )
                },
            );

        let download_not_requires = row![
            text(t("setup.ready")),
            self.states
                .iter()
                .filter(|x| x.download_state == DownloadState::NotRequired)
                .fold(column![].spacing(5), |col, state| {
                    col.push(text(state_name(&state.config)))
                })
        ]
        .spacing(5);

        let local_dir = local_dir().unwrap();
        let disk_space = DiskSpace::new(&local_dir).expect("failed to get file system info");
        let disk_space_as_gb = disk_space.as_gb();
        let disk = column![
            text(t("setup.download_into")),
            text(local_dir.to_string_lossy().to_string()),
            text(format!(
                "({}: {:.1} {} / {:.1} {})",
                t("setup.disk_space"),
                disk_space_as_gb.available,
                t("setup.disk_gb_avail"),
                disk_space_as_gb.total,
                t("setup.disk_gb_total"),
            ))
        ]
        .spacing(5);

        container(
            container(
                column![download_requires, disk, download_not_requires]
                    .align_x(alignment::Horizontal::Left)
                    .spacing(20),
            )
            .width(Length::Shrink),
        )
        .center_x(Fill)
        .into()
    }
}

fn state_name(config: &DownloaderConfig) -> String {
    match config {
        DownloaderConfig::AiModel(model_container) => {
            let Ok(safetensors_path) = model_container.safetensors_path() else {
                return t("setup.item.clip"); // safe fallback
            };
            let parent_name = safetensors_path
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            if parent_name.contains("clip") {
                t("setup.item.clip")
            } else if parent_name.contains("wav2vec2") {
                t("setup.item.wav2vec2")
            } else {
                eprintln!("state_name: unknown AI model config at {}", safetensors_path.display());
                t("setup.item.clip") // degrade gracefully instead of panicking
            }
        }
        DownloaderConfig::Ffmepg => t("setup.item.ffmpeg"),
    }
}
