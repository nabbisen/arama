mod database;
pub mod image_cache_manager;
mod path;

#[derive(Debug)]
struct Cache {
    #[allow(dead_code)]
    id: u32,
    #[allow(dead_code)]
    path: String,
    last_modified: u32,
    #[allow(dead_code)]
    cache_kind: u32,
}

enum CacheKind {
    Image,
}
