#[derive(Clone, Debug)]
pub struct SimilarMediaItem {
    pub path: String,
    pub thumbnail_path: Option<String>,
    pub similarity: f32,
}
