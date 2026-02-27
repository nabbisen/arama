use std::path::Path;

use super::{
    file_fingerprint::FileFingerprint,
    hash_strategy::HashStrategy,
    mode::{Mode, effective_mode},
    util::{full_hash, partial_hash, read_mtime},
};

// ---------------------------------------------------------------------------
// 外部向け関数 (crate 内公開)
// ---------------------------------------------------------------------------

/// ファイルを読んでフィンガープリントを計算する (upsert 時に使用)。
pub(crate) fn compute(path: &Path, strategy: &HashStrategy) -> std::io::Result<FileFingerprint> {
    let meta = std::fs::metadata(path)?;
    let file_size = meta.len();

    match effective_mode(file_size, strategy) {
        Mode::Full => Ok(FileFingerprint {
            hash: full_hash(path)?,
            mtime_ns: None,
        }),
        Mode::Partial { partial_bytes } => Ok(FileFingerprint {
            hash: partial_hash(path, file_size, partial_bytes)?,
            mtime_ns: read_mtime(&meta),
        }),
    }
}

/// DB に保存された `(stored_hash, stored_mtime)` と現在のファイルを比較する (lookup 時に使用)。
///
/// - `stored_mtime` が Some かつ現在の mtime と一致する場合はハッシュ計算を省略して `true` を返す。
/// - それ以外はハッシュを計算して比較する。
pub(crate) fn matches_stored(
    stored_hash: &str,
    stored_mtime: Option<i64>,
    path: &Path,
    strategy: &HashStrategy,
) -> std::io::Result<bool> {
    let meta = std::fs::metadata(path)?;
    let file_size = meta.len();

    // mtime クイックチェック (大ファイルの部分ハッシュモードのみ有効)
    if let Some(s_mtime) = stored_mtime {
        if let Some(c_mtime) = read_mtime(&meta) {
            if s_mtime == c_mtime {
                // mtime が一致 → 内容も同一とみなしてハッシュ計算をスキップ
                return Ok(true);
            }
        }
        // mtime 不一致 → ハッシュで再検証 (fall through)
    }

    let current_hash = match effective_mode(file_size, strategy) {
        Mode::Full => full_hash(path)?,
        Mode::Partial { partial_bytes } => partial_hash(path, file_size, partial_bytes)?,
    };

    Ok(current_hash == stored_hash)
}
