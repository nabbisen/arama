//! `file_feature_cache` インテグレーションテスト。
//!
//! # テスト方針
//!
//! - 実ファイルを `tempfile` で生成して使う (ハッシュ計算・mtime 検証を実際に動かす)
//! - `NoExtension` で汎用エンジン単体の動作を検証する
//! - カスタム `CacheExtension` で拡張テーブルの migrate / CASCADE 削除を検証する
//! - `CacheWrite` / `CacheRead` trait 経由の呼び出しでポリモーフィズムを確認する

use std::io::Write;
use std::path::PathBuf;

use file_feature_cache::{
    CacheConfig, CacheExtension, CacheRead, CacheReader, CacheWrite, CacheWriter, DbLocation,
    NoExtension,
};

// ---------------------------------------------------------------------------
// テストヘルパー
// ---------------------------------------------------------------------------

/// 実ファイルを作成し、Drop 時に自動削除するガード。
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

    fn path_str(&self) -> &str {
        self.path.to_str().unwrap()
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

/// `NoExtension` を使うインメモリ Writer を生成する。
fn mem_writer() -> CacheWriter<NoExtension> {
    CacheWriter::open_in_memory().unwrap()
}

// ---------------------------------------------------------------------------
// カスタム拡張テーブルのセットアップ
// ---------------------------------------------------------------------------

/// テスト用の拡張: `scores` テーブルを追加する。
#[derive(Clone)]
struct ScoreExtension;

impl CacheExtension for ScoreExtension {
    fn migrate(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS scores (
                file_id INTEGER PRIMARY KEY REFERENCES files(id) ON DELETE CASCADE,
                score   REAL    NOT NULL
            );
        ",
        )
    }
}

fn upsert_with_score(writer: &CacheWriter<ScoreExtension>, path: &str, score: f64) {
    let file_id = writer.upsert_file(path).unwrap();
    let conn = writer.write_conn().unwrap();
    conn.execute(
        "INSERT INTO scores (file_id, score) VALUES (?1, ?2)
         ON CONFLICT(file_id) DO UPDATE SET score = excluded.score",
        rusqlite::params![file_id, score],
    )
    .unwrap();
}

fn fetch_score(writer: &CacheWriter<ScoreExtension>, path: &str) -> Option<f64> {
    let conn = writer.read_conn().unwrap();
    conn.query_row(
        "SELECT s.score FROM scores s
         JOIN files f ON f.id = s.file_id
         WHERE f.file_path = ?1",
        [path],
        |r| r.get::<_, f64>(0),
    )
    .ok()
}

// ===========================================================================
// upsert_file / check
// ===========================================================================

#[test]
fn upsert_then_check_returns_true() {
    let writer = mem_writer();
    let f = TempFile::new(b"hello");
    writer.upsert_file(f.path_str()).unwrap();
    assert!(writer.check(f.path_str()).unwrap());
}

#[test]
fn check_on_unknown_path_returns_false() {
    let writer = mem_writer();
    assert!(!writer.check("/no/such/file").unwrap());
}

#[test]
fn upsert_is_idempotent() {
    let writer = mem_writer();
    let f = TempFile::new(b"same content");
    let id1 = writer.upsert_file(f.path_str()).unwrap();
    let id2 = writer.upsert_file(f.path_str()).unwrap();
    // 同じファイルパスは同じ id になる (UPSERT)
    assert_eq!(id1, id2);
}

#[test]
fn upsert_returns_new_id_for_different_files() {
    let writer = mem_writer();
    let f1 = TempFile::new(b"file A");
    let f2 = TempFile::new(b"file B");
    let id1 = writer.upsert_file(f1.path_str()).unwrap();
    let id2 = writer.upsert_file(f2.path_str()).unwrap();
    assert_ne!(id1, id2);
}

// ===========================================================================
// ファイル変更検出 (check / verify_or_invalidate)
// ===========================================================================

