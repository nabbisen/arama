use iced::Subscription;

use super::{App, message::Message};

impl App {
    pub fn subscription(&self) -> Subscription<Message> {
        // let subscriptions = [self.gallery.subscription().map(Message::GalleryMessage)];
        // Subscription::batch(subscriptions)
        Subscription::batch([])
    }
}
