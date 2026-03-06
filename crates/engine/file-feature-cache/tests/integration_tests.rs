//! `file_feature_cache` インテグレーションテスト。
//!
//! - 実ファイルを `tempfile` で生成して使う (ハッシュ計算・mtime 検証を実際に動かす)
//! - `NoExtension` で汎用エンジン単体の動作を検証する
//! - カスタム `CacheExtension` で拡張テーブルの migrate / CASCADE 削除を検証する

use std::io::Write;
use std::path::PathBuf;

use file_feature_cache::{
    CacheConfig, CacheExtension, CacheRead, CacheReader, CacheWrite, CacheWriter, DbLocation,
    NoExtension,
};

// ---------------------------------------------------------------------------
// テストヘルパー
// ---------------------------------------------------------------------------

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(content: &[u8]) -> Self {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content).unwrap();
        let (_, path) = f.keep().unwrap();
        TempFile { path }
    }
    fn path(&self) -> &std::path::Path {
        &self.path
    }
    fn overwrite(&self, content: &[u8]) {
        std::fs::write(&self.path, content).unwrap();
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

fn mem_writer() -> CacheWriter<NoExtension> {
    CacheWriter::open_in_memory().unwrap()
}

// ---------------------------------------------------------------------------
// カスタム拡張テーブル
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct ScoreExtension;

impl CacheExtension for ScoreExtension {
    fn migrate(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS scores (
                id      INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
                score   REAL NOT NULL
            );",
        )
    }
}

fn upsert_score(writer: &CacheWriter<ScoreExtension>, path: &std::path::Path, score: f64) {
    let id = writer.refresh(path).unwrap();
    let conn = writer.write_conn().unwrap();
    conn.execute(
        "INSERT INTO scores (id, score) VALUES (?1, ?2)
         ON CONFLICT(id) DO UPDATE SET score = excluded.score",
        rusqlite::params![id, score],
    )
    .unwrap();
}

fn fetch_score(writer: &CacheWriter<ScoreExtension>, path: &std::path::Path) -> Option<f64> {
    let conn = writer.read_conn().unwrap();
    conn.query_row(
        "SELECT s.score FROM scores s JOIN files f ON f.id = s.id WHERE f.path = ?1",
        [path.canonicalize().unwrap().to_str().unwrap()],
        |r| r.get::<_, f64>(0),
    )
    .ok()
}

// ===========================================================================
// refresh
// ===========================================================================

#[test]
fn refresh_registers_file_and_returns_id() {
    let writer = mem_writer();
    let f = TempFile::new(b"hello");
    let id = writer.refresh(f.path()).unwrap();
    assert!(id > 0);
}

#[test]
fn refresh_is_idempotent_for_unchanged_file() {
    let writer = mem_writer();
    let f = TempFile::new(b"same content");
    let id1 = writer.refresh(f.path()).unwrap();
    let id2 = writer.refresh(f.path()).unwrap();
    // 内容不変 → 同じ id が返る (DB 書き込みなし)
    assert_eq!(id1, id2);
}

#[test]
fn refresh_returns_new_id_after_content_change() {
    let writer = mem_writer();
    let f = TempFile::new(b"original");
    let id1 = writer.refresh(f.path()).unwrap();
    f.overwrite(b"modified");
    let id2 = writer.refresh(f.path()).unwrap();
    // 変更あり → 旧レコード削除 + 再 INSERT → 異なる id
    assert_ne!(id1, id2);
}

#[test]
fn refresh_returns_different_ids_for_different_files() {
    let writer = mem_writer();
    let f1 = TempFile::new(b"file A");
    let f2 = TempFile::new(b"file B");
    assert_ne!(
        writer.refresh(f1.path()).unwrap(),
        writer.refresh(f2.path()).unwrap()
    );
}

