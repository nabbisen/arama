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
    //     // 映像・音声とも同じ cross-max ロジックで計算する
    //     // → 冒頭カット・末尾カット・時間ズレに頑健
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

// ─── 類似度計算 ────────────────────────────────────────────────────────

/// 双方向 max-cosine 類似度
///
/// A の各ベクトルに最も近い B のベクトルを探してスコア化し、
/// B→A 方向も同様に計算して平均を取る。
///
/// L2 正規化済みベクトルを前提とするため dot 積 = cosine 類似度。
///
/// この方式により:
///   - 冒頭・末尾カットによるタイムスタンプのズレに対応できる
///   - 無音・暗転の挿入があっても高スコアを維持できる
///   - 完全に別内容の動画には全ペアのスコアが低くなり正しく低スコアになる
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
            // 閾値未満は 0.0 として扱う（マッチなし）
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
