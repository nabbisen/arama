use arama_ui_widgets::dir_tree;

#[derive(Debug, Clone)]
pub enum Message {
    Open,
    Close,
    DirTreeMessage(dir_tree::message::Message),
}