#[test]
fn refresh_on_nonexistent_file_returns_err() {
    let writer = mem_writer();
    assert!(
        writer
            .refresh(std::path::Path::new("/no/such/file"))
            .is_err()
    );
}

// ===========================================================================
// check
// ===========================================================================

#[test]
fn check_returns_true_after_refresh() {
    let writer = mem_writer();
    let f = TempFile::new(b"hello");
    writer.refresh(f.path()).unwrap();
    assert!(writer.check(f.path()).unwrap());
}

#[test]
fn check_returns_false_for_unregistered_file() {
    let writer = mem_writer();
    let f = TempFile::new(b"unregistered");
    assert!(!writer.check(f.path()).unwrap());
}

#[test]
fn check_returns_false_for_nonexistent_file() {
    // ファイル不在は canonicalize 失敗 → false (Err にならない)
    let writer = mem_writer();
    assert!(
        !writer
            .check(std::path::Path::new("/absolutely/no/such/file"))
            .unwrap()
    );
}

#[test]
fn check_returns_false_after_content_change_and_removes_record() {
    let writer = mem_writer();
    let f = TempFile::new(b"original");
    writer.refresh(f.path()).unwrap();
    f.overwrite(b"modified");
    assert!(!writer.check(f.path()).unwrap());
    assert!(
        !writer.list_paths().unwrap().contains(
            &f.path()
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        )
    );
}

#[test]
fn check_via_reader_detects_change_and_removes_record() {
    let writer = mem_writer();
    let f = TempFile::new(b"reader check test");
    writer.refresh(f.path()).unwrap();

    let reader = writer.as_reader();
    f.overwrite(b"changed");

    assert!(!reader.check(f.path()).unwrap());
    // writer 側からも消えている (Arc 共有)
    assert!(!writer.check(f.path()).unwrap());
}

// ===========================================================================
// delete
// ===========================================================================

#[test]
fn delete_returns_true_for_registered_file() {
    let writer = mem_writer();
    let f = TempFile::new(b"to delete");
    writer.refresh(f.path()).unwrap();
    assert!(writer.delete(f.path()).unwrap());
}

#[test]
fn delete_returns_false_for_unregistered_file() {
    let writer = mem_writer();
    let f = TempFile::new(b"not registered");
    // ファイルは存在するが DB に未登録
    assert!(!writer.delete(f.path()).unwrap());
}

#[test]
fn delete_on_nonexistent_file_returns_err() {
    let writer = mem_writer();
    // ファイルが存在しない → canonicalize 失敗 → Err
    assert!(
        writer
            .delete(std::path::Path::new("/no/such/file"))
            .is_err()
    );
}

#[test]
fn after_delete_check_returns_false() {
    let writer = mem_writer();
    let f = TempFile::new(b"delete me");
    writer.refresh(f.path()).unwrap();
    writer.delete(f.path()).unwrap();
    assert!(!writer.check(f.path()).unwrap());
}

// ===========================================================================
// list_paths
// ===========================================================================

#[test]
fn list_paths_empty_on_fresh_db() {
    assert!(mem_writer().list_paths().unwrap().is_empty());
}

#[test]
fn list_paths_returns_all_registered_sorted() {
    let writer = mem_writer();
    let files: Vec<_> = (0..5)
        .map(|i| TempFile::new(format!("f{i}").as_bytes()))
        .collect();
    for f in &files {
        writer.refresh(f.path()).unwrap();
    }
    let paths = writer.list_paths().unwrap();
    assert_eq!(paths.len(), 5);
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted);
}

#[test]
fn list_paths_decreases_after_delete() {
    let writer = mem_writer();
    let f1 = TempFile::new(b"a");
    let f2 = TempFile::new(b"b");
    writer.refresh(f1.path()).unwrap();
    writer.refresh(f2.path()).unwrap();
    writer.delete(f1.path()).unwrap();
    assert_eq!(writer.list_paths().unwrap().len(), 1);
}

