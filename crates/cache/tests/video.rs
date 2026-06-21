// Video-namespace integration tests for arama-cache.

#[path = "helpers.rs"]
#[allow(dead_code)]
mod helpers;

use std::path::Path;

use arama_cache::{LookupResult, UpsertVideoRequest};

use helpers::{TempFile, video_writer};

// ---------------------------------------------------------------------------
// upsert / lookup
// ---------------------------------------------------------------------------

#[test]
fn video_upsert_and_lookup_hit_with_both_vectors() {
    let (writer, _db) = video_writer();
    let file = TempFile::new(b"video data");

    writer
        .upsert(UpsertVideoRequest {
            path: file.path().to_path_buf(),
            clip_vector: Some(vec![1.0, 2.0]),
            wav2vec2_vector: Some(vec![3.0, 4.0]),
        })
        .unwrap();

    match writer.lookup(file.path()).unwrap() {
        LookupResult::Hit(entry) => {
            let feat = entry.features.expect("features should be Some");
            assert_eq!(feat.clip_vector, Some(vec![1.0f32, 2.0]));
            assert_eq!(feat.wav2vec2_vector, Some(vec![3.0f32, 4.0]));
        }
        other => panic!("expected Hit, got {:?}", other),
    }
}

#[test]
fn video_upsert_partial_vectors_preserved_via_coalesce() {
    let (writer, _db) = video_writer();
    let file = TempFile::new(b"video partial");

    // Write CLIP only.
    writer
        .upsert(UpsertVideoRequest {
            path: file.path().to_path_buf(),
            clip_vector: Some(vec![0.5]),
            wav2vec2_vector: None,
        })
        .unwrap();
    // Add wav2vec2 only — the CLIP vector must be preserved (coalesce).
    writer
        .upsert(UpsertVideoRequest {
            path: file.path().to_path_buf(),
            clip_vector: None,
            wav2vec2_vector: Some(vec![0.9]),
        })
        .unwrap();

    match writer.lookup(file.path()).unwrap() {
        LookupResult::Hit(entry) => {
            let feat = entry.features.unwrap();
            assert_eq!(feat.clip_vector, Some(vec![0.5f32]));
            assert_eq!(feat.wav2vec2_vector, Some(vec![0.9f32]));
        }
        other => panic!("expected Hit, got {:?}", other),
    }
}

#[test]
fn video_lookup_miss_on_unregistered_file() {
    let (writer, _db) = video_writer();
    let file = TempFile::new(b"unregistered video");
    assert!(matches!(
        writer.lookup(file.path()).unwrap(),
        LookupResult::Miss
    ));
}

#[test]
fn video_lookup_invalidated_on_file_change() {
    let (writer, _db) = video_writer();
    let file = TempFile::new(b"original video");
    writer
        .upsert(UpsertVideoRequest {
            path: file.path().to_path_buf(),
            clip_vector: Some(vec![1.0]),
            wav2vec2_vector: None,
        })
        .unwrap();
    // Different byte length — detection is unambiguous via size change.
    file.overwrite(b"modified video!");
    assert!(matches!(
        writer.lookup(file.path()).unwrap(),
        LookupResult::Invalidated
    ));
}

// ---------------------------------------------------------------------------
// upsert_all / lookup_all
// ---------------------------------------------------------------------------

#[test]
fn video_upsert_all_registers_all_files() {
    let (writer, _db) = video_writer();
    let files: Vec<_> = (0..4)
        .map(|i| TempFile::new(format!("batch vid {i}").as_bytes()))
        .collect();

    let reqs: Vec<_> = files
        .iter()
        .map(|f| UpsertVideoRequest {
            path: f.path().to_path_buf(),
            clip_vector: Some(vec![0.1, 0.2]),
            wav2vec2_vector: Some(vec![0.3, 0.4]),
        })
        .collect();

    let results = writer.upsert_all(reqs);
    assert_eq!(results.len(), 4);
    for (_, r) in &results {
        assert!(r.is_ok(), "{r:?}");
    }
}

#[test]
fn video_upsert_all_partial_failure_continues() {
    let (writer, _db) = video_writer();
    let good = TempFile::new(b"good video");
    let bad = std::path::PathBuf::from("/no/such/video.mp4");

    let results = writer.upsert_all(vec![
        UpsertVideoRequest {
            path: good.path().to_path_buf(),
            clip_vector: None,
            wav2vec2_vector: None,
        },
        UpsertVideoRequest {
            path: bad,
            clip_vector: None,
            wav2vec2_vector: None,
        },
    ]);

    assert_eq!(results.len(), 2);
    assert!(results[0].1.is_ok());
    assert!(results[1].1.is_err());
}

#[test]
fn video_lookup_all_returns_hits_for_upserted_files() {
    let (writer, _db) = video_writer();
    let files: Vec<_> = (0..4)
        .map(|i| TempFile::new(format!("vid_lookup_all {i}").as_bytes()))
        .collect();

    let reqs: Vec<_> = files
        .iter()
        .map(|f| UpsertVideoRequest {
            path: f.path().to_path_buf(),
            clip_vector: Some(vec![1.0]),
            wav2vec2_vector: None,
        })
        .collect();
    writer.upsert_all(reqs);

    let paths: Vec<&Path> = files.iter().map(|f| f.path()).collect();
    let results = writer.as_reader().lookup_all(&paths);

    assert_eq!(results.len(), 4);
    for (_, r) in &results {
        assert!(matches!(r.as_ref().unwrap(), LookupResult::Hit(_)));
    }
}
