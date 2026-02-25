use arama_cache::{CacheConcumer, CacheProducer};
use swdir::DirNode;

use crate::pipeline::encode::image::{clip, clip_calculator};

pub async fn image_embedding(dir_node: DirNode) -> Option<String> {
    let calculator = match clip_calculator() {
        Ok(x) => x,
        Err(err) => return Some(format!("failed to load clip calculator: {}", err)),
    };

    for path in dir_node.files {
        match CacheConcumer::get_cache(&path) {
            Ok(cache) => {
                let cache = if let Some(cache) = cache {
                    cache
                } else {
                    // todo error handling
                    return Some("failed to get cache".to_string());
                };

                let embedding = match clip(&path, &calculator) {
                    Ok(x) => x,
                    Err(err) => return Some(format!("failed to clip calculation: {}", err)),
                };
                match CacheProducer::set_embedding(cache.id(), embedding.embedding) {
                    Ok(_) => (),
                    Err(err) => return Some(format!("failed to set embedding: {}", err)),
                }
            }
            Err(err) => return Some(err.to_string()),
        }
    }
    None
}
