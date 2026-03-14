//! ai-cache で使用する型定義。

use std::path::PathBuf;

/// キャッシュ照会の結果。
#[derive(Debug)]
pub enum LookupResult<T> {
    /// キャッシュヒット。
    Hit(T),
    /// ファイルが変更されていた。古いキャッシュは削除済み。
    Invalidated,
    /// キャッシュに登録されていない。
    Miss,
}

// ---------------------------------------------------------------------------
// 画像
// ---------------------------------------------------------------------------

/// 画像キャッシュへの書き込みリクエスト。
#[derive(Debug)]
pub struct UpsertImageRequest {
    pub path: PathBuf,
    /// CLIP 特徴量ベクトル (1 枚 → 1 ベクトル)。`None` の場合は書き込まない。
    pub clip_vector: Option<Vec<f32>>,
}

/// 画像キャッシュエントリ。
#[derive(Debug)]
pub struct ImageCacheEntry {
    /// DB に保存された正規化済みパス。
    pub path: String,
    /// サムネイルファイルのパス。未生成の場合は `None`。
    pub thumbnail_path: Option<String>,
    /// 特徴量。未登録の場合は `None`。
    pub features: Option<ImageFeatures>,
}

#[derive(Debug, PartialEq)]
pub struct ImageFeatures {
    pub clip_vector: Vec<f32>,
}

// ---------------------------------------------------------------------------
// 動画
// ---------------------------------------------------------------------------

/// 動画キャッシュへの書き込みリクエスト。
#[derive(Debug)]
pub struct UpsertVideoRequest {
    pub path: PathBuf,
    /// コマ平均 CLIP 特徴量ベクトル。`None` の場合は既存の値を保持する。
    pub clip_vector: Option<Vec<f32>>,
    // /// シーン平均 wav2vec2 特徴量ベクトル。`None` の場合は既存の値を保持する。
    // pub wav2vec2_vector: Option<Vec<f32>>,
}

/// 動画キャッシュエントリ。
#[derive(Debug)]
pub struct VideoCacheEntry {
    pub path: String,
    pub thumbnail_path: Option<String>,
    pub features: Option<VideoFeatures>,
}

#[derive(Debug, PartialEq)]
pub struct VideoFeatures {
    /// コマ平均 CLIP 特徴量ベクトル。
    pub clip_vector: Option<Vec<f32>>,
    /// シーン平均 wav2vec2 特徴量ベクトル。
    pub wav2vec2_vector: Option<Vec<f32>>,
}
