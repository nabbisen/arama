use std::path::PathBuf;

use arama_env::cache_lookup_strategy::CacheLookupStrategy;
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
    similar_media: Vec<SimilarMediaItem>,
}

impl MediaFocusDialog {
    pub fn new<T: Into<PathBuf>>(path: T, cache_lookup_strategy: CacheLookupStrategy) -> Self {
        Self {
            history: vec![path.into()],
            history_index: 0,
            hovered_media_item_path_str: None,
            actual_size: false,
            cache_lookup_strategy: cache_lookup_strategy,
            similar_media: vec![],
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
