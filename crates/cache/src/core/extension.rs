//! `MediaExtension` — 画像・動画・サムネイル用テーブルの定義。

use file_feature_cache::CacheExtension;
use rusqlite::Connection;

/// 画像・動画特化の拡張テーブルを定義する。
///
/// | テーブル | 内容 |
/// |---|---|
/// | `thumbnails` | サムネイルファイルのパス (1 ファイル : 1 サムネイル) |
/// | `image_features` | CLIP ベクトル |
/// | `video_features` | CLIP ベクトル + wav2vec2 ベクトル (どちらも NULL 許容) |
///
/// すべて `files(id)` を外部キーとして参照し、`ON DELETE CASCADE` を設定する。
#[derive(Clone)]
pub struct MediaExtension;

impl CacheExtension for MediaExtension {
    fn migrate(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS thumbnails (
                id             INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
                thumbnail_path TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS image_features (
                id          INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
                clip_vector BLOB NOT NULL
            );

            -- clip_vector / wav2vec2_vector はどちらも NULL 許容
            -- 片方ずつ書き込む用途に対応するため
            CREATE TABLE IF NOT EXISTS video_features (
                id               INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
                clip_vector      BLOB,
                wav2vec2_vector  BLOB
            );
            ",
        )
    }
}
