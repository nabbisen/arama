/// Result of a similarity-dialog cache read (RFC 035): the items obtained,
/// plus whether any read failed along the way. `had_errors` with a
/// non-empty `items` is a partial result; with an empty `items` it is a
/// failed lookup, and must not be presented as an absence of matches.
#[derive(Clone, Debug, Default)]
pub struct SimilarityReadOutcome<T> {
    pub items: Vec<T>,
    pub had_errors: bool,
}
