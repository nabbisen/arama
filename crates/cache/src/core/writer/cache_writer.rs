use std::sync::Arc;

use crate::CacheConfig;
use crate::core::reader::cache_reader::CacheReader;
use crate::core::store::cache_store::CacheStore;
use crate::core::store::path::resolve_db_path;
use crate::error::Result;

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
    /// デフォルト設定で DB を開く。
    ///
    /// DB パスは自動解決される (詳細は [`CacheConfig`] を参照)。
    /// 設定を変えたい場合は [`open_with_config`] を使う。
    ///
    /// [`open_with_config`]: CacheWriter::open_with_config
    pub fn open() -> Result<Self> {
        Self::open_with_config(CacheConfig::default())
    }

    /// カスタム設定で DB を開く。
    ///
    /// DB パスは `config.db_path` → 環境変数 `arama_cache_DB` → XDG → フォールバックの順で解決する。
    pub fn open_with_config(config: CacheConfig) -> Result<Self> {
        let path = resolve_db_path(&config); // 【変更】引数追加
        let inner = Arc::new(CacheStore::open(&path, config)?);
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
