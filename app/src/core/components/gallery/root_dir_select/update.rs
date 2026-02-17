use iced::Task;

use super::{RootDirSelect, message::Message};

impl RootDirSelect {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DialogOpen => Task::perform(
                async {
                    rfd::FileDialog::new()
                        .set_title("Folder select")
                        .pick_folder()
                },
                Message::DialogClose,
            ),
            Message::DialogClose(path) => {
                self.selected_path = path;
                Task::none()
            }
        }
    }
}
