#[derive(Debug, Clone)]
pub enum Message {
    /// Task 039: the button only ever *requests* deletion now - the
    /// actual delete happens after confirmation, at the app level (see
    /// `settings_dialog::message::Message::CacheDeleteRequested`, which
    /// this bubbles into).
    CacheDeleteRequested,
}
