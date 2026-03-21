use iced::Task;

use super::MediaFocusDialog;
use super::message::Message;

impl MediaFocusDialog {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SimilarMediaReady(similar_images) => self.similar_media = similar_images,
            Message::MediaItemEnter(path) => self.hovered_media_item_path = Some(path),
            Message::MediaItemExit => self.hovered_media_item_path = None,
            Message::ViewSizeToggle(actual_size) => self.actual_size = actual_size,
            Message::CloseClick => (),
        }
        Task::none()
    }
}