// ===========================================================================
// as_reader — Arc 共有・権限分離
// ===========================================================================

#[test]
fn reader_sees_data_written_by_writer() {
    let writer = mem_writer();
    let f = TempFile::new(b"shared store test");
    writer.refresh(f.path()).unwrap();
    assert!(writer.as_reader().check(f.path()).unwrap());
}

#[test]
fn multiple_reader_clones_share_same_store() {
    let writer = mem_writer();
    let f = TempFile::new(b"clone test");
    writer.refresh(f.path()).unwrap();
    let r1 = writer.as_reader();
    let r2 = r1.clone();
    assert!(r1.check(f.path()).unwrap());
    assert!(r2.check(f.path()).unwrap());
}

// ===========================================================================
// CacheExtension — 拡張テーブルの migrate と CASCADE 削除
// ===========================================================================

#[test]
fn extension_migrate_creates_custom_table() {
    let writer = CacheWriter::<ScoreExtension>::open_in_memory().unwrap();
    let f = TempFile::new(b"scored file");
    upsert_score(&writer, f.path(), 0.95);
    assert_eq!(fetch_score(&writer, f.path()), Some(0.95));
}

#[test]
fn extension_score_updates_on_re_refresh() {
    let writer = CacheWriter::<ScoreExtension>::open_in_memory().unwrap();
    let f = TempFile::new(b"update score");
    upsert_score(&writer, f.path(), 0.5);
    upsert_score(&writer, f.path(), 0.99);
    assert_eq!(fetch_score(&writer, f.path()), Some(0.99));
}

#[test]
fn extension_cascade_delete_on_explicit_delete() {
    let writer = CacheWriter::<ScoreExtension>::open_in_memory().unwrap();
    let f = TempFile::new(b"cascade test");
    upsert_score(&writer, f.path(), 0.7);
    writer.delete(f.path()).unwrap();
    assert_eq!(fetch_score(&writer, f.path()), None);
}

#[test]
fn extension_cascade_delete_on_file_change_via_check() {
    let writer = CacheWriter::<ScoreExtension>::open_in_memory().unwrap();
    let f = TempFile::new(b"original");
    upsert_score(&writer, f.path(), 0.8);
    f.overwrite(b"modified");
    // check() が変更を検出 → files 削除 → scores も CASCADE 削除
    assert!(!writer.check(f.path()).unwrap());
    assert_eq!(fetch_score(&writer, f.path()), None);
}

#[test]
fn extension_cascade_delete_on_file_change_via_refresh() {
    let writer = CacheWriter::<ScoreExtension>::open_in_memory().unwrap();
    let f = TempFile::new(b"original");
    upsert_score(&writer, f.path(), 0.6);
    f.overwrite(b"modified");
    // refresh() が変更を検出 → 旧 files 削除 → scores も CASCADE 削除
    // 新しい id を返すので、新しい scores はまだ空
    let new_id = writer.refresh(f.path()).unwrap();
    assert!(new_id > 0);
    assert_eq!(fetch_score(&writer, f.path()), None);
}

// ===========================================================================
// onetime — ファイルベース DB への永続化
// ===========================================================================

#[test]
fn onetime_data_persists_across_instances() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let loc = DbLocation::Custom(db.path().to_path_buf());
    let file = TempFile::new(b"persist test");

    CacheWriter::<NoExtension>::onetime(loc.clone())
        .unwrap()
        .refresh(file.path())
        .unwrap();

    assert!(
        CacheWriter::<NoExtension>::onetime(loc)
            .unwrap()
            .check(file.path())
            .unwrap()
    );
}

#[test]
fn reader_onetime_reads_persisted_data() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let loc = DbLocation::Custom(db.path().to_path_buf());
    let file = TempFile::new(b"reader onetime");

    CacheWriter::<NoExtension>::onetime(loc.clone())
        .unwrap()
        .refresh(file.path())
        .unwrap();

    assert!(
        CacheReader::<NoExtension>::onetime(loc)
            .unwrap()
            .check(file.path())
            .unwrap()
    );
}

