//! `CacheStore` — コネクションプール・設定・DB ヘルパーを一元管理するコア。

use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{OpenFlags, OptionalExtension};

use crate::core::extension::CacheExtension;
use crate::core::identity::{FileFingerprint, HashStrategy, compute, matches_stored};
use crate::error::{CacheError, Result};
use crate::schema::initialize;

// ---------------------------------------------------------------------------
// DbLocation
// ---------------------------------------------------------------------------

/// DB ファイルの場所を指定する。
#[derive(Debug, Clone)]
pub enum DbLocation {
    /// パスを完全に指定する。
    Custom(PathBuf),

    /// XDG キャッシュディレクトリ (`$XDG_CACHE_HOME/<app>/<name>`) を使う。
    ///
    /// `name` が `None` の場合は `cache.db`。
    /// アプリ名は実行ファイル名から自動取得する。
    AppCache(Option<String>),

    /// カレントディレクトリ (`./<name>`) に作成する。
    ///
    /// `name` が `None` の場合は `cache.db`。
    WorkDir(Option<String>),
}

impl Default for DbLocation {
    fn default() -> Self {
        Self::WorkDir(None)
    }
}

impl DbLocation {
    pub fn resolve(&self) -> PathBuf {
        match self {
            Self::Custom(p) => p.clone(),

            Self::AppCache(name) => {
                let base = std::env::var("XDG_CACHE_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| {
                        std::env::var("HOME")
                            .map(|h| PathBuf::from(h).join(".cache"))
                            .unwrap_or_else(|_| PathBuf::from(".cache"))
                    });
                let app = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
                    .unwrap_or_else(|| "app".to_string());
                base.join(app).join(name.as_deref().unwrap_or("cache.db"))
            }

            Self::WorkDir(name) => {
                PathBuf::from(format!("./{}", name.as_deref().unwrap_or("cache.db")))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CacheConfig
// ---------------------------------------------------------------------------

/// セッション開始時に渡す設定。
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub db_location: DbLocation,
    /// 読み取りプールのコネクション数。rayon 等の並列スレッド数に合わせて設定する。
    pub read_conns: u32,
    /// サムネイルファイルを格納するディレクトリ。`None` の場合は管理しない。
    pub thumbnail_dir: Option<PathBuf>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            db_location: DbLocation::default(),
            read_conns: num_cpus(),
            thumbnail_dir: None,
        }
    }
}

// ---------------------------------------------------------------------------
// 型エイリアス
// ---------------------------------------------------------------------------

pub(crate) type ReadConn = r2d2::PooledConnection<SqliteConnectionManager>;
pub(crate) type WriteConn = r2d2::PooledConnection<SqliteConnectionManager>;

// ---------------------------------------------------------------------------
// CacheStore
// ---------------------------------------------------------------------------

pub struct CacheStore<E: CacheExtension> {
    read_pool: Pool<SqliteConnectionManager>,
    write_pool: Pool<SqliteConnectionManager>,
    pub config: CacheConfig,
    hash_strategy: HashStrategy,
    _ext: PhantomData<E>,
}

impl<E: CacheExtension> CacheStore<E> {
    pub fn open(db_path: &Path, config: CacheConfig) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| CacheError::io(parent, e))?;
            }
        }

        let write_manager = SqliteConnectionManager::file(db_path)
            .with_flags(
                OpenFlags::SQLITE_OPEN_READ_WRITE
                    | OpenFlags::SQLITE_OPEN_CREATE
                    | OpenFlags::SQLITE_OPEN_URI,
            )
            .with_init(|c| {
                c.execute_batch(
                    "PRAGMA journal_mode = WAL;
                     PRAGMA foreign_keys = ON;
                     PRAGMA synchronous   = NORMAL;
                     PRAGMA busy_timeout  = 5000;",
                )
            });

        let write_pool = Pool::builder()
            .max_size(1)
            .build(write_manager)
            .map_err(CacheError::pool)?;

        {
            let conn = write_pool.get().map_err(CacheError::pool)?;
            initialize::<E>(&conn)?;
        }

        let read_manager = SqliteConnectionManager::file(db_path)
            .with_flags(OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI)
            .with_init(|c| {
                c.execute_batch(
                    "PRAGMA journal_mode = WAL;
                     PRAGMA foreign_keys = ON;",
                )
            });

        let read_pool = Pool::builder()
            .max_size(config.read_conns.max(1))
            .build(read_manager)
            .map_err(CacheError::pool)?;

        Ok(Self {
            read_pool,
            write_pool,
            hash_strategy: HashStrategy::default(),
            config,
            _ext: PhantomData,
        })
    }

    pub fn open_in_memory(config: CacheConfig) -> Result<Self> {
        let manager = SqliteConnectionManager::memory().with_init(|c| {
            c.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA foreign_keys = ON;",
            )
        });

        let pool = Pool::builder()
            .max_size(1)
            .build(manager)
            .map_err(CacheError::pool)?;

        {
            let conn = pool.get().map_err(CacheError::pool)?;
            initialize::<E>(&conn)?;
        }

        Ok(Self {
            read_pool: pool.clone(),
            write_pool: pool,
            hash_strategy: HashStrategy::default(),
            config,
            _ext: PhantomData,
        })
    }

    pub fn read(&self) -> Result<ReadConn> {
        self.read_pool.get().map_err(CacheError::pool)
    }

    pub fn write(&self) -> Result<WriteConn> {
        self.write_pool.get().map_err(CacheError::pool)
    }
}

