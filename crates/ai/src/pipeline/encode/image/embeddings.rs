use std::path::PathBuf;

use anyhow::anyhow;
use arama_cache::{
    CacheConfig, DbLocation, ImageCacheConfig, ImageCacheWriter, LookupResult, UpsertImageRequest,
};
use arama_env::{VIDEO_EXTENSION_ALLOWLIST, cache_storage_path, cache_thumbnail_dir_path};

use crate::{
    config::video_similarity_config::VideoSimilarityConfig,
    pipeline::encode::image::{clip, clip_calculator},
    pipeline_manager::video_similarity_pipeline::VideoSimilarityPipeline,
};

pub async fn image_embedding(paths: Vec<PathBuf>) -> anyhow::Result<Option<String>> {
    let calculator = match clip_calculator() {
        Ok(x) => x,
        Err(err) => {
            return Err(anyhow!("failed to load clip calculator: {}", err));
        }
    };

    let db_location =
        DbLocation::Custom(cache_storage_path().expect("failed to get cache stogate path"));
    let cache_writer = ImageCacheWriter::as_session(ImageCacheConfig {
        cache_config: CacheConfig {
            db_location,
            read_conns: 4,
            thumbnail_dir: Some(cache_thumbnail_dir_path()?),
        },
    })?;

    for path in paths {
        if path.extension().is_some_and(|x| {
            VIDEO_EXTENSION_ALLOWLIST.contains(&x.to_string_lossy().to_string().as_str())
        }) {
            let _ = VideoSimilarityPipeline::new(VideoSimilarityConfig::default())?.preload(&path);
            continue;
        }

        match cache_writer.as_reader().lookup(&path)? {
            LookupResult::Hit(x) if x.features.is_some() => continue,
            _ => (),
        }

        let embedding = match clip(&path, &calculator) {
            Ok(x) => x,
            Err(err) => return Err(anyhow!("failed to clip calculation: {}", err)),
        };
        let req = UpsertImageRequest {
            path,
            // thumbnail_path: path,
            clip_vector: Some(embedding.embedding),
        };
        match cache_writer.upsert(req) {
            Ok(_) => (),
            Err(err) => return Err(anyhow!("failed to set embedding: {}", err)),
        }
    }

    Ok(None)
}
