//! RFC 033 Part B: proves a poisoned `ReadPool` surfaces as an error, never
//! as a cache miss. Crate-internal because the property requires a panic
//! while a specific `ReadPool` slot's guard is held, which is only reachable
//! through the private `read` field on `ImageCacheReader` — no public method
//! on the facade runs caller-supplied code while that guard is held, so the
//! external `crates/cache/tests/**` integration boundary cannot induce this.
//! See `rfcs/proposed/033-cache-dependency-and-rust-baseline.md` Part B and
//! `.git-exclude/reviewed/059-rfc033-cache-error-tier-scope-question-review.md`.

use localcache::LocalFileCacheError;

use super::*;
use crate::core::engine::CacheError;

#[test]
fn poisoned_read_pool_surfaces_as_error_not_miss() {
    let db = tempfile::NamedTempFile::new().expect("create temp db file");
    // `read_conns = 1` is load-bearing, not incidental: `ReadPool::checkout`
    // skips a poisoned slot exactly like a busy one during its `try_lock`
    // scan, and only reports `Poisoned` from the blocking fallback reached
    // when no slot remains to try. A multi-slot pool would not surface the
    // error deterministically.
    let writer = ImageCacheWriter::as_session(ImageCacheConfig {
        cache_config: CacheConfig {
            db_location: DbLocation::Custom(db.path().to_path_buf()),
            read_conns: 1,
            thumbnail_dir: None,
        },
    })
    .expect("open writer");
    let reader = writer.as_reader();

    // Poison the pool's one slot: panic while `query_run`'s closure holds
    // the checked-out guard, so it unwinds while locked.
    let pool = reader.read.clone();
    let panicked = std::thread::spawn(move || {
        let _ = pool.query_run(|_q| panic!("induced poison for RFC 033 Part B test"));
    })
    .join();
    assert!(
        panicked.is_err(),
        "expected the spawned thread to panic while holding the pool guard"
    );

    let result = reader.lookup(db.path());

    assert!(
        matches!(
            result,
            Err(CacheError::Engine(LocalFileCacheError::Poisoned {
                resource: "ReadPool"
            }))
        ),
        "expected Err(CacheError::Engine(Poisoned {{ resource: \"ReadPool\" }})), got {result:?}"
    );
    assert!(
        !matches!(result, Ok(LookupResult::Miss)),
        "a poisoned pool must never surface as a cache miss, got {result:?}"
    );
}