#[test]
fn check_returns_false_after_content_change() {
    let writer = mem_writer();
    let f = TempFile::new(b"original");
    writer.upsert_file(f.path_str()).unwrap();

    f.overwrite(b"modified");

    // 変更検出 → 内部で自動削除 → false
    assert!(!writer.check(f.path_str()).unwrap());
    // 削除済みなので list_paths にも出ない
    assert!(
        !writer
            .list_paths()
            .unwrap()
            .contains(&f.path_str().to_string())
    );
}

#[test]
fn check_via_reader_also_detects_change_and_deletes() {
    let writer = mem_writer();
    let f = TempFile::new(b"reader check test");
    writer.upsert_file(f.path_str()).unwrap();

    let reader = writer.as_reader();
    f.overwrite(b"changed");

    assert!(!reader.check(f.path_str()).unwrap());
    // writer 側からも消えている (Arc 共有)
    assert!(!writer.check(f.path_str()).unwrap());
}

#[test]
fn verify_or_invalidate_returns_true_when_unchanged() {
    let writer = mem_writer();
    let f = TempFile::new(b"stable");
    writer.upsert_file(f.path_str()).unwrap();
    assert!(writer.verify_or_invalidate(f.path_str()).unwrap());
}

#[test]
fn verify_or_invalidate_returns_false_and_removes_on_change() {
    let writer = mem_writer();
    let f = TempFile::new(b"will change");
    writer.upsert_file(f.path_str()).unwrap();

    f.overwrite(b"changed content");
    assert!(!writer.verify_or_invalidate(f.path_str()).unwrap());
    assert!(
        !writer
            .list_paths()
            .unwrap()
            .contains(&f.path_str().to_string())
    );
}

#[test]
fn verify_or_invalidate_returns_true_for_unknown_path() {
    // 登録されていないパスは「無効化する対象がない」= true
    let writer = mem_writer();
    assert!(writer.verify_or_invalidate("/nonexistent/path").unwrap());
}

// ===========================================================================
// delete
// ===========================================================================

#[test]
fn delete_returns_true_for_existing_entry() {
    let writer = mem_writer();
    let f = TempFile::new(b"to delete");
    writer.upsert_file(f.path_str()).unwrap();
    assert!(writer.delete(f.path_str()).unwrap());
}

#[test]
fn delete_returns_false_for_unknown_path() {
    let writer = mem_writer();
    assert!(!writer.delete("/not/registered").unwrap());
}

#[test]
fn after_delete_check_returns_false() {
    let writer = mem_writer();
    let f = TempFile::new(b"delete me");
    writer.upsert_file(f.path_str()).unwrap();
    writer.delete(f.path_str()).unwrap();
    assert!(!writer.check(f.path_str()).unwrap());
}

// ===========================================================================
// list_paths
// ===========================================================================

#[test]
fn list_paths_empty_on_fresh_db() {
    let writer = mem_writer();
    assert!(writer.list_paths().unwrap().is_empty());
}

#[test]
fn list_paths_returns_all_registered_sorted() {
    let writer = mem_writer();
    let files: Vec<_> = (0..5)
        .map(|i| TempFile::new(format!("f{i}").as_bytes()))
        .collect();
    for f in &files {
        writer.upsert_file(f.path_str()).unwrap();
    }
    let paths = writer.list_paths().unwrap();
    assert_eq!(paths.len(), 5);
    // ソート済みであることを確認
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted);
}

#[test]
fn list_paths_decreases_after_delete() {
    let writer = mem_writer();
    let f1 = TempFile::new(b"a");
    let f2 = TempFile::new(b"b");
    writer.upsert_file(f1.path_str()).unwrap();
    writer.upsert_file(f2.path_str()).unwrap();
    writer.delete(f1.path_str()).unwrap();
    assert_eq!(writer.list_paths().unwrap().len(), 1);
}

// ===========================================================================
// as_reader — Arc 共有・権限分離
// ===========================================================================

#[test]
fn reader_sees_data_written_by_writer() {
    let writer = mem_writer();
    let f = TempFile::new(b"shared store test");
    writer.upsert_file(f.path_str()).unwrap();

    let reader = writer.as_reader();
    assert!(reader.check(f.path_str()).unwrap());
}

