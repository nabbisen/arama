use std::path::PathBuf;

use anyhow::anyhow;
use arama_cache::{
    CacheConfig, DbLocation, ImageCacheConfig, ImageCacheWriter, UpsertImageRequest,
};
use arama_env::{cache_storage_path, cache_thumbnail_dir_path};

use crate::pipeline::encode::image::{clip, clip_calculator};

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
        cache: CacheConfig {
            db_location,
            read_conns: 4,
            thumbnail_dir: Some(cache_thumbnail_dir_path()?),
        },
    })?;
    for path in paths {
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
