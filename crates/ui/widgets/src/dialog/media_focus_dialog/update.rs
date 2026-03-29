use iced::Task;

use super::MediaFocusDialog;
use super::message::Message;

impl MediaFocusDialog {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SimilarMediaReady(similar_images) => self.similar_media = similar_images,
            Message::SimilarMediaItemDoubleClicked(path) => {
                self.history.push(path.clone());
                self.history_index = self.history.len() - 1;
                self.hovered_media_item_path_str = None;
                self.similar_media = vec![];

                let cloned = self.clone();
                return Task::perform(
                    async move { cloned.similar_media() },
                    Message::SimilarMediaReady,
                );
            }
            Message::HistoryPrevious => {
                self.history_index -= 1;

                let cloned = self.clone();
                return Task::perform(
                    async move { cloned.similar_media() },
                    Message::SimilarMediaReady,
                );
            }
            Message::HistoryNext => {
                self.history_index += 1;

                let cloned = self.clone();
                return Task::perform(
                    async move { cloned.similar_media() },
                    Message::SimilarMediaReady,
                );
            }
            Message::FileShow => {
                let path = self.history[self.history_index].clone();
                let _ = file_handle::FileHandle::show(&path);
            }
            Message::CacheLookupStrategyChanged(x) => {
                self.cache_lookup_strategy = x;

                let cloned = self.clone();
                return Task::perform(
                    async move { cloned.similar_media() },
                    Message::SimilarMediaReady,
                );
            }
            Message::MediaItemEnter(path_str) => self.hovered_media_item_path_str = Some(path_str),
            Message::MediaItemExit => self.hovered_media_item_path_str = None,
            Message::ViewSizeToggle => self.actual_size = !self.actual_size,
            Message::CloseClick => (),
        }
        Task::none()
    }
}
