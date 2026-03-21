use std::path::PathBuf;

use iced::Task;

pub mod message;
mod types;
mod update;
mod util;
mod view;

use message::Message;
use types::SimilarMediaItem;
use util::similar_media;

#[derive(Clone, Debug)]
pub struct MediaFocusDialog {
    history: Vec<PathBuf>,
    history_index: usize,
    hovered_media_item_path_str: Option<String>,
    actual_size: bool,
    similar_media: Vec<SimilarMediaItem>,
}

impl MediaFocusDialog {
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        Self {
            history: vec![path.into()],
            history_index: 0,
            hovered_media_item_path_str: None,
            actual_size: false,
            similar_media: vec![],
        }
    }

    pub fn default_task(&self) -> Task<Message> {
        let path = self.history[self.history_index].clone();
        Task::perform(
            async move { similar_media(&path) },
            Message::SimilarMediaReady,
        )
    }
}