// ---------------------------------------------------------------------------
// canonical_str — DB キーへの変換 (crate 内で唯一の canonicalize 呼び出し箇所)
// ---------------------------------------------------------------------------

/// `path` を canonicalize して DB キーに使う文字列に変換する。
///
/// ファイルが存在しない場合は `Err(CacheError::Io)` を返す。
/// この関数は `file-feature-cache` 内で唯一の canonicalize 実施箇所である。
pub(crate) fn canonical_str(path: &Path) -> Result<String> {
    path.canonicalize()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| CacheError::io(path, e))
}

// ---------------------------------------------------------------------------
// DB ヘルパー関数
// ---------------------------------------------------------------------------

/// `files` テーブルから (id, file_hash, mtime_ns) を取得する。
pub(crate) fn db_lookup<E: CacheExtension>(
    store: &CacheStore<E>,
    key: &str,
) -> Result<Option<(i64, String, Option<i64>)>> {
    let conn = store.read()?;
    let row = conn
        .query_row(
            "SELECT id, file_hash, mtime_ns FROM files WHERE path = ?1",
            [key],
            |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<i64>>(2)?,
                ))
            },
        )
        .optional()?;
    Ok(row)
}

/// `files` テーブルに INSERT し、新しい `id` を返す。
pub(crate) fn db_insert<E: CacheExtension>(
    store: &CacheStore<E>,
    key: &str,
    hash: &str,
    mtime_ns: Option<i64>,
) -> Result<i64> {
    let conn = store.write()?;
    conn.execute(
        "INSERT INTO files (path, file_hash, mtime_ns)
         VALUES (?1, ?2, ?3)",
        rusqlite::params![key, hash, mtime_ns],
    )?;
    conn.query_row("SELECT id FROM files WHERE path = ?1", [key], |r| {
        r.get::<_, i64>(0)
    })
    .map_err(Into::into)
}

/// `files` テーブルを id で削除する。CASCADE で拡張テーブルも削除される。
pub(crate) fn db_delete_by_id<E: CacheExtension>(store: &CacheStore<E>, id: i64) -> Result<()> {
    let conn = store.write()?;
    conn.execute("DELETE FROM files WHERE id = ?1", [id])?;
    Ok(())
}

/// `files` テーブルを path (正規化済みキー) で削除する。
pub(crate) fn db_delete_by_key<E: CacheExtension>(
    store: &CacheStore<E>,
    key: &str,
) -> Result<bool> {
    let conn = store.write()?;
    let n = conn.execute("DELETE FROM files WHERE path = ?1", [key])?;
    Ok(n > 0)
}

// ---------------------------------------------------------------------------
// 共有ユーティリティ
// ---------------------------------------------------------------------------

/// 現在のファイルが DB の保存値と一致するか確認する。
pub(crate) fn file_matches<E: CacheExtension>(
    store: &CacheStore<E>,
    stored_hash: &str,
    stored_mtime: Option<i64>,
    path: &Path,
) -> Result<bool> {
    matches_stored(stored_hash, stored_mtime, path, &store.hash_strategy)
        .map_err(|e| CacheError::io(path, e))
}

/// ファイルの fingerprint を計算する。
pub(crate) fn compute_fingerprint<E: CacheExtension>(
    store: &CacheStore<E>,
    path: &Path,
) -> Result<FileFingerprint> {
    compute(path, &store.hash_strategy).map_err(|e| CacheError::io(path, e))
}

// ---------------------------------------------------------------------------
// 内部ユーティリティ
// ---------------------------------------------------------------------------

fn num_cpus() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(4)
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_location_custom_resolves_to_given_path() {
        let loc = DbLocation::Custom("/tmp/myapp/cache.db".into());
        assert_eq!(loc.resolve(), PathBuf::from("/tmp/myapp/cache.db"));
    }

    #[test]
    fn db_location_workdir_default_filename() {
        assert_eq!(
            DbLocation::WorkDir(None).resolve(),
            PathBuf::from("./cache.db")
        );
    }

    #[test]
    fn db_location_workdir_custom_filename() {
        assert_eq!(
            DbLocation::WorkDir(Some("inference.db".into())).resolve(),
            PathBuf::from("./inference.db"),
        );
    }

    #[test]
    fn db_location_appcache_ends_with_cache_db() {
        let s = DbLocation::AppCache(None).resolve();
        assert!(s.to_str().unwrap().ends_with("cache.db"));
    }
}
