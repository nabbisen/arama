// ---------------------------------------------------------------------------
// DB パス解決テスト
// ---------------------------------------------------------------------------

use crate::CacheConfig;

use super::*;

#[test]
fn resolve_falls_back_without_env_var() {
    let config = CacheConfig::default();
    unsafe {
        std::env::remove_var("arama_cache_DB");
    }
    let resolved = resolve_db_path(&config);
    let s = resolved.to_str().unwrap();
    assert!(
        s.contains("cache.db") || s.contains("arama_cache.db"),
        "unexpected fallback path: {s}"
    );
}
