use iced::Task;

use super::{
    Header, dir_nav,
    message::{Event, Internal, Message},
};

impl Header {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Event(_) => return Task::none(),
            Message::Internal(message) => match message {
                Internal::DirNavMessage(message) => {
                    let task = self
                        .dir_nav
                        .update(message.clone())
                        .map(|x| Message::Internal(Internal::DirNavMessage(x)));
                    match message {
                        dir_nav::message::Message::Event(message) => match message {
                            dir_nav::message::Event::DirSelect(path) => {
                                return Task::done(Message::Event(Event::DirSelect(path)));
                            }
                        },
                        dir_nav::message::Message::Internal(_) => return task,
                    }
                }
                Internal::SettingsNavMessage(message) => {
                    let task = self
                        .settings_nav
                        .update(message)
                        .map(|x| Message::Internal(Internal::SettingsNavMessage(x)));
                    return task;
                }
            },
        }
    }
}
