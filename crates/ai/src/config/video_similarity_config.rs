//! 類似度判定の設定とサンプリングタイムスタンプ生成

use crate::{CLIP_IMAGE_SIZE, CROSS_MAX_SIMILARITY_THRESHOLD, VIDEO_IMAGE_WEIGHT};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct VideoSimilarityConfig {
    // ── 冒頭ゾーン ──────────────────────────────────────────────────
    pub head_fixed_anchors_secs: Vec<f64>,

    /// 冒頭ゾーンの最大長（秒）
    /// 実際の上限は min(head_zone_secs, duration * 0.5) で決まる
    pub head_zone_secs: f64,

    /// 冒頭ゾーン内のサンプル点数
    /// 総サンプル数の約半分に設定することで冒頭を厚めにカバーする
    pub head_sample_count: usize,

    // ── 中間・末尾 ───────────────────────────────────────────────────
    /// 動画全体に対するパーセンテージアンカー（0.0〜1.0）
    pub percent_anchors: Vec<f64>,

    /// 末尾固定アンカー（動画末尾からの秒数）
    pub tail_anchors_secs: Vec<f64>,

    // ── マージ ───────────────────────────────────────────────────────
    /// 近接するサンプル点を統合する最小間隔（秒）
    /// Whisper セグメント長の 2 倍程度が目安
    pub min_sample_gap_secs: f64,

    // ── 音声・映像 ──────────────────────────────────────────────────
    /// 1 音声セグメントの長さ（秒）
    pub audio_segment_duration_secs: f64,

    /// CLIP 入力画像サイズ（ピクセル）
    pub clip_image_size: usize,

    // ── 類似度重み ───────────────────────────────────────────────────
    pub image_weight: f32,
    pub audio_weight: f32,

    pub cross_max_similarity_threshold: f32,
}

impl Default for VideoSimilarityConfig {
    fn default() -> Self {
        Self {
            head_fixed_anchors_secs: vec![3.0, 9.0, 15.0],

            // 冒頭 135 秒に 5 点（約 27 秒間隔）
            // → 総サンプル 14 点中 8 点 = 57% が冒頭
            head_zone_secs: 135.0,
            head_sample_count: 5,

            // 中間 3 点（動画の長さに比例してスケール）
            percent_anchors: vec![0.30, 0.50, 0.70],

            // 末尾 3 点
            tail_anchors_secs: vec![30.0, 15.0, 5.0],

            // 20 秒以内の近接点は後を除去
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
    /// 動画の長さを受け取り、サンプリングタイムスタンプ一覧を生成する
    ///
    /// 処理手順:
    ///   1. 冒頭ゾーン内に head_sample_count 点を均等配置
    ///   2. パーセンテージアンカーを絶対秒数に変換
    ///   3. 末尾オフセットを絶対秒数に変換
    ///   4. ソート → 範囲外除去 → min_sample_gap_secs でマージ
    pub fn compute_sample_timestamps(&self, duration_secs: f64) -> Vec<f64> {
        let mut points: Vec<f64> = Vec::new();

        // 1. 冒頭固定アンカー（必ず含める）
        for &t in &self.head_fixed_anchors_secs {
            if t < duration_secs {
                points.push(t);
            }
        }

        // 2. 冒頭ゾーン均等サンプリング（固定アンカーの後の区間）
        let head_zone = self.head_zone_secs.min(duration_secs * 0.5);
        // 固定アンカーの最後の点以降から均等割り
        let head_start = self.head_fixed_anchors_secs.last().copied().unwrap_or(0.0);
        if head_zone > head_start {
            let step = (head_zone - head_start) / (self.head_sample_count + 1) as f64;
            for i in 1..=self.head_sample_count {
                points.push(head_start + step * i as f64);
            }
        }

        // 3. パーセンテージアンカー
        for &pct in &self.percent_anchors {
            points.push(duration_secs * pct);
        }

        // 4. 末尾固定アンカー
        for &offset in &self.tail_anchors_secs {
            let t = duration_secs - offset;
            if t > 0.0 {
                points.push(t);
            }
        }

        points.sort_by(|a, b| a.partial_cmp(b).unwrap());
        points.retain(|&t| t > 0.0 && t < duration_secs);

        // 固定アンカーは min_gap の対象外にする
        deduplicate_preserving_fixed(
            points,
            &self.head_fixed_anchors_secs,
            self.min_sample_gap_secs,
        )
    }

    /// サンプリング設定のサマリ文字列（ログ・デバッグ用）
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
            "duration={:.0}s  total={} [head={}, mid={}, tail={}]  → [{}]",
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
            // 固定アンカーは無条件で保持
            result.push(t);
        } else if result.last().map_or(true, |&last| t - last >= min_gap) {
            result.push(t);
        }
    }
    result
}
