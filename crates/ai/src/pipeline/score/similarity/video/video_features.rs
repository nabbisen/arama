#[derive(Debug, Clone)]
pub struct VideoFeatures {
    pub path: String,

    /// CLIP 映像埋め込み: [N_frames × 512]（L2 正規化済み）
    pub video_embeddings: Vec<f32>,
    // /// Whisper 音声埋め込み: [N_segments × hidden_dim]（L2 正規化済み）
    // /// hidden_dim: tiny=384 / base=512 / small=768 / medium=1024
    // pub audio_embeddings: Vec<f32>,
}
