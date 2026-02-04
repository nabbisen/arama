use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    DialogOpen,
    DialogClose(Option<PathBuf>),
}
