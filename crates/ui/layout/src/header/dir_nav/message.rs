use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    Event(Event),
    Internal(Internal),
}

#[derive(Debug, Clone)]
pub enum Event {
    DirSelect(PathBuf),
}

#[derive(Debug, Clone)]
pub enum Internal {
    Input(String),
    Submit,
    RfdOpen,
    RfdClose(Option<PathBuf>),
}
