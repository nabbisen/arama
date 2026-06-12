use arama_i18n::t;

pub(super) mod about;
pub(super) mod ai_settings;
pub(super) mod file_system_settings;
pub(super) mod general_settings;

#[derive(Clone, Debug)]
pub enum Tab {
    General,
    Ai,
    FileSystem,
    About,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[Tab::General, Tab::Ai, Tab::FileSystem, Tab::About]
    }

    pub fn label(&self) -> String {
        match self {
            Tab::General => t("settings.tab.general"),
            Tab::Ai => t("settings.tab.ai"),
            Tab::FileSystem => t("settings.tab.filesystem"),
            Tab::About => t("settings.tab.about"),
        }
    }
}

impl Default for Tab {
    fn default() -> Self {
        Tab::General
    }
}
