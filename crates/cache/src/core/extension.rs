//! `MediaExtension` — 画像・動画・サムネイル用テーブルの定義。

use file_feature_cache::CacheExtension;
use rusqlite::Connection;

/// 画像・動画特化の拡張テーブルを定義する。
///
/// `file_feature_cache` エンジンが `files` テーブルを作成した後に
/// [`migrate`] が呼ばれ、以下のテーブルを追加する。
///
/// - `thumbnails`     — サムネイルパス (1 ファイル : 1 サムネイル)
/// - `image_features` — CLIP ベクトル
/// - `video_features` — CLIP + wav2vec2 ベクトル
///
/// [`migrate`]: MediaExtension::migrate
#[derive(Clone)]
pub struct MediaExtension;

impl CacheExtension for MediaExtension {
    fn migrate(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS thumbnails (
                file_id        INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
                thumbnail_path TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS image_features (
                file_id     INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
                clip_vector BLOB    NOT NULL
            );

            CREATE TABLE IF NOT EXISTS video_features (
                file_id          INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
                clip_vector      BLOB    NOT NULL,
                wav2vec2_vector  BLOB    NOT NULL
            );
        ",
        )
    }
}
