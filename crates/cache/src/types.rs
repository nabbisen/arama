// ---------------------------------------------------------------------------
// キャッシュ登録・更新リクエスト
// ---------------------------------------------------------------------------

/// 画像ファイルのキャッシュ登録 / 更新リクエスト。
///
/// ファイル同一性確認 (hash / mtime) はキャッシュストアが内部で自動計算する。
/// `thumbnail_path` / `clip_vector` は `None` を渡すと既存値を上書きしない (部分更新)。
#[derive(Debug, Clone)]
pub struct UpsertImageRequest {
    /// キャッシュ対象ファイルのパス (一意キー かつ hash 計算の対象)
    pub file_path: String,
    /// サムネイル画像のパス
    pub thumbnail_path: Option<String>,
    /// CLIP 特徴量ベクトル
    pub clip_vector: Option<Vec<f32>>,
}

/// 動画ファイルのキャッシュ登録 / 更新リクエスト。
#[derive(Debug, Clone)]
pub struct UpsertVideoRequest {
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    /// フレーム画像群から算出した CLIP 特徴量
    pub clip_vector: Option<Vec<f32>>,
    /// 音声データから算出した wav2vec2 特徴量
    pub wav2vec2_vector: Option<Vec<f32>>,
}

// ---------------------------------------------------------------------------
// キャッシュ照会結果
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

/// `lookup_image` の返り値
#[derive(Debug, Clone)]
pub struct ImageCacheEntry {
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub features: Option<ImageFeatures>,
}

/// `lookup_video` の返り値
#[derive(Debug, Clone)]
pub struct VideoCacheEntry {
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub features: Option<VideoFeatures>,
}

/// キャッシュ照会・変更確認の総合結果
#[derive(Debug)]
pub enum LookupResult<T> {
    /// ファイルが一致し、キャッシュエントリが存在する
    Hit(T),
    /// DB にレコードはあるが、ファイルが変更されていた (古いデータは削除済み)
    Invalidated,
    /// DB にレコード自体が存在しない
    Miss,
}
