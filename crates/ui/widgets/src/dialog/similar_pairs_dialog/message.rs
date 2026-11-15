use super::types::EmbeddingsReadyResult;

#[derive(Debug, Clone)]
pub enum Message {
    /// Review 162 (T061's second rework): the `u64` is the generation
    /// `default_task` was started with - the same value `App` had just
    /// incremented and stored at the moment this scan began. A dialog
    /// close-then-reopen starts a new scan with a new generation while
    /// the old one, unable to be aborted (`ScanControl`'s own doc
    /// comment explains why), is still running; without this, its
    /// eventual partial result would be accepted as if it were the new
    /// scan's, overwriting the new scan's own in-progress state and
    /// dropping its control. The caller (`App`) compares this against
    /// its current generation and drops anything that does not match,
    /// before this message ever reaches `update` below.
    EmbeddingsReady(u64, EmbeddingsReadyResult),
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
