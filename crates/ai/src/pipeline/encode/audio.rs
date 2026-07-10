//! Shared audio encoder interface.
mod wav2vec2_config;
pub mod wav2vec2_encoder;

use crate::pipeline::extract::video_extractor::audio_segment::AudioSegmentView;

/// Audio encoder trait.
///
/// Returns one embedding vector per segment instead of collapsing all segments
/// into one vector. Callers can then compute cross-max similarity, which is
/// robust to opening cuts, ending cuts, and timeline offsets.
pub trait AudioEncoder: Send + Sync {
    /// Encode each segment independently and return a vector sequence.
    ///
    /// Return shape: `[N_segments x feature_dim]`; each vector is L2-normalized.
    fn encode_segments(&self, segments: &[AudioSegmentView<'_>]) -> Vec<Vec<f32>>;

    /// Output vector dimension.
    fn feature_dim(&self) -> usize;

    /// Sample rate requested from ffmpeg, in Hz.
    fn required_sample_rate(&self) -> u32;
}
