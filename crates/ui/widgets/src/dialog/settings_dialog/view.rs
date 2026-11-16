use iced::{
    Element,
    Length::Fill,
    widget::{button, column, container, row, scrollable, text},
};

use super::{SettingsDialog, Tab, message::Message};

impl SettingsDialog {
    pub fn view(&self) -> Element<'_, Message> {
        let tab_menus = Tab::all().iter().fold(row![].spacing(5), |acc, x| {
            let menu = button(text(x.label())).on_press(Message::TabSelect(x.to_owned()));
            acc.push(menu)
        });

        let tab = match self.tab {
            Tab::General => self
                .general_settings
                .view()
                .map(Message::GeneralSettingsTabMessage),
            Tab::Ai => self.ai_settings.view().map(Message::AiSettingsTabMessage),
            Tab::FileSystem => self
                .file_system_settings
                .view()
                .map(Message::FileSystemSettingsTabMessage),
            Tab::About => self.about.view().map(Message::AboutTabMessage),
        };

        // T063 (RFC 052 §2): a fixed-size, non-scrolling box here let a
        // `Fill`-constrained column (`general_settings::view`, no
        // `scrollable` of its own) starve later rows to zero height
        // once content grew past 400px - first seen with the Larger
        // text step at the 760x520 minimum window. Scrolling the tab's
        // own content, not just its container, keeps the box's size
        // stable while letting content that outgrows it scroll instead
        // of being silently dropped.
        column![
            tab_menus,
            container(scrollable(tab).height(Fill))
                .width(600)
                .height(400)
        ]
        .spacing(10)
        .into()
    }
}
