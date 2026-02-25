//! 音声前処理ユーティリティ
//!
//! Whisper の前処理で使うメルフィルタバンク構築と周波数変換。
//! エンコーダの実装は whisper_encoder.rs に集約している。

/// 三角メルフィルタバンクの重み行列を構築する
///
/// 戻り値: [n_mels × n_bins] の行優先フラット配列
/// n_bins = fft_size / 2 + 1
pub fn build_mel_filterbank(
    sr: u32,
    fft_size: usize,
    n_mels: usize,
    f_min: f32,
    f_max: f32,
) -> Vec<f32> {
    let n_bins = fft_size / 2 + 1;
    let mel_min = hz_to_mel(f_min);
    let mel_max = hz_to_mel(f_max);

    let mel_pts: Vec<f32> = (0..=n_mels + 1)
        .map(|i| mel_min + (mel_max - mel_min) * i as f32 / (n_mels + 1) as f32)
        .collect();

    let bins: Vec<usize> = mel_pts
        .iter()
        .map(|&m| ((mel_to_hz(m) * (fft_size + 1) as f32 / sr as f32) as usize).min(n_bins - 1))
        .collect();

    // 三角フィルタ重みを事前計算（ゼロ除算ガード付き）
    let mut weights = vec![0.0f32; n_mels * n_bins];
    for m in 0..n_mels {
        let lo = bins[m];
        let mid = bins[m + 1];
        let hi = bins[m + 2];
        let up = (mid as isize - lo as isize).max(1) as f32;
        let down = (hi as isize - mid as isize).max(1) as f32;
        for k in lo..=mid {
            weights[m * n_bins + k] = (k - lo) as f32 / up;
        }
        for k in mid..=hi.min(n_bins - 1) {
            weights[m * n_bins + k] = (hi - k) as f32 / down;
        }
    }
    weights
}

#[inline]
pub fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

#[inline]
pub fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10.0_f32.powf(mel / 2595.0) - 1.0)
}
