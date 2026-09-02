use arama_i18n::t;
use iced::{
    Element,
    widget::{button, column, row, text},
};

use super::{ConfirmDialog, message::Message};

impl ConfirmDialog {
    pub fn view(&self) -> Element<'_, Message> {
        column![
            text(self.title.clone())
                .size(arama_theme::title_size())
                .line_height(arama_theme::title_line_height()),
            text(self.body.clone())
                .size(arama_theme::body_size())
                .line_height(arama_theme::body_line_height()),
            row![
                button(text(t("confirm.cancel"))).on_press(Message::Cancel),
                button(text(self.confirm_label.clone()))
                    .style(arama_theme::danger)
                    .on_press(Message::Confirm),
            ]
            .spacing(10),
        ]
        .spacing(15)
        .into()
    }
}
