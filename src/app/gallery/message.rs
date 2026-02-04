use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    ImagesLoaded(Vec<PathBuf>),
    ScaleUp,
    ScaleDown,
    Quit,
}
