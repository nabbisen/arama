use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CacheLookupStrategy {
    Everywhere,
    CurrentDirAndSubDirs,
    #[default]
    CurrentDirOnly,
}

impl CacheLookupStrategy {
    pub const ALL: [CacheLookupStrategy; 3] = [
        Self::Everywhere,
        Self::CurrentDirAndSubDirs,
        Self::CurrentDirOnly,
    ];
}

impl std::fmt::Display for CacheLookupStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::Everywhere => "Everywhere",
            Self::CurrentDirAndSubDirs => "Current dir and subdirs",
            Self::CurrentDirOnly => "Current dir only",
        };
        write!(f, "{}", label)
    }
}
