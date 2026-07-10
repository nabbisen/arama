// use super::{video_features::VideoFeatures, video_similarity_result::VideoSimilarityResult};

pub struct VideoSimilarityCalculator {
    pub image_weight: f32,
    pub audio_weight: f32,
    pub cross_max_similarity_threshold: f32,
}

impl VideoSimilarityCalculator {
    pub fn new(image_weight: f32, audio_weight: f32, cross_max_similarity_threshold: f32) -> Self {
        Self {
            image_weight,
            audio_weight,
            cross_max_similarity_threshold,
        }
    }

    // pub fn compare(
    //     &self,
    //     a: &VideoFeatures,
    //     b: &VideoFeatures,
    // ) -> anyhow::Result<VideoSimilarityResult> {
    //     // Use the same cross-max logic for image and audio.
    //     // This is robust to opening cuts, ending cuts, and timeline offsets.
    //     let image_sim = cross_max_similarity(
    //         &a.video_embeddings,
    //         &b.video_embeddings,
    //         self.cross_max_similarity_threshold,
    //     );
    //     let audio_sim = cross_max_similarity(
    //         &a.audio_embeddings,
    //         &b.audio_embeddings,
    //         self.cross_max_similarity_threshold,
    //     );
    //     let combined = self.image_weight * image_sim + self.audio_weight * audio_sim;

    //     Ok(VideoSimilarityResult {
    //         video_a: a.path.clone(),
    //         video_b: b.path.clone(),
    //         image_sim,
    //         audio_sim,
    //         combined_score: combined,
    //     })
    // }
}

// Similarity calculation.

/// Bidirectional max-cosine similarity.
///
/// Scores each vector in A against the closest vector in B, repeats the same
/// calculation from B to A, then averages both directions.
///
/// Inputs are expected to be L2-normalized, so dot product equals cosine
/// similarity.
///
/// This keeps scores stable when videos have opening/ending cuts, inserted
/// silence or dark frames, while unrelated videos stay low because every pair
/// has a low score.
pub fn cross_max_similarity(a: &[Vec<f32>], b: &[Vec<f32>], threshold: f32) -> f32 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let ab: Vec<f32> = a
        .iter()
        .map(|ea| {
            let best = b
                .iter()
                .map(|eb| dot(ea, eb))
                .fold(f32::NEG_INFINITY, f32::max);
            // Treat scores below the threshold as no match.
            if best >= threshold { best } else { 0.0 }
        })
        .collect();

    let ba: Vec<f32> = b
        .iter()
        .map(|eb| {
            let best = a
                .iter()
                .map(|ea| dot(ea, eb))
                .fold(f32::NEG_INFINITY, f32::max);
            if best >= threshold { best } else { 0.0 }
        })
        .collect();

    let total: f32 = ab.iter().chain(ba.iter()).sum();
    total / (ab.len() + ba.len()) as f32
}

#[inline]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
    use super::cross_max_similarity;

    #[test]
    fn cross_max_similarity_returns_zero_for_empty_inputs() {
        assert_eq!(cross_max_similarity(&[], &[vec![1.0, 0.0]], 0.5), 0.0);
        assert_eq!(cross_max_similarity(&[vec![1.0, 0.0]], &[], 0.5), 0.0);
    }

    #[test]
    fn cross_max_similarity_applies_threshold_to_best_matches() {
        let a = vec![vec![1.0, 0.0]];
        let b = vec![vec![0.8, 0.6]];

        assert_eq!(cross_max_similarity(&a, &b, 0.9), 0.0);
        assert!((cross_max_similarity(&a, &b, 0.8) - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn cross_max_similarity_averages_both_directions() {
        let a = vec![vec![1.0, 0.0]];
        let b = vec![vec![1.0, 0.0], vec![0.0, 1.0]];

        let score = cross_max_similarity(&a, &b, 0.5);

        assert!((score - (2.0 / 3.0)).abs() < f32::EPSILON);
    }
}
