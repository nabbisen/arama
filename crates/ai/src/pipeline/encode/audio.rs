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
    /// One entry per segment that encoded successfully - a segment whose
    /// encoding fails is *excluded*, not replaced with a zero vector
    /// (Task 042 / audit A10): a zero vector is not a neutral element of
    /// a mean, and it is not unit-length. The returned length can
    /// therefore be shorter than `segments.len()`, down to and including
    /// empty when every segment failed - the caller derives the failure
    /// count from `segments.len() - result.len()` and must fold it into
    /// whatever failure count already reaches its own caller, or a
    /// partial encode failure becomes invisible again one level up.
    ///
    /// Each returned vector is L2-normalized.
    fn encode_segments(&self, segments: &[AudioSegmentView<'_>]) -> Vec<Vec<f32>>;

    /// Output vector dimension.
    fn feature_dim(&self) -> usize;

    /// Sample rate requested from ffmpeg, in Hz.
    fn required_sample_rate(&self) -> u32;
}
