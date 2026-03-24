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

    pub fn label(&self) -> &'static str {
        match self {
            Tab::General => "General",
            Tab::Ai => "AI",
            Tab::FileSystem => "File system",
            Tab::About => "About",
        }
    }
}

impl Default for Tab {
    fn default() -> Self {
        Tab::General
    }
}
