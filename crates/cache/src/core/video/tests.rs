//! RFC 033 Part B: video mirror of the image poisoning-surface test. See
//! `crates/cache/src/core/image/tests.rs` for the full rationale.

use localcache::LocalFileCacheError;

use super::*;
use crate::core::engine::CacheError;

#[test]
fn poisoned_read_pool_surfaces_as_error_not_miss() {
    let db = tempfile::NamedTempFile::new().expect("create temp db file");
    // See the image-crate sibling test: `read_conns = 1` is required to
    // force `ReadPool::checkout`'s blocking fallback deterministically.
    let writer = VideoCacheWriter::as_session(VideoCacheConfig {
        cache_config: CacheConfig {
            db_location: DbLocation::Custom(db.path().to_path_buf()),
            read_conns: 1,
            thumbnail_dir: None,
        },
        ffmpeg_path: None,
    })
    .expect("open writer");
    let reader = writer.as_reader();

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
