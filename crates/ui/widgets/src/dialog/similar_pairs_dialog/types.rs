use crate::dialog::similarity_read_outcome::SimilarityReadOutcome;

#[derive(Clone, Debug)]
pub struct SimilarPair {
    pub left: SimilarPairItem,
    pub right: SimilarPairItem,
    pub similarity: f32,
}

/// RFC 047 S3.3: `prepare_embeddings`'s full result - the shared
/// [`SimilarityReadOutcome`] plus how many pairs matched the threshold
/// before `MAX_SIMILAR_PAIRS` truncated the list. Kept separate from
/// `SimilarityReadOutcome` itself (rather than adding a field to it)
/// because that type is shared with `media_focus_dialog`, which has no
/// cap or retained-versus-total concept at all.
#[derive(Clone, Debug)]
pub struct EmbeddingsReadyResult {
    pub outcome: SimilarityReadOutcome<SimilarPair>,
    pub total_found: usize,
}

// Written by hand rather than `#[derive(Default)]`: the derive macro
// adds a `SimilarPair: Default` bound ("imperfect derives") that this
// type does not actually need - `Vec<SimilarPair>::default()` is `vec![]`
// regardless of whether `SimilarPair` itself implements `Default`, and
// adding that impl just to satisfy the derive would be a wider promise
// than this type needs.
impl Default for EmbeddingsReadyResult {
    fn default() -> Self {
        Self {
            outcome: SimilarityReadOutcome {
                items: vec![],
                had_errors: false,
                nothing_indexed: false,
                ffmpeg_missing_with_videos: false,
            },
            total_found: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SimilarPairItem {
    pub path: String,
    pub thumbnail_path: Option<String>,
}
