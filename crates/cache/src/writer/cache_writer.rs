use std::path::Path;
use std::sync::Arc;

use crate::CacheConfig;
use crate::error::Result;
use crate::reader::cache_reader::CacheReader;
use crate::store::cache_store::CacheStore;

/// 参照 + 更新の全権限を持つハンドル。
///
/// - lookup / list は [`CacheReader`] に委譲する (コードの重複なし)
/// - [`as_reader`] で参照専用の [`CacheReader`] にダウングレードできる
/// - [`CacheReader`] から `CacheWriter` への昇格はできない
///
/// `Clone` は `Arc<CacheStore>` のカウントアップのみで低コスト。
///
/// [`as_reader`]: CacheWriter::as_reader
#[derive(Clone)]
pub struct CacheWriter {
    pub(crate) reader: CacheReader,
}

impl CacheWriter {
    /// 指定パスの SQLite ファイルを開く (存在しない場合は新規作成)。
    pub fn open(path: impl AsRef<Path>, config: CacheConfig) -> Result<Self> {
        let inner = Arc::new(CacheStore::open(path.as_ref(), config)?);
        Ok(Self {
            reader: CacheReader::new(inner),
        })
    }

    /// インメモリ DB を開く (テスト用)。
    pub fn open_in_memory() -> Result<Self> {
        Self::open_in_memory_with_config(CacheConfig::default())
    }

    /// 設定付きインメモリ DB を開く (テスト用)。
    pub fn open_in_memory_with_config(config: CacheConfig) -> Result<Self> {
        let inner = Arc::new(CacheStore::open_in_memory(config)?);
        Ok(Self {
            reader: CacheReader::new(inner),
        })
    }

    /// 参照専用の [`CacheReader`] を返す。
    ///
    /// 内部の `Arc<CacheStore>` を共有するため、コスト・DB 接続ともに追加消費なし。
    /// lookup のみ必要な箇所に渡すことで権限を制限できる。
    pub fn as_reader(&self) -> CacheReader {
        self.reader.clone()
    }

    pub(crate) fn inner(&self) -> &Arc<CacheStore> {
        &self.reader.inner
    }
}
