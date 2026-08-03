use arama_env::local_dir;
use arama_i18n::t;
use disk_space::DiskSpace;
use iced::Length::Fill;
use iced::widget::{Column, button, column, container, progress_bar, row, text};
use iced::{Element, Length, alignment};

use crate::components::setup::downloader::config::DownloaderConfig;

use super::{Downloader, message::Message, state::DownloadState};

impl Downloader {
    pub fn view(&self) -> Element<'_, Message> {
        let download_requires = self
            .states
            .iter()
            .enumerate()
            .filter(|(_, state)| state.download_state != DownloadState::NotRequired)
            .fold(
                column![text(t("setup.not_ready"))]
                    .max_width(400)
                    .spacing(10),
                |col, (id, state)| {
                    let (status, progress) = match &state.download_state {
                        DownloadState::Idle => (t("setup.status.missing"), 0.0),
                        DownloadState::Checking => (t("setup.status.checking"), 0.0),
                        DownloadState::WorkerDraining => {
                            (t("setup.status.ffmpeg_worker_draining"), 0.0)
                        }
                        DownloadState::Downloading(p) => {
                            (format!("{} {:.1}%", t("setup.status.downloading"), *p), *p)
                        }
                        DownloadState::Finished => (t("setup.status.ready"), 100.0),
                        DownloadState::Errored(e) => {
                            (format!("{}: {}", t("setup.status.error"), e), 0.0)
                        }
                        DownloadState::NotRequired => unreachable!(),
                        DownloadState::ExternalRequired => {
                            (t("setup.status.external_required"), 0.0)
                        }
                    };

                    let name = if state.download_state == DownloadState::ExternalRequired {
                        state_name(&state.config)
                    } else {
                        let size_str = if let Some(x) = state.file_size {
                            x.to_string()
                        } else {
                            t("setup.item.size_unknown")
                        };
                        format!("{} ({} MB)", state_name(&state.config), size_str)
                    };

                    let mut item =
                        column![text(format!("{} : {}", name, status)).size(14)].spacing(5);
                    if state.download_state == DownloadState::ExternalRequired {
                        item = item
                            .push(text(t("setup.ffmpeg.external_help")).size(14))
                            .push(
                                row![
                                    button(text(t("setup.ffmpeg.recheck")))
                                        .on_press(Message::RecheckFfmpeg(id)),
                                    button(text(t("setup.ffmpeg.select")))
                                        .on_press(Message::SelectFfmpegDirectory),
                                ]
                                .spacing(10),
                            );
                    } else {
                        item = item.push(container(progress_bar(0.0..=100.0, progress)).height(12));
                    }
                    col.push(item)
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

        let disk = disk_info_view();

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

fn disk_info_view<'a>() -> Column<'a, Message> {
    let local_dir = match local_dir() {
        Ok(local_dir) => local_dir,
        Err(err) => {
            return column![
                text(t("setup.download_into")),
                text(format!("{}: {}", t("setup.status.error"), err))
            ]
            .spacing(5);
        }
    };

    let disk_space = match DiskSpace::new(&local_dir) {
        Ok(disk_space) => disk_space,
        Err(err) => {
            return column![
                text(t("setup.download_into")),
                text(local_dir.to_string_lossy().to_string()),
                text(format!("{}: {}", t("setup.status.error"), err))
            ]
            .spacing(5);
        }
    };

    let disk_space_as_gb = disk_space.as_gb();
    column![
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
    .spacing(5)
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
                eprintln!(
                    "state_name: unknown AI model config at {}",
                    safetensors_path.display()
                );
                t("setup.item.clip") // degrade gracefully instead of panicking
            }
        }
        DownloaderConfig::Ffmepg => t("setup.item.ffmpeg"),
    }
}
