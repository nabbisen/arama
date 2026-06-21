use iced::Subscription;

use super::{App, message::Message};

impl App {
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([snora::toast::subscription(&self.toasts, || {
            Message::ToastSweep
        })])
    }
}
