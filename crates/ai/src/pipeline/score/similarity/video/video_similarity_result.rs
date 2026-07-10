#[derive(Debug)]
pub struct VideoSimilarityResult {
    pub video_a: String,
    pub video_b: String,
    pub image_sim: f32,      // Video cross-max similarity.
    pub audio_sim: f32,      // Audio cross-max similarity.
    pub combined_score: f32, // Weighted total.
}
