use iced::{
    Element,
    widget::{button, column, row},
};

use super::{SettingsDialog, Tab, message::Message};

impl SettingsDialog {
    pub fn view(&self) -> Element<'_, Message> {
        let tab_menus: Vec<Element<Message>> = Tab::all()
            .iter()
            .map(|x| {
                button(x.label())
                    .on_press(Message::TabSelect(x.to_owned()))
                    .into()
            })
            .collect();
        let tab_menus_container = row(tab_menus);

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
        };

        // todo
        column![tab_menus_container, tab].into()
    }
}
