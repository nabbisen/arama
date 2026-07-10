#[derive(Debug, Clone)]
pub struct VideoFeatures {
    pub path: String,

    /// CLIP video embeddings: `[N_frames x 512]`, L2-normalized.
    pub video_embeddings: Vec<f32>,
    /// wav2vec2 audio embeddings: `[N_segments x hidden_dim]`, L2-normalized.
    /// Typical hidden dimensions: tiny=384, base=512, small=768, medium=1024.
    pub audio_embeddings: Vec<f32>,
}
