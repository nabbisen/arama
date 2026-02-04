use iced::Element;
use iced::widget::{button, row, text};

use super::{RootDirSelect, message::Message};

impl RootDirSelect {
    // ビュー（UI描画）
    pub fn view(&self) -> Element<'_, Message> {
        let button = button("Root Dir").on_press(Message::DialogOpen);
        let label = text(if let Some(selected_path) = self.selected_path.as_ref() {
            selected_path.to_string_lossy()
        } else {
            "".into()
        });
        row![button, label].into()
    }
}
