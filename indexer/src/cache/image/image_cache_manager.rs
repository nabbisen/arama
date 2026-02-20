use std::{
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
};

use arama_env::validate_dir;
use rusqlite::Connection;
use swdir::DirNode;

mod refresh;

use refresh::refresh_caches;

use super::{
    Cache,
    byte::{blob_to_vector, vector_to_blob},
    database::{
        SELECT_EMBEDDING_BY_ID_STMT, SELECT_ID_BY_PATH_STMT, SELECT_ROW_BY_PATH_STMT,
        UPDATE_EMBEDDING_STMT, connection, table_prepare_if_necessary,
    },
    path::{cache_thumbnail_dir, cache_thumbnail_file_path},
};

#[derive(Clone)]
pub struct ImageCacheManager {
    thumbnail_width: u32,
    thumbnail_height: u32,
}

impl ImageCacheManager {
    pub fn new(thumbnail_width: u32, thumbnail_height: u32) -> Result<Self> {
        validate_dir(&cache_thumbnail_dir()?)?;
        match table_prepare_if_necessary() {
            Ok(_) => (),
            Err(err) => return Err(Error::new(ErrorKind::Other, err.to_string())),
        };

        Ok(Self {
            thumbnail_width,
            thumbnail_height,
        })
    }

    pub fn get_cache(path: &Path) -> anyhow::Result<Option<Cache>> {
        let canonicalized_path = path.canonicalize()?;
        let canonicalized_path_str = canonicalized_path.to_string_lossy();
        let conn: Connection = connection()?;
        let mut stmt = conn.prepare(SELECT_ROW_BY_PATH_STMT)?;
        match stmt.query_one([&canonicalized_path_str], |row| {
            Ok(Cache {
                id: row.get(0)?,
                path: row.get(1)?,
                last_modified: row.get(2)?,
                cache_kind: row.get(3)?,
                embedding: row.get(4)?,
            })
        }) {
            Ok(row) => Ok(Some(row)),
            Err(_) => Ok(None),
        }
    }

    pub fn get_cache_file_path(path: &Path) -> anyhow::Result<Option<PathBuf>> {
        let canonicalized_path = path.canonicalize()?;
        let canonicalized_path_str = canonicalized_path.to_string_lossy();
        let conn: Connection = connection()?;
        let mut stmt = conn.prepare(SELECT_ID_BY_PATH_STMT)?;
        match stmt.query_one([&canonicalized_path_str], |row| row.get::<usize, u32>(0)) {
            Ok(id) => Ok(Some(cache_thumbnail_file_path(id)?)),
            Err(_) => Ok(None),
        }
    }

    pub fn get_embedding(id: u32) -> anyhow::Result<Option<Vec<f32>>> {
        let conn: Connection = connection()?;
        let mut stmt = conn.prepare(SELECT_EMBEDDING_BY_ID_STMT)?;
        let embedding = stmt.query_one([id], |row| row.get::<usize, Vec<u8>>(0));
        match embedding {
            Ok(x) => Ok(Some(blob_to_vector(x))),
            Err(_) => Ok(None),
        }
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
