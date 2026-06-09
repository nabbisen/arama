//! Payload structs stored per cached file.
//!
//! These replace the v1 extension tables (`thumbnails`, `image_features`,
//! `video_features`): one typed payload per file, serialized by
//! `localcache`'s bincode codec (wire-format stability is guaranteed by
//! localcache RFC 0008).
//!
//! **Layout changes are breaking.** Adding, removing, or reordering fields
//! changes the bincode layout; any such change must be accompanied by a
//! bump of the matching `*_PAYLOAD_VERSION` constant in
//! [`crate::core::engine`] so stale entries are purged instead of
//! mis-decoded.

use serde::{Deserialize, Serialize};

/// Payload for one cached image file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct ImagePayload {
    /// Absolute path of the generated thumbnail, when one was produced.
    pub thumbnail_path: Option<String>,
    /// CLIP feature vector (one vector per image).
    pub clip_vector: Option<Vec<f32>>,
}

/// Payload for one cached video file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct VideoPayload {
    /// Absolute path of the generated poster thumbnail, when produced.
    pub thumbnail_path: Option<String>,
    /// Frame-averaged CLIP feature vector.
    pub clip_vector: Option<Vec<f32>>,
    /// Scene-averaged wav2vec2 feature vector.
    pub wav2vec2_vector: Option<Vec<f32>>,
}
