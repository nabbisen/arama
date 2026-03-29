pub mod view;

const MIN_SIMILARITY_RANGE: f32 = 0.5;
const MAX_SIMILARITY_RANGE: f32 = 1.0;

#[derive(Clone, Debug)]
pub struct SimilarityBadge {
    similarity: f32,
}

impl SimilarityBadge {
    pub fn new(similarity: f32) -> Self {
        Self { similarity }
    }
}