// ===========================================================================
// as_session — CacheConfig の各フィールド
// ===========================================================================

#[test]
fn as_session_with_thumbnail_dir_sets_path() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let thumb_dir = tempfile::TempDir::new().unwrap();
    let writer = CacheWriter::<NoExtension>::as_session(CacheConfig {
        db_location: DbLocation::Custom(db.path().to_path_buf()),
        read_conns: 1,
        thumbnail_dir: Some(thumb_dir.path().to_path_buf()),
    })
    .unwrap();

    let file = TempFile::new(b"thumb test");
    let id = writer.refresh(file.path()).unwrap();
    let path = writer.thumbnail_path(id, "jpg").unwrap();

    assert!(path.starts_with(thumb_dir.path()));
    assert_eq!(path.extension().and_then(|e| e.to_str()), Some("jpg"));
}

#[test]
fn thumbnail_path_returns_none_when_dir_not_configured() {
    let writer = mem_writer();
    let file = TempFile::new(b"no thumb dir");
    let id = writer.refresh(file.path()).unwrap();
    assert!(writer.thumbnail_path(id, "jpg").is_none());
}

// ===========================================================================
// CacheWrite / CacheRead trait 経由の呼び出し
// ===========================================================================

#[test]
fn cache_write_trait_methods_work() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let writer = CacheWriter::<NoExtension>::as_session(CacheConfig {
        db_location: DbLocation::Custom(db.path().to_path_buf()),
        read_conns: 2,
        thumbnail_dir: None,
    })
    .unwrap();
    let file = TempFile::new(b"trait test");
    writer.refresh(file.path()).unwrap();

    let _ = writer.list_paths().unwrap();
    let _ = writer.delete(file.path()).unwrap();
}

#[test]
fn cache_read_trait_check_via_reader() {
    let writer = mem_writer();
    let file = TempFile::new(b"read trait test");
    writer.refresh(file.path()).unwrap();

    let reader: &dyn CacheRead = &writer.as_reader();
    assert!(reader.check(file.path()).unwrap());
    assert_eq!(reader.list_paths().unwrap().len(), 1);
}

// ===========================================================================
// スレッド並列 check
// ===========================================================================

