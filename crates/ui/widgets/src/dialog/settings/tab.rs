pub(super) mod ai_settings;
pub(super) mod file_system_settings;
pub(super) mod general_settings;

#[derive(Clone, Debug)]
pub enum Tab {
    General,
    Ai,
    FileSystem,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[Tab::General, Tab::Ai, Tab::FileSystem]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Tab::General => "General",
            Tab::Ai => "AI",
            Tab::FileSystem => "File system",
        }
    }
}

impl Default for Tab {
    fn default() -> Self {
        Tab::General
    }
}
