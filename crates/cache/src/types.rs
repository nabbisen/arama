// ---------------------------------------------------------------------------
// upsert リクエスト型
// ---------------------------------------------------------------------------

/// 画像ファイルのキャッシュ登録 / 更新リクエスト。
#[derive(Debug, Clone)]
pub struct UpsertImageRequest {
    /// 対象ファイルパス。内部で `canonicalize()` を施す。
    pub file_path: String,
    /// CLIP 特徴量ベクトル。`None` の場合は既存値を保持する (部分更新)。
    pub clip_vector: Option<Vec<f32>>,
}

/// 動画ファイルのキャッシュ登録 / 更新リクエスト。
#[derive(Debug, Clone)]
pub struct UpsertVideoRequest {
    /// 対象ファイルパス。内部で `canonicalize()` を施す。
    pub file_path: String,
    /// CLIP 特徴量ベクトル。`None` の場合は既存値を保持する (部分更新)。
    pub clip_vector: Option<Vec<f32>>,
    /// wav2vec2 特徴量ベクトル。`None` の場合は既存値を保持する (部分更新)。
    pub wav2vec2_vector: Option<Vec<f32>>,
}

// ---------------------------------------------------------------------------
// 照会結果型
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ImageFeatures {
    pub clip_vector: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct VideoFeatures {
    pub clip_vector: Vec<f32>,
    pub wav2vec2_vector: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct ImageCacheEntry {
    /// canonicalize 済みファイルパス
    pub file_path: String,
    /// canonicalize 済みサムネイルパス。サムネイルなしの場合 `None`
    pub thumbnail_path: Option<String>,
    pub features: Option<ImageFeatures>,
}

#[derive(Debug, Clone)]
pub struct VideoCacheEntry {
    /// canonicalize 済みファイルパス
    pub file_path: String,
    /// canonicalize 済みサムネイルパス。サムネイルなしの場合 `None`
    pub thumbnail_path: Option<String>,
    pub features: Option<VideoFeatures>,
}

/// キャッシュ照会の結果
#[derive(Debug)]
pub enum LookupResult<T> {
    /// ファイルが一致し、キャッシュエントリが存在する
    Hit(T),
    /// DB にレコードはあるが、ファイルが変更されていた (古いデータは削除済み)
    Invalidated,
    /// DB にレコード自体が存在しない
    Miss,
}
