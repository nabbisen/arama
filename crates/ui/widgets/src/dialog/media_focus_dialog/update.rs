use iced::Task;

use super::MediaFocusDialog;
use super::message::Message;
use super::util::similar_media;

impl MediaFocusDialog {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SimilarMediaReady(similar_images) => self.similar_media = similar_images,
            Message::SimilarMediaItemDoubleClicked(path) => {
                self.path = path.clone();
                self.hovered_media_item_path_str = None;
                self.similar_media = vec![];

                return Task::perform(
                    async move { similar_media(&path) },
                    Message::SimilarMediaReady,
                );
            }
            Message::MediaItemEnter(path_str) => self.hovered_media_item_path_str = Some(path_str),
            Message::MediaItemExit => self.hovered_media_item_path_str = None,
            Message::ViewSizeToggle(actual_size) => self.actual_size = actual_size,
            Message::CloseClick => (),
        }
        Task::none()
    }
}