#[test]
fn multiple_reader_clones_share_same_store() {
    let writer = mem_writer();
    let f = TempFile::new(b"clone test");
    writer.upsert_file(f.path_str()).unwrap();

    let r1 = writer.as_reader();
    let r2 = r1.clone();
    let r3 = r2.clone();

    assert!(r1.check(f.path_str()).unwrap());
    assert!(r2.check(f.path_str()).unwrap());
    assert!(r3.check(f.path_str()).unwrap());
}

// ===========================================================================
// CacheExtension — 拡張テーブルの migrate と CASCADE 削除
// ===========================================================================

#[test]
fn extension_migrate_creates_custom_table() {
    let writer = CacheWriter::<ScoreExtension>::open_in_memory().unwrap();
    let f = TempFile::new(b"scored file");
    upsert_with_score(&writer, f.path_str(), 0.95);
    assert_eq!(fetch_score(&writer, f.path_str()), Some(0.95));
}

#[test]
fn extension_score_updates_on_re_upsert() {
    let writer = CacheWriter::<ScoreExtension>::open_in_memory().unwrap();
    let f = TempFile::new(b"update score");
    upsert_with_score(&writer, f.path_str(), 0.5);
    upsert_with_score(&writer, f.path_str(), 0.99);
    assert_eq!(fetch_score(&writer, f.path_str()), Some(0.99));
}

#[test]
fn extension_cascade_delete_removes_score() {
    let writer = CacheWriter::<ScoreExtension>::open_in_memory().unwrap();
    let f = TempFile::new(b"cascade test");
    upsert_with_score(&writer, f.path_str(), 0.7);
    writer.delete(f.path_str()).unwrap();
    assert_eq!(fetch_score(&writer, f.path_str()), None);
}

#[test]
fn extension_cascade_delete_on_file_change() {
    let writer = CacheWriter::<ScoreExtension>::open_in_memory().unwrap();
    let f = TempFile::new(b"original");
    upsert_with_score(&writer, f.path_str(), 0.8);

    f.overwrite(b"modified");
    // check() が変更を検出して files を削除 → scores も CASCADE 削除
    assert!(!writer.check(f.path_str()).unwrap());
    assert_eq!(fetch_score(&writer, f.path_str()), None);
}

// ===========================================================================
// oneshot — ファイルベース DB への永続化
// ===========================================================================

#[test]
fn oneshot_data_persists_across_instances() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let loc = DbLocation::Custom(db.path().to_path_buf());
    let f = TempFile::new(b"persist test");

    // 1 つ目のインスタンスで書き込む
    CacheWriter::<NoExtension>::oneshot(loc.clone(), None)
        .unwrap()
        .upsert_file(f.path_str())
        .unwrap();

    // 別インスタンスで読み取れる
    let found = CacheWriter::<NoExtension>::oneshot(loc, None)
        .unwrap()
        .check(f.path_str())
        .unwrap();
    assert!(found);
}

#[test]
fn reader_oneshot_reads_persisted_data() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let loc = DbLocation::Custom(db.path().to_path_buf());
    let f = TempFile::new(b"reader oneshot");

    CacheWriter::<NoExtension>::oneshot(loc.clone(), None)
        .unwrap()
        .upsert_file(f.path_str())
        .unwrap();

    let found = CacheReader::<NoExtension>::oneshot(loc)
        .unwrap()
        .check(f.path_str())
        .unwrap();
    assert!(found);
}

// ===========================================================================
// as_session — CacheConfig の各フィールド
// ===========================================================================

#[test]
fn as_session_with_explicit_read_conns() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let writer = CacheWriter::<NoExtension>::as_session(CacheConfig {
        db_location: DbLocation::Custom(db.path().to_path_buf()),
        read_conns: 4,
        thumbnail_dir: None,
    })
    .unwrap();
    let f = TempFile::new(b"session test");
    writer.upsert_file(f.path_str()).unwrap();
    assert!(writer.check(f.path_str()).unwrap());
}

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

    let f = TempFile::new(b"thumb test");
    let file_id = writer.upsert_file(f.path_str()).unwrap();
    let path = writer.thumbnail_path(file_id, "jpg").unwrap();

    assert!(path.starts_with(thumb_dir.path()));
    assert_eq!(path.extension().and_then(|e| e.to_str()), Some("jpg"));
}

