#[derive(Debug)]
pub struct VideoSimilarityResult {
    pub video_a: String,
    pub video_b: String,
    pub image_sim: f32,      // 映像 cross-max 類似度
    pub audio_sim: f32,      // 音声 cross-max 類似度
    pub combined_score: f32, // 加重合計
}
