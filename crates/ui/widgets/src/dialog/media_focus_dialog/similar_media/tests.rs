use std::fs;

use arama_cache::{ImageCacheWriter, UpsertImageRequest};

use super::*;

/// A real, isolated cache in its own tempdir — never the owner's
/// profile. Dropped (and its directory removed) at the end of the
/// test that created it.
struct TestCache {
    _dir: tempfile::TempDir,
    db_path: std::path::PathBuf,
}

impl TestCache {
    fn new() -> Self {
        let dir = tempfile::TempDir::new().expect("create tempdir");
        let db_path = dir.path().join("cache.sqlite");
        Self { _dir: dir, db_path }
    }

    fn cache_config(&self) -> CacheConfig {
        CacheConfig {
            db_location: DbLocation::Custom(self.db_path.clone()),
            read_conns: 2,
            thumbnail_dir: None,
        }
    }

    fn image_writer(&self) -> ImageCacheWriter {
        ImageCacheWriter::as_session(ImageCacheConfig {
            cache_config: self.cache_config(),
        })
        .expect("open image cache writer")
    }
}

/// A `CacheConfig` whose db path can never be created: `blocker` is a
/// real file, so creating `blocker/sub/cache.sqlite`'s parent
/// directory fails with a genuine I/O error (ENOTDIR) — this forces
/// `ImageCacheReader::as_session` to return `Err` without needing to
/// corrupt or poison a real cache.
fn unconstructable_cache_config(dir: &std::path::Path) -> CacheConfig {
    let blocker = dir.join("blocker");
    fs::write(&blocker, b"not a directory").expect("create blocker file");
    CacheConfig {
        db_location: DbLocation::Custom(blocker.join("sub").join("cache.sqlite")),
        read_conns: 2,
        thumbnail_dir: None,
    }
}

fn real_file(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    fs::write(&path, b"fixture content").expect("create fixture file");
    path
}

#[test]
fn similar_images_reports_error_when_cache_reader_construction_fails() {
    let dir = tempfile::TempDir::new().expect("create tempdir");
    let target = real_file(dir.path(), "target.jpg");

    let outcome = similar_images(
        &target,
        unconstructable_cache_config(dir.path()),
        CacheLookupStrategy::CurrentDirOnly,
        0.0,
    );

    // RFC 035: a whole-lookup failure must be a visible error state,
    // never an empty success indistinguishable from "no matches."
    assert!(
        outcome.had_errors,
        "construction failure must set had_errors"
    );
    assert!(outcome.items.is_empty());
}

#[test]
fn similar_images_empty_cache_is_not_an_error() {
    let cache = TestCache::new();
    let dir = tempfile::TempDir::new().expect("create tempdir");
    let target = real_file(dir.path(), "target.jpg");
    // Open (and immediately drop) a writer so the schema exists, but
    // upsert nothing — a genuinely empty cache.
    let _ = cache.image_writer();

    let outcome = similar_images(
        &target,
        cache.cache_config(),
        CacheLookupStrategy::CurrentDirOnly,
        0.0,
    );

    assert!(!outcome.had_errors, "an empty cache is not a failure");
    assert!(outcome.items.is_empty());
}

#[test]
fn similar_images_unindexed_target_with_populated_cache_is_not_an_error() {
    let cache = TestCache::new();
    let dir = tempfile::TempDir::new().expect("create tempdir");
    let target = real_file(dir.path(), "target.jpg");
    let other = real_file(dir.path(), "other.jpg");

    let writer = cache.image_writer();
    writer
        .upsert(UpsertImageRequest {
            path: other,
            clip_vector: Some(vec![1.0, 0.0, 0.0]),
        })
        .expect("upsert candidate entry");
    // `target` is deliberately never upserted — this is the exact
    // shape RFC 035 §3.1 warns is easy to get wrong: it looks
    // identical to the failure returns around it, but must not
    // produce a message.

    let outcome = similar_images(
        &target,
        cache.cache_config(),
        CacheLookupStrategy::CurrentDirOnly,
        0.0,
    );

    assert!(
        !outcome.had_errors,
        "an unindexed target item is an ordinary empty state, not a failure"
    );
    assert!(outcome.items.is_empty());
    assert!(
        outcome.nothing_indexed,
        "RFC 036: the target itself was never indexed"
    );
}

