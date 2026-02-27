// ---------------------------------------------------------------------------
// upsert リクエスト型
// ---------------------------------------------------------------------------

use r2d2_sqlite::SqliteConnectionManager;

/// 画像ファイルのキャッシュ登録 / 更新リクエスト。
#[derive(Debug, Clone)]
pub struct UpsertImageRequest {
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub clip_vector: Option<Vec<f32>>,
}

/// 動画ファイルのキャッシュ登録 / 更新リクエスト。
#[derive(Debug, Clone)]
pub struct UpsertVideoRequest {
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub clip_vector: Option<Vec<f32>>,
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
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub features: Option<ImageFeatures>,
}

#[derive(Debug, Clone)]
pub struct VideoCacheEntry {
    pub file_path: String,
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

// ---------------------------------------------------------------------------
// 型エイリアス
// ---------------------------------------------------------------------------

pub(crate) type ReadConn = r2d2::PooledConnection<SqliteConnectionManager>;
pub(crate) type WriteConn = r2d2::PooledConnection<SqliteConnectionManager>;