#[test]
fn thumbnail_path_returns_none_when_dir_not_configured() {
    let writer = mem_writer();
    let f = TempFile::new(b"no thumb dir");
    let file_id = writer.upsert_file(f.path_str()).unwrap();
    assert!(writer.thumbnail_path(file_id, "jpg").is_none());
}

// ===========================================================================
// CacheWrite / CacheRead trait 経由の呼び出し
// ===========================================================================

/// trait オブジェクト経由で Writer を操作できることを確認する。
fn exercise_writer<W>(writer: &W, path: &str)
where
    W: CacheWrite,
    W::Reader: CacheRead,
{
    let file_id_result = CacheWriter::<NoExtension>::open_in_memory()
        .unwrap()
        .upsert_file(path);
    // trait 経由では upsert_file は呼べないが、delete / verify / list はできる
    let _ = writer.list_paths().unwrap();
    let _ = writer.verify_or_invalidate(path).unwrap();
    let _ = writer.delete(path).unwrap();
    drop(file_id_result);
}

#[test]
fn cache_write_trait_methods_work() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let writer = CacheWriter::<NoExtension>::as_session(CacheConfig {
        db_location: DbLocation::Custom(db.path().to_path_buf()),
        read_conns: 2,
        thumbnail_dir: None,
    })
    .unwrap();
    let f = TempFile::new(b"trait test");
    writer.upsert_file(f.path_str()).unwrap();
    exercise_writer(&writer, f.path_str());
}

#[test]
fn cache_read_trait_check_via_reader() {
    let writer = mem_writer();
    let f = TempFile::new(b"read trait test");
    writer.upsert_file(f.path_str()).unwrap();

    let reader: &dyn CacheRead = &writer.as_reader();
    assert!(reader.check(f.path_str()).unwrap());
    assert_eq!(reader.list_paths().unwrap().len(), 1);
}

// ===========================================================================
// rayon 並列 check
// ===========================================================================

#[test]
fn parallel_check_with_rayon() {
    use std::sync::Arc;

    let db = tempfile::NamedTempFile::new().unwrap();
    let writer = Arc::new(
        CacheWriter::<NoExtension>::as_session(CacheConfig {
            db_location: DbLocation::Custom(db.path().to_path_buf()),
            read_conns: 8,
            thumbnail_dir: None,
        })
        .unwrap(),
    );

    let files: Vec<_> = (0..16)
        .map(|i| TempFile::new(format!("parallel {i}").as_bytes()))
        .collect();
    for f in &files {
        writer.upsert_file(f.path_str()).unwrap();
    }

    let reader = writer.as_reader();
    // rayon の並列イテレータで全ファイルを check する
    let results: Vec<bool> = {
        use std::sync::Mutex;
        let acc = Mutex::new(Vec::new());
        std::thread::scope(|s| {
            let handles: Vec<_> = files
                .iter()
                .map(|f| {
                    let r = reader.clone();
                    let path = f.path_str().to_string();
                    s.spawn(move || r.check(&path).unwrap())
                })
                .collect();
            for h in handles {
                acc.lock().unwrap().push(h.join().unwrap());
            }
        });
        acc.into_inner().unwrap()
    };

    assert!(
        results.iter().all(|&ok| ok),
        "全スレッドで check が true であること"
    );
}

// ===========================================================================
// DbLocation のパス解決
// ===========================================================================

#[test]
fn db_location_custom_uses_given_path() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let loc = DbLocation::Custom(db.path().to_path_buf());
    // 指定パスに DB が開けること (エラーなし)
    CacheWriter::<NoExtension>::oneshot(loc, None).unwrap();
}

#[test]
fn db_location_workdir_custom_filename() {
    // WorkDir(Some("test_cache.db")) は ./test_cache.db に作成される
    // テスト後に削除
    let loc = DbLocation::WorkDir(Some("__test_workdir_cache.db".to_string()));
    let result = CacheWriter::<NoExtension>::oneshot(loc, None);
    let _ = std::fs::remove_file("./__test_workdir_cache.db");
    assert!(result.is_ok());
}
