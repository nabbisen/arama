#[derive(Clone, Debug)]
pub struct SimilarPair {
    pub left: SimilarPairItem,
    pub right: SimilarPairItem,
    pub similarity: f32,
}

#[derive(Clone, Debug)]
pub struct SimilarPairItem {
    pub path: String,
    pub thumbnail_path: Option<String>,
}
