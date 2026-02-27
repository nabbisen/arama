// ---------------------------------------------------------------------------
// DB パス解決テスト
// ---------------------------------------------------------------------------

#[test]
fn resolve_db_path_prefers_env_var() {
    let db = tempfile::NamedTempFile::new().unwrap();
    let db_path = db.path().to_str().unwrap().to_string();
    let (_, db_path_buf) = db.keep().unwrap();

    unsafe {
        std::env::set_var("ARAMA_CACHE_DB", &db_path);
    }
    let resolved = super::resolve_db_path();
    unsafe {
        std::env::remove_var("ARAMA_CACHE_DB");
    }
    let _ = std::fs::remove_file(&db_path_buf);

    assert_eq!(resolved.to_str().unwrap(), db_path);
}

#[test]
fn resolve_db_path_falls_back_without_env_var() {
    // 環境変数が未設定の場合、XDG or フォールバックパスが返る (存在確認はしない)
    unsafe {
        std::env::remove_var("ARAMA_CACHE_DB");
    }
    let resolved = super::resolve_db_path();
    // パスに "arama_cache" と "cache.db" が含まれるか、フォールバックの "./arama_cache.db" になる
    let s = resolved.to_str().unwrap();
    assert!(
        s.contains("cache.db") || s.contains("arama_cache.db"),
        "unexpected fallback path: {s}"
    );
}
