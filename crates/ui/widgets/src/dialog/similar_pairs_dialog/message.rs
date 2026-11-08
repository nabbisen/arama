use super::types::EmbeddingsReadyResult;

#[derive(Debug, Clone)]
pub enum Message {
    EmbeddingsReady(EmbeddingsReadyResult),
    /// RFC 047 S6 / review 161 #3: `(compared, total)`, read from the
    /// scan's own `ScanControl` on a timer while it runs. Delivered only
    /// while `pairs` is still `None` - once `EmbeddingsReady` arrives a
    /// late tick is possible but harmless, `update` simply has nothing
    /// left to do with it.
    Progress(usize, usize),
    MediaItemEnter(String),
    MediaItemDoubleClicked(String),
    MediaExit,
}
