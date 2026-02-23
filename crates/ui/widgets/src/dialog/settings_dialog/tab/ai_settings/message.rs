#[derive(Debug, Clone)]
pub enum Message {
    LoadStart,
    Loaded(Option<String>),
}