#[test]
fn similar_images_searched_and_found_nothing_is_distinct_from_not_indexed() {
    let cache = TestCache::new();
    let dir = tempfile::TempDir::new().expect("create tempdir");
    let target = real_file(dir.path(), "target.jpg");
    let dissimilar = real_file(dir.path(), "dissimilar.jpg");

    let writer = cache.image_writer();
    writer
        .upsert(UpsertImageRequest {
            path: target.clone(),
            clip_vector: Some(vec![1.0, 0.0, 0.0]),
        })
        .expect("upsert target entry");
    writer
        .upsert(UpsertImageRequest {
            path: dissimilar,
            // Orthogonal vector: dot product is 0.0, well under any
            // positive threshold.
            clip_vector: Some(vec![0.0, 1.0, 0.0]),
        })
        .expect("upsert dissimilar entry");

    let outcome = similar_images(
        &target,
        cache.cache_config(),
        CacheLookupStrategy::CurrentDirOnly,
        0.5,
    );

    assert!(!outcome.had_errors);
    assert!(outcome.items.is_empty());
    assert!(
        !outcome.nothing_indexed,
        "RFC 036: the target was indexed and a search ran - this is \
         'found nothing', not 'nothing indexed yet'"
    );
}

#[test]
fn similar_images_finds_indexed_similar_target_without_error() {
    let cache = TestCache::new();
    let dir = tempfile::TempDir::new().expect("create tempdir");
    let target = real_file(dir.path(), "target.jpg");
    let similar = real_file(dir.path(), "similar.jpg");

    let writer = cache.image_writer();
    writer
        .upsert(UpsertImageRequest {
            path: target.clone(),
            clip_vector: Some(vec![1.0, 0.0, 0.0]),
        })
        .expect("upsert target entry");
    writer
        .upsert(UpsertImageRequest {
            path: similar,
            clip_vector: Some(vec![1.0, 0.0, 0.0]),
        })
        .expect("upsert similar entry");

    let outcome = similar_images(
        &target,
        cache.cache_config(),
        CacheLookupStrategy::CurrentDirOnly,
        0.5,
    );

    assert!(!outcome.had_errors);
    assert_eq!(
        outcome.items.len(),
        1,
        "the identical-vector entry must match"
    );
}

#[test]
fn similar_videos_missing_ffmpeg_produces_no_error_and_no_items() {
    let dir = tempfile::TempDir::new().expect("create tempdir");
    let target = real_file(dir.path(), "target.mp4");

    // No cache setup: `similar_videos` must return before ever
    // touching `cache_config` when there is no toolchain, per RFC 035
    // §3.1 — a dummy config that would fail if used proves this.
    let bogus_config = CacheConfig {
        db_location: DbLocation::Custom(dir.path().join("unused.sqlite")),
        read_conns: 1,
        thumbnail_dir: None,
    };

    let outcome = similar_videos(
        &target,
        bogus_config,
        CacheLookupStrategy::CurrentDirOnly,
        0.0,
        None,
    );

    assert!(
        !outcome.had_errors,
        "missing ffmpeg has its own dedicated surface, not this dialog's error message"
    );
    assert!(outcome.items.is_empty());
    assert!(
        outcome.ffmpeg_missing_with_videos,
        "RFC 036: the dialog must say video comparison did not run"
    );
}

// `similar_videos`'s cache-reader-construction-failure, per-entry
// batch-error, and unindexed-target paths are not covered here:
// `FfmpegToolchain`'s fields are `pub(super)` to `arama-sidecar`'s
// video_engine module, so no `Some(toolchain)` value can be
// constructed from this crate without a real, validated ffmpeg pair.
// `similar_videos` is hand-written in parallel with `similar_images`
// (identical shape, same fix applied to both) and the above tests
// exercise that shape directly; this is the same kind of declared
// gap as the async-fn/full-`App` constraints recorded in this
// project's earlier RFC 033/RFC 034 work.
