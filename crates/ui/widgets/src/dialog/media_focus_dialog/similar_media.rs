use std::path::Path;

use arama_ai::{
    config::video_similarity_config::VideoSimilarityConfig,
    pipeline::score::similarity::video::video_similarity_calculator::score_mean_vectors,
};
use arama_sidecar::media::video::video_engine::FfmpegToolchain;
use rayon::prelude::*;

use arama_cache::{
    CacheConfig, DbLocation, ImageCacheConfig, ImageCacheReader, VideoCacheConfig, VideoCacheReader,
};
use arama_env::{
    VIDEO_EXTENSION_ALLOWLIST, cache_lookup_strategy::CacheLookupStrategy, cache_storage_path,
    cache_thumbnail_dir_path,
};

use crate::dialog::similarity_read_outcome::SimilarityReadOutcome;

use super::{MediaFocusDialog, types::SimilarMediaItem};

impl MediaFocusDialog {
    pub fn similar_media(&self) -> SimilarityReadOutcome<SimilarMediaItem> {
        let threshold = self.similarity_threshold;
        let path = &self.history[self.history_index];

        let db_location = match cache_storage_path() {
            Ok(path) => DbLocation::Custom(path),
            Err(err) => {
                eprintln!("failed to get cache storage path: {err}");
                return SimilarityReadOutcome {
                    items: vec![],
                    had_errors: true,
                };
            }
        };
        let read_conns = 4;
        let thumbnail_dir = match cache_thumbnail_dir_path() {
            Ok(path) => Some(path),
            Err(err) => {
                eprintln!("failed to get cache thumbnail dir path: {err}");
                return SimilarityReadOutcome {
                    items: vec![],
                    had_errors: true,
                };
            }
        };

        let cache_config = CacheConfig {
            db_location,
            read_conns,
            thumbnail_dir,
        };

        let is_video = path.extension().is_some_and(|x| {
            VIDEO_EXTENSION_ALLOWLIST.contains(&x.to_string_lossy().to_string().as_str())
        });

        if is_video {
            similar_videos(
                path,
                cache_config,
                self.cache_lookup_strategy,
                threshold,
                self.ffmpeg_toolchain.as_ref(),
            )
        } else {
            similar_images(path, cache_config, self.cache_lookup_strategy, threshold)
        }
    }
}

fn similar_images(
    path: &Path,
    cache_config: CacheConfig,
    cache_lookup_strategy: CacheLookupStrategy,
    threshold: f32,
) -> SimilarityReadOutcome<SimilarMediaItem> {
    let image_cache_reader = match ImageCacheReader::as_session(ImageCacheConfig { cache_config }) {
        Ok(reader) => reader,
        Err(err) => {
            eprintln!("failed to get image cache reader: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
            };
        }
    };

    let cache_lookuped = match cache_lookup_strategy {
        CacheLookupStrategy::Everywhere => image_cache_reader.all(),
        CacheLookupStrategy::CurrentDirAndSubDirs => {
            image_cache_reader.all_in_dir_and_sub_dirs(path)
        }
        CacheLookupStrategy::CurrentDirOnly => image_cache_reader.all_in_dir(path),
    };

    let mut had_batch_errors = false;
    let cache_entries = match cache_lookuped {
        // `.flatten()` alone would silently drop each per-entry `Err` here
        // — the same per-item-continue shape RFC 035 exists to close, just
        // hidden inside a batch call instead of an explicit loop.
        Ok(entries) => entries
            .into_iter()
            .filter_map(|entry| match entry {
                Ok(entry) => Some(entry),
                Err(err) => {
                    eprintln!("failed to lookup image cache entry: {err}");
                    had_batch_errors = true;
                    None
                }
            })
            .collect::<Vec<_>>(),
        Err(err) => {
            eprintln!("failed to lookup image cache entries: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
            };
        }
    };

    // Split target and candidate entries.
    let canonical_path = match path.canonicalize() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(err) => {
            // In scope (RFC 035 §3.1): not a CacheError, but a real
            // failure that would otherwise return an empty vector
            // indistinguishable from "no similar images."
            eprintln!("failed to canonicalize image path: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
            };
        }
    };
    let (target_item, candidates): (Vec<_>, Vec<_>) = cache_entries
        .into_iter()
        .partition(|x| x.path == canonical_path);

    let Some(target_clip_vector) = target_item
        .first()
        .and_then(|target| target.features.as_ref())
        .map(|features| features.clip_vector.to_owned())
    else {
        // Out of scope (RFC 035 §3.1): the target item simply is not
        // indexed yet. An ordinary empty state, not a failure — unless a
        // batch read error already happened, in which case the target's
        // own entry may be exactly what silently failed above; that must
        // not be reported as an ordinary absence.
        return SimilarityReadOutcome {
            items: vec![],
            had_errors: had_batch_errors,
        };
    };

    // Compute similarities in parallel.
    let mut ret = candidates
        .into_par_iter()
        .filter_map(|x| {
            let features = x.features?;
            let similarity = dot_product(&target_clip_vector, &features.clip_vector);
            Some(SimilarMediaItem {
                path: x.path,
                thumbnail_path: x.thumbnail_path,
                similarity,
            })
        })
        .filter(|x| threshold <= x.similarity)
        .collect::<Vec<_>>();

    // Sort by descending similarity. Unstable sort is faster and ordering of
    // exact ties is irrelevant here.
    ret.sort_unstable_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    SimilarityReadOutcome {
        items: ret,
        had_errors: had_batch_errors,
    }
}

