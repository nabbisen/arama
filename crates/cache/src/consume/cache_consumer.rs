use std::{
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
};

use arama_env::validate_dir;
use arama_repr::byte::blob_to_vector;
use rusqlite::Connection;
use swdir::DirNode;

use arama_storage::{
    database::image::{
        CREATE_TABLE_TMP_IMAGE_PATHS_STMT, INSERT_TMP_IMAGE_PATHS_STMT,
        SELECT_EMBEDDING_BY_ID_STMT, SELECT_ID_BY_PATH_STMT, SELECT_ID_EMBEDDING_BY_PATHS_STMT,
        SELECT_ROW_BY_PATH_LIMIT_1_STMT, connection, table_ensure,
    },
    env::{
        cache::image::Cache,
        path::image::{cache_thumbnail_dir, cache_thumbnail_file_path},
    },
};

#[derive(Clone)]
pub struct CacheConcumer {}

impl CacheConcumer {
    pub fn new() -> Result<Self> {
        validate_dir(&cache_thumbnail_dir()?)?;
        match table_ensure() {
            Ok(_) => (),
            Err(err) => return Err(Error::new(ErrorKind::Other, err.to_string())),
        };

        Ok(Self {})
    }

    pub fn get_cache(path: &Path) -> anyhow::Result<Option<Cache>> {
        let canonicalized_path = path.canonicalize()?;
        let canonicalized_path_str = canonicalized_path.to_string_lossy();
        let conn: Connection = connection()?;
        let mut stmt = conn.prepare(SELECT_ROW_BY_PATH_LIMIT_1_STMT)?;
        match stmt.query_row([&canonicalized_path_str], |row| {
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

    pub fn get_embeddings<T: Into<DirNode>>(
        dir_node: T,
    ) -> anyhow::Result<Vec<(PathBuf, Vec<f32>)>> {
        let conn: Connection = connection()?;

        conn.execute(CREATE_TABLE_TMP_IMAGE_PATHS_STMT, ())?;

        let paths: Vec<String> = dir_node
            .into()
            .flatten_paths()
            .iter()
            .map(|x| x.canonicalize())
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap().to_string_lossy().to_string())
            .collect();

        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(INSERT_TMP_IMAGE_PATHS_STMT)?;
            for path in paths {
                stmt.execute([path])?;
            }
        }
        tx.commit()?;

        let mut stmt = conn.prepare(SELECT_ID_EMBEDDING_BY_PATHS_STMT)?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<usize, u32>(0)?, row.get::<usize, Vec<u8>>(1)?))
        });
        match rows {
            Ok(x) => {
                let embeddings = x
                    .map(|x| match x {
                        Ok(x) => Some((x.0, blob_to_vector(x.1))),
                        Err(_err) => None,
                    })
                    // todo: error handling
                    .filter(|x| x.is_some())
                    .map(|x| {
                        let (id, embedding) = x.unwrap();
                        (cache_thumbnail_file_path(id), embedding)
                    })
                    .filter(|x| x.0.is_ok())
                    .map(|(path, embedding)| (path.unwrap(), embedding))
                    .collect::<Vec<(PathBuf, Vec<f32>)>>();
                Ok(embeddings)
            }
            Err(_) => Ok(vec![]),
        }
    }
}
