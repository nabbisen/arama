use iced::Subscription;

use super::{App, message::Message};

impl App {
    pub fn subscription(&self) -> Subscription<Message> {
        // RFC 044: `iced::keyboard::listen()` only fires for keys no
        // widget already claimed (it filters on
        // `event::Status::Ignored`) - Phase 0.1 confirmed every key arama
        // cares about (Escape, F6, Tab) is ignored today, so this is the
        // one subscription that reaches all of them.
        let key_sub = iced::keyboard::listen().map(|event| match event {
            iced::keyboard::Event::KeyPressed { key, modifiers, .. } => {
                Message::KeyPressed(key, modifiers)
            }
            _ => Message::NoOp,
        });

        Subscription::batch([
            snora::toast::subscription(&self.toasts, || Message::ToastSweep),
            key_sub,
        ])
    }
}
