use std::path::PathBuf;

use arama_env::cache_lookup_strategy::CacheLookupStrategy;
use arama_sidecar::media::video::video_engine::FfmpegToolchain;
use iced::Task;

pub mod message;
mod similar_media;
mod types;
mod update;
mod view;

use message::Message;
use types::SimilarMediaItem;

#[derive(Clone, Debug)]
pub struct MediaFocusDialog {
    history: Vec<PathBuf>,
    history_index: usize,
    hovered_media_item_path_str: Option<String>,
    actual_size: bool,
    cache_lookup_strategy: CacheLookupStrategy,
    similarity_threshold: f32,
    similar_media: Vec<SimilarMediaItem>,
    /// Set when any cache read failed while preparing `similar_media`
    /// (RFC 035). One aggregated flag per lookup, not one per failed file.
    has_read_error: bool,
    ffmpeg_toolchain: Option<FfmpegToolchain>,
}

impl MediaFocusDialog {
    pub fn new<T: Into<PathBuf>>(
        path: T,
        cache_lookup_strategy: CacheLookupStrategy,
        similarity_threshold: f32,
        ffmpeg_toolchain: Option<FfmpegToolchain>,
    ) -> Self {
        Self {
            history: vec![path.into()],
            history_index: 0,
            hovered_media_item_path_str: None,
            actual_size: false,
            cache_lookup_strategy,
            similarity_threshold,
            similar_media: vec![],
            has_read_error: false,
            ffmpeg_toolchain,
        }
    }

    pub fn default_task(&self) -> Task<Message> {
        let cloned = self.clone();
        Task::perform(
            async move { cloned.similar_media() },
            Message::SimilarMediaReady,
        )
    }
}
