use std::io::{Error, ErrorKind, Result};

use arama_env::validate_dir;
use rusqlite::Connection;
use swdir::DirNode;

use crate::{
    engine::{
        byte::vector_to_blob,
        database::image::{UPDATE_EMBEDDING_STMT, connection, table_ensure},
    },
    env::path::image::cache_thumbnail_dir,
    produce::refresh::refresh_caches,
};

#[derive(Clone)]
pub struct CacheProducer {
    thumbnail_width: u32,
    thumbnail_height: u32,
}

impl CacheProducer {
    pub fn new(thumbnail_width: u32, thumbnail_height: u32) -> Result<Self> {
        validate_dir(&cache_thumbnail_dir()?)?;
        match table_ensure() {
            Ok(_) => (),
            Err(err) => return Err(Error::new(ErrorKind::Other, err.to_string())),
        };

        Ok(Self {
            thumbnail_width,
            thumbnail_height,
        })
    }

    pub fn set_embedding(id: u32, embedding: Vec<f32>) -> anyhow::Result<()> {
        let blob = vector_to_blob(embedding);
        let conn: Connection = connection()?;
        conn.execute(UPDATE_EMBEDDING_STMT, (blob, id))?;
        Ok(())
    }

    pub async fn refresh(self, dir_node: DirNode) -> Vec<String> {
        match refresh_caches(dir_node, self.thumbnail_width, self.thumbnail_height).await {
            Ok(x) => x,
            Err(err) => vec![err.to_string()],
        }
    }

    // todo: delete where cache_kind = 'image'
    pub fn clear() {}
}
