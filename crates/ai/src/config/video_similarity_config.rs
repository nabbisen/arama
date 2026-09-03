//! Similarity scoring settings and sampling timestamp generation.

use crate::{CLIP_IMAGE_SIZE, CROSS_MAX_SIMILARITY_THRESHOLD, VIDEO_IMAGE_WEIGHT};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct VideoSimilarityConfig {
    // Head zone.
    pub head_fixed_anchors_secs: Vec<f64>,

    /// Maximum head-zone length in seconds.
    /// The actual upper bound is `min(head_zone_secs, duration * 0.5)`.
    pub head_zone_secs: f64,

    /// Number of sample points inside the head zone.
    /// This is set to roughly half of all samples to cover the opening more densely.
    pub head_sample_count: usize,

    // Middle and tail anchors.
    /// Percentage anchors across the full video, from 0.0 to 1.0.
    pub percent_anchors: Vec<f64>,

    /// Fixed tail anchors in seconds before the end of the video.
    pub tail_anchors_secs: Vec<f64>,

    // Merge policy.
    /// Minimum gap in seconds used to merge nearby sample points.
    /// Roughly twice the Whisper segment duration is a practical baseline.
    pub min_sample_gap_secs: f64,

    // Audio and image settings.
    /// Length of one audio segment in seconds.
    pub audio_segment_duration_secs: f64,

    /// CLIP input image size in pixels.
    pub clip_image_size: usize,

    // Similarity weights.
    pub image_weight: f32,
    pub audio_weight: f32,

    pub cross_max_similarity_threshold: f32,
}

impl Default for VideoSimilarityConfig {
    fn default() -> Self {
        Self {
            head_fixed_anchors_secs: vec![3.0, 9.0, 15.0],

            // Five points in the first 135 seconds, about 27 seconds apart.
            // With the fixed anchors, 8 of 14 samples are in the opening.
            head_zone_secs: 135.0,
            head_sample_count: 5,

            // Three middle points, scaled by video duration.
            percent_anchors: vec![0.30, 0.50, 0.70],

            // Three tail points.
            tail_anchors_secs: vec![30.0, 15.0, 5.0],

            // Drop later points that are within 20 seconds of an earlier point.
            min_sample_gap_secs: 20.0,

            audio_segment_duration_secs: 20.0,
            clip_image_size: CLIP_IMAGE_SIZE,

            image_weight: VIDEO_IMAGE_WEIGHT,
            audio_weight: video_audio_weight!(),

            cross_max_similarity_threshold: CROSS_MAX_SIMILARITY_THRESHOLD,
        }
    }
}

impl VideoSimilarityConfig {
    /// Generate sampling timestamps for a video duration.
    ///
    /// Steps:
    ///   1. Place `head_sample_count` points evenly inside the head zone.
    ///   2. Convert percentage anchors to absolute seconds.
    ///   3. Convert tail offsets to absolute seconds.
    ///   4. Sort, remove out-of-range points, and merge using `min_sample_gap_secs`.
    pub fn compute_sample_timestamps(&self, duration_secs: f64) -> Vec<f64> {
        let mut points: Vec<f64> = Vec::new();

        // 1. Fixed head anchors, always included when in range.
        for &t in &self.head_fixed_anchors_secs {
            if t < duration_secs {
                points.push(t);
            }
        }

        // 2. Even sampling in the head zone after the fixed anchors.
        let head_zone = self.head_zone_secs.min(duration_secs * 0.5);
        // Divide the interval after the last fixed anchor evenly.
        let head_start = self.head_fixed_anchors_secs.last().copied().unwrap_or(0.0);
        if head_zone > head_start {
            let step = (head_zone - head_start) / (self.head_sample_count + 1) as f64;
            for i in 1..=self.head_sample_count {
                points.push(head_start + step * i as f64);
            }
        }

        // 3. Percentage anchors.
        for &pct in &self.percent_anchors {
            points.push(duration_secs * pct);
        }

        // 4. Fixed tail anchors.
        for &offset in &self.tail_anchors_secs {
            let t = duration_secs - offset;
            if t > 0.0 {
                points.push(t);
            }
        }

        // Task 040 (audit A3): `partial_cmp(...).unwrap()` panics on NaN.
        // `get_duration` now rejects a non-finite duration at its own
        // boundary, but `total_cmp` (total, NaN-safe) closes the panic
        // class here too, regardless of how a non-finite `duration_secs`
        // might reach this function.
        points.sort_by(f64::total_cmp);
        points.retain(|&t| t > 0.0 && t < duration_secs);

        // Fixed anchors are exempt from the minimum-gap merge.
        deduplicate_preserving_fixed(
            points,
            &self.head_fixed_anchors_secs,
            self.min_sample_gap_secs,
        )
    }

    /// Sampling settings summary for logs and debugging.
    pub fn sampling_summary(&self, duration_secs: f64) -> String {
        let ts = self.compute_sample_timestamps(duration_secs);
        let actual_head_zone = self.head_zone_secs.min(duration_secs * 0.5);
        let head_count = ts.iter().filter(|&&t| t <= actual_head_zone).count();
        let tail_threshold = duration_secs
            - self
                .tail_anchors_secs
                .iter()
                .cloned()
                .fold(0.0_f64, f64::max);
        let tail_count = ts.iter().filter(|&&t| t >= tail_threshold).count();
        let mid_count = ts.len().saturating_sub(head_count + tail_count);
        let labels: Vec<String> = ts.iter().map(|&t| format!("{:.0}s", t)).collect();

        format!(
            "duration={:.0}s  total={} [head={}, mid={}, tail={}]  -> [{}]",
            duration_secs,
            ts.len(),
            head_count,
            mid_count,
            tail_count,
            labels.join(", "),
        )
    }
}

fn deduplicate_preserving_fixed(sorted: Vec<f64>, fixed: &[f64], min_gap: f64) -> Vec<f64> {
    let mut result: Vec<f64> = Vec::new();
    for t in sorted {
        let is_fixed = fixed.iter().any(|&f| (f - t).abs() < 1e-9);
        if is_fixed {
            // Always keep fixed anchors.
            result.push(t);
        } else if result.last().is_none_or(|&last| t - last >= min_gap) {
            result.push(t);
        }
    }
    result
}
