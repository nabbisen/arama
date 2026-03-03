//! ファイル同一性確認ロジック。
//!
//! # 戦略の選択基準
//!
//! | ファイルサイズ | ハッシュ種別 | mtime の扱い |
//! |---|---|---|
//! | 閾値未満 (デフォルト 4 MB) | 完全 SHA-256 | 保存しない |
//! | 閾値以上 | 先頭 + 末尾の部分 SHA-256 | クイックフィルタとして保存 |
//!
//! ## 大ファイルの lookup フロー
//!
//! ```text
//! stored_mtime == current_mtime ──→ Hash 計算スキップ → Hit fast path
//!                ↓ 不一致
//!          部分 Hash を再計算
//!          stored_hash == new_hash ──→ Hit (内容は同じ、mtime だけ更新)
//!                         ↓ 不一致
//!                      Invalidated
//! ```

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::time::SystemTime;

use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------
// 公開設定型
// ---------------------------------------------------------------------------

/// ファイル同一性確認の戦略。`CacheConfig` に組み込んで使う。
#[derive(Debug, Clone)]
pub(crate) enum HashStrategy {
    /// 常にファイル全体の SHA-256 を使う。
    /// 小ファイルや変更検出の精度を最優先する場合に適する。
    Full,

    /// ファイルサイズで Full と Partial を自動選択する (デフォルト)。
    ///
    /// - `threshold_bytes` 未満 → `Full` と同等
    /// - `threshold_bytes` 以上 → 先頭 + 末尾 `partial_bytes` の部分 SHA-256。
    ///   mtime をクイックフィルタとして DB に保存し、一致すれば Hash 計算を省略する。
    SizeAdaptive {
        /// 部分ハッシュへ切り替えるファイルサイズの閾値 (bytes)。
        threshold_bytes: u64,
        /// 先頭・末尾それぞれから読み取る bytes 数。
        partial_bytes: usize,
    },
}

impl Default for HashStrategy {
    fn default() -> Self {
        Self::SizeAdaptive {
            threshold_bytes: 4 * 1024 * 1024, // 4 MB
            partial_bytes: 64 * 1024,         // 64 KB × 2
        }
    }
}

// ---------------------------------------------------------------------------
// 内部型
// ---------------------------------------------------------------------------

/// 計算済みフィンガープリント。DB への保存値と同形。
#[derive(Debug)]
pub(crate) struct FileFingerprint {
    /// SHA-256 の hex 文字列 (完全 or 部分)
    pub hash: String,
    /// `SizeAdaptive` で大ファイルと判定された場合のみ Some。
    /// `Full` または小ファイルでは None。
    pub mtime_ns: Option<i64>,
}

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

// ---------------------------------------------------------------------------
// 内部ユーティリティ
// ---------------------------------------------------------------------------

enum Mode {
    Full,
    Partial { partial_bytes: usize },
}

fn effective_mode(file_size: u64, strategy: &HashStrategy) -> Mode {
    match strategy {
        HashStrategy::Full => Mode::Full,
        HashStrategy::SizeAdaptive {
            threshold_bytes,
            partial_bytes,
        } => {
            if file_size < *threshold_bytes {
                Mode::Full
            } else {
                Mode::Partial {
                    partial_bytes: *partial_bytes,
                }
            }
        }
    }
}

fn full_hash(path: &Path) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(256 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn partial_hash(path: &Path, file_size: u64, partial_bytes: usize) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();

    // ファイルサイズ自体もハッシュに含める
    // → 異なるサイズのファイルで先頭/末尾の内容が偶然一致する誤検知を防ぐ
    hasher.update(file_size.to_le_bytes());

    // 先頭ブロック
    let head_len = (partial_bytes as u64).min(file_size) as usize;
    let mut buf = vec![0u8; head_len];
    file.read_exact(&mut buf)?;
    hasher.update(&buf);

    // 末尾ブロック (先頭と重複しない範囲のみ)
    let tail_offset = file_size.saturating_sub(partial_bytes as u64);
    if tail_offset > head_len as u64 {
        file.seek(SeekFrom::Start(tail_offset))?;
        let tail_len = (file_size - tail_offset) as usize;
        buf.resize(tail_len, 0);
        file.read_exact(&mut buf)?;
        hasher.update(&buf);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn read_mtime(meta: &std::fs::Metadata) -> Option<i64> {
    let t = meta.modified().ok()?;
    match t.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => i64::try_from(d.as_nanos()).ok(),
        // 1970 以前のタイムスタンプは符号付きで保存 (実用上ほぼ発生しない)
        Err(e) => i64::try_from(e.duration().as_nanos()).ok().map(|n| -n),
    }
}