#[test]
fn parallel_check_with_threads() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let writer = CacheWriter::<NoExtension>::as_session(CacheConfig {
        db_location: DbLocation::Custom(db.path().to_path_buf()),
        read_conns: 8,
        thumbnail_dir: None,
    })
    .unwrap();

    let files: Vec<_> = (0..16)
        .map(|i| TempFile::new(format!("parallel {i}").as_bytes()))
        .collect();
    for f in &files {
        writer.refresh(f.path()).unwrap();
    }

    let reader = writer.as_reader();
    let results = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

    std::thread::scope(|s| {
        let handles: Vec<_> = files
            .iter()
            .map(|f| {
                let r = reader.clone();
                let path = f.path.clone();
                let res = results.clone();
                s.spawn(move || {
                    let ok = r.check(&path).unwrap();
                    res.lock().unwrap().push(ok);
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
    });

    assert!(results.lock().unwrap().iter().all(|&ok| ok));
}

// ===========================================================================
// refresh_all
// ===========================================================================

#[test]
fn refresh_all_registers_all_files() {
    let writer = mem_writer();
    let files: Vec<_> = (0..5)
        .map(|i| TempFile::new(format!("batch {i}").as_bytes()))
        .collect();
    let paths: Vec<&std::path::Path> = files.iter().map(|f| f.path()).collect();

    let results = writer.refresh_all(&paths);

    assert_eq!(results.len(), 5);
    for (_, r) in &results {
        assert!(r.is_ok(), "{r:?}");
    }
    // 全て異なる id
    let ids: Vec<i64> = results.into_iter().map(|(_, r)| r.unwrap()).collect();
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(unique.len(), 5);
}

#[test]
fn refresh_all_partial_failure_continues() {
    let writer = mem_writer();
    let good = TempFile::new(b"good file");
    let bad = std::path::Path::new("/no/such/file");

    let results = writer.refresh_all(&[good.path(), bad]);

    assert_eq!(results.len(), 2);
    assert!(results[0].1.is_ok());
    assert!(results[1].1.is_err());
}

#[test]
fn refresh_all_is_idempotent_for_unchanged_files() {
    let writer = mem_writer();
    let files: Vec<_> = (0..3)
        .map(|i| TempFile::new(format!("idem {i}").as_bytes()))
        .collect();
    let paths: Vec<&std::path::Path> = files.iter().map(|f| f.path()).collect();

    let first: Vec<i64> = writer
        .refresh_all(&paths)
        .into_iter()
        .map(|(_, r)| r.unwrap())
        .collect();
    let second: Vec<i64> = writer
        .refresh_all(&paths)
        .into_iter()
        .map(|(_, r)| r.unwrap())
        .collect();
    assert_eq!(first, second);
}

#[test]
fn refresh_all_detects_file_change_and_returns_new_id() {
    let writer = mem_writer();
    let file = TempFile::new(b"original");
    let paths = [file.path()];

    let id1 = writer.refresh_all(&paths).remove(0).1.unwrap();
    file.overwrite(b"modified");
    let id2 = writer.refresh_all(&paths).remove(0).1.unwrap();
    assert_ne!(id1, id2);
}

// ===========================================================================
// check_all
// ===========================================================================

#[test]
fn check_all_returns_true_for_registered_files() {
    let writer = mem_writer();
    let files: Vec<_> = (0..4)
        .map(|i| TempFile::new(format!("ca {i}").as_bytes()))
        .collect();
    let paths: Vec<&std::path::Path> = files.iter().map(|f| f.path()).collect();
    for p in &paths {
        writer.refresh(p).unwrap();
    }

    let results = writer.check_all(&paths);
    assert!(results.iter().all(|(_, r)| *r.as_ref().unwrap()));
}

#[test]
fn check_all_returns_false_for_unregistered_files() {
    let writer = mem_writer();
    let files: Vec<_> = (0..3)
        .map(|i| TempFile::new(format!("unreg {i}").as_bytes()))
        .collect();
    let paths: Vec<&std::path::Path> = files.iter().map(|f| f.path()).collect();

    let results = writer.check_all(&paths);
    assert!(results.iter().all(|(_, r)| !r.as_ref().unwrap()));
}

#[test]
fn check_all_mixed_registered_and_unregistered() {
    let writer = mem_writer();
    let reg = TempFile::new(b"registered");
    let unreg = TempFile::new(b"unregistered");
    writer.refresh(reg.path()).unwrap();

    let results = writer.check_all(&[reg.path(), unreg.path()]);
    assert_eq!(results.len(), 2);
    // reg.path が先なので results[0] が true, results[1] が false
    // ただし順序は paths スライスに従う
    let map: std::collections::HashMap<_, _> =
        results.into_iter().map(|(p, r)| (p, r.unwrap())).collect();
    assert!(map[&reg.path().to_path_buf()]);
    assert!(!map[&unreg.path().to_path_buf()]);
}

#[test]
fn check_all_via_reader() {
    let writer = mem_writer();
    let files: Vec<_> = (0..4)
        .map(|i| TempFile::new(format!("reader_ca {i}").as_bytes()))
        .collect();
    let paths: Vec<&std::path::Path> = files.iter().map(|f| f.path()).collect();
    for p in &paths {
        writer.refresh(p).unwrap();
    }

    let reader = writer.as_reader();
    let results = reader.check_all(&paths);
    assert!(results.iter().all(|(_, r)| *r.as_ref().unwrap()));
}
