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

    /// Compare frame/audio mean vectors using RFC 018's partial-modality matrix.
    ///
    /// `None` or a no-signal vector means that modality is unavailable. When
    /// both entries share only one valid modality, that modality's cosine score
    /// is used directly. When both entries have both modalities, configured
    /// weights are applied. Entries with no shared valid modality are invalid.
    pub fn compare_mean_vectors(
        &self,
        left_image: Option<&[f32]>,
        left_audio: Option<&[f32]>,
        right_image: Option<&[f32]>,
        right_audio: Option<&[f32]>,
    ) -> Option<f32> {
        score_mean_vectors(
            left_image,
            left_audio,
            right_image,
            right_audio,
            self.image_weight,
            self.audio_weight,
        )
    }
}

pub fn score_mean_vectors(
    left_image: Option<&[f32]>,
    left_audio: Option<&[f32]>,
    right_image: Option<&[f32]>,
    right_audio: Option<&[f32]>,
    image_weight: f32,
    audio_weight: f32,
) -> Option<f32> {
    let left_image = valid_vector(left_image);
    let left_audio = valid_vector(left_audio);
    let right_image = valid_vector(right_image);
    let right_audio = valid_vector(right_audio);

    match (left_image, left_audio, right_image, right_audio) {
        (Some(li), Some(la), Some(ri), Some(ra)) => {
            Some(image_weight * dot(li, ri) + audio_weight * dot(la, ra))
        }
        (Some(li), _, Some(ri), _) => Some(dot(li, ri)),
        (_, Some(la), _, Some(ra)) => Some(dot(la, ra)),
        _ => None,
    }
}

pub fn has_signal(vector: &[f32]) -> bool {
    vector.iter().any(|value| *value != 0.0)
}

fn valid_vector(vector: Option<&[f32]>) -> Option<&[f32]> {
    vector.filter(|value| !value.is_empty() && has_signal(value))
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
    use super::{cross_max_similarity, score_mean_vectors};

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

    #[test]
    fn score_mean_vectors_uses_weighted_score_for_full_entries() {
        let left_image = [1.0, 0.0];
        let left_audio = [0.0, 1.0];
        let right_image = [0.5, 0.0];
        let right_audio = [0.0, 0.25];

        let score = score_mean_vectors(
            Some(&left_image),
            Some(&left_audio),
            Some(&right_image),
            Some(&right_audio),
            0.6,
            0.4,
        )
        .unwrap();

        assert!((score - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn score_mean_vectors_compares_image_only_matrix_rows() {
        let image_a = [1.0, 0.0];
        let image_b = [0.75, 0.0];
        let audio = [0.0, 1.0];

        assert_eq!(
            score_mean_vectors(Some(&image_a), None, Some(&image_b), None, 0.6, 0.4),
            Some(0.75)
        );
        assert_eq!(
            score_mean_vectors(Some(&image_a), None, Some(&image_b), Some(&audio), 0.6, 0.4),
            Some(0.75)
        );
        assert_eq!(
            score_mean_vectors(Some(&image_b), Some(&audio), Some(&image_a), None, 0.6, 0.4),
            Some(0.75)
        );
    }

    #[test]
    fn score_mean_vectors_compares_audio_only_matrix_rows() {
        let audio_a = [1.0, 0.0];
        let audio_b = [0.25, 0.0];
        let image = [0.0, 1.0];

        assert_eq!(
            score_mean_vectors(None, Some(&audio_a), None, Some(&audio_b), 0.6, 0.4),
            Some(0.25)
        );
        assert_eq!(
            score_mean_vectors(None, Some(&audio_a), Some(&image), Some(&audio_b), 0.6, 0.4),
            Some(0.25)
        );
        assert_eq!(
            score_mean_vectors(Some(&image), Some(&audio_b), None, Some(&audio_a), 0.6, 0.4),
            Some(0.25)
        );
    }

    #[test]
    fn score_mean_vectors_rejects_entries_without_shared_valid_modality() {
        let image = [1.0, 0.0];
        let audio = [0.0, 1.0];
        let zero = [0.0, 0.0];

        assert_eq!(
            score_mean_vectors(Some(&image), None, None, Some(&audio), 0.6, 0.4),
            None
        );
        assert_eq!(
            score_mean_vectors(Some(&zero), None, Some(&image), None, 0.6, 0.4),
            None
        );
        assert_eq!(
            score_mean_vectors(None, None, Some(&image), Some(&audio), 0.6, 0.4),
            None
        );
    }
}