fn similar_videos(
    path: &Path,
    cache_config: CacheConfig,
    cache_lookup_strategy: CacheLookupStrategy,
    threshold: f32,
    toolchain: Option<&FfmpegToolchain>,
) -> SimilarityReadOutcome<SimilarMediaItem> {
    let ffmpeg_path = match toolchain {
        Some(toolchain) => toolchain.ffmpeg_path().to_path_buf(),
        None => {
            eprintln!("failed to discover a compatible ffmpeg/ffprobe pair");
            // Deliberately excluded from `had_errors` (RFC 035 §3.1): video
            // comparison never ran, so nothing failed to be read. Missing
            // ffmpeg has its own dedicated surface in Settings -> AI.
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: false,
            };
        }
    };
    let video_cache_reader = match VideoCacheReader::as_session(VideoCacheConfig {
        cache_config,
        ffmpeg_path: Some(ffmpeg_path),
    }) {
        Ok(reader) => reader,
        Err(err) => {
            eprintln!("failed to get video cache reader: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
            };
        }
    };

    let cache_lookuped = match cache_lookup_strategy {
        CacheLookupStrategy::Everywhere => video_cache_reader.all(),
        CacheLookupStrategy::CurrentDirAndSubDirs => {
            video_cache_reader.all_in_dir_and_sub_dirs(path)
        }
        CacheLookupStrategy::CurrentDirOnly => video_cache_reader.all_in_dir(path),
    };

    let mut had_batch_errors = false;
    let cache_entries = match cache_lookuped {
        // `.flatten()` alone would silently drop each per-entry `Err` here
        // — the same per-item-continue shape RFC 035 exists to close, just
        // hidden inside a batch call instead of an explicit loop.
        Ok(entries) => entries
            .into_iter()
            .filter_map(|entry| match entry {
                Ok(entry) => Some(entry),
                Err(err) => {
                    eprintln!("failed to lookup video cache entry: {err}");
                    had_batch_errors = true;
                    None
                }
            })
            .collect::<Vec<_>>(),
        Err(err) => {
            eprintln!("failed to lookup video cache entries: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
            };
        }
    };

    // Split target and candidate entries.
    let canonical_path = match path.canonicalize() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(err) => {
            // In scope (RFC 035 §3.1): not a CacheError, but a real
            // failure that would otherwise return an empty vector
            // indistinguishable from "no similar videos."
            eprintln!("failed to canonicalize video path: {err}");
            return SimilarityReadOutcome {
                items: vec![],
                had_errors: true,
            };
        }
    };
    let (target_item, candidates): (Vec<_>, Vec<_>) = cache_entries
        .into_iter()
        .partition(|x| x.path == canonical_path);

    let Some(target_features) = target_item
        .first()
        .and_then(|target| target.features.as_ref())
    else {
        // Out of scope (RFC 035 §3.1): the target item simply is not
        // indexed yet. An ordinary empty state, not a failure — unless a
        // batch read error already happened, in which case the target's
        // own entry may be exactly what silently failed above; that must
        // not be reported as an ordinary absence.
        return SimilarityReadOutcome {
            items: vec![],
            had_errors: had_batch_errors,
        };
    };
    let video_similarity_config = VideoSimilarityConfig::default();

    // Compute similarities in parallel.
    let mut ret = candidates
        .into_par_iter()
        .filter_map(|x| {
            let similarity = match &x.features {
                Some(features) => score_mean_vectors(
                    target_features.clip_vector.as_deref(),
                    target_features.wav2vec2_vector.as_deref(),
                    features.clip_vector.as_deref(),
                    features.wav2vec2_vector.as_deref(),
                    video_similarity_config.image_weight,
                    video_similarity_config.audio_weight,
                )?,
                _ => return None,
            };

            Some(SimilarMediaItem {
                path: x.path,
                thumbnail_path: x.thumbnail_path,
                similarity,
            })
        })
        .filter(|x| threshold <= x.similarity)
        .collect::<Vec<_>>();

    // Sort by descending similarity. Unstable sort is faster and ordering of
    // exact ties is irrelevant here.
    ret.sort_unstable_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    SimilarityReadOutcome {
        items: ret,
        had_errors: had_batch_errors,
    }
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
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
}
