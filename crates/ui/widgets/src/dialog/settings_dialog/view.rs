use iced::{
    Element,
    Length::Fill,
    widget::{button, column, container, row},
};

use super::{SettingsDialog, Tab, message::Message};

impl SettingsDialog {
    pub fn view(&self) -> Element<'_, Message> {
        let tab_menus = Tab::all().iter().fold(row![].spacing(5), |acc, x| {
            let menu = button(x.label()).on_press(Message::TabSelect(x.to_owned()));
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

        // todo
        column![tab_menus, container(tab).width(600).height(400)]
            .spacing(10)
            .into()
    }
}
