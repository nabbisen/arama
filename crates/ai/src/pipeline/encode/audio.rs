//! 音声エンコーダの共通インターフェース
mod mel_filters;
pub mod whisper_encoder;

use crate::pipeline::extract::video_extractor::audio_segment::AudioSegmentView;

/// 音声エンコーダのトレイト
///
/// セグメントごとの埋め込みベクトル列を返す（1 本に潰さない）。
/// 呼び出し側で cross-max similarity を計算することで、
/// 冒頭カット・末尾カット・時間ズレに頑健な比較ができる。
pub trait AudioEncoder: Send + Sync {
    /// 各セグメントを独立してエンコードしてベクトル列を返す
    ///
    /// 戻り値: [N_segments × feature_dim]（各ベクトルは L2 正規化済み）
    fn encode_segments(&self, segments: &[AudioSegmentView<'_>]) -> Vec<Vec<f32>>;

    /// 出力ベクトルの次元数
    fn feature_dim(&self) -> usize;

    /// ffmpeg に要求するサンプルレート（Hz）
    fn required_sample_rate(&self) -> u32;
}
