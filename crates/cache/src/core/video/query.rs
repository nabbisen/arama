use std::path::{MAIN_SEPARATOR, Path};

use file_feature_cache::CacheError;
use rayon::prelude::*;
use rusqlite::{Connection, ToSql};

use crate::core::codec::blob_to_vec;

pub struct CacheEntry {
    pub path: String,
    pub thumbnail_path: Option<String>,
    pub image_features: Option<Vec<f32>>,
    pub audio_features: Option<Vec<f32>>,
}

struct CacheRawEntry {
    path: String,
    thumbnail_path: Option<String>,
    image_vec: Option<Vec<u8>>,
    audio_vec: Option<Vec<u8>>,
}

pub fn all(conn: &Connection) -> Result<Vec<Result<CacheEntry, CacheError>>, CacheError> {
    all_impl(conn, None, &[])
}

pub fn all_in_dir(
    path: &Path,
    conn: &Connection,
) -> Result<Vec<Result<CacheEntry, CacheError>>, CacheError> {
    let dir = if !path.is_dir() {
        path.parent().expect("failed to get parent dir")
    } else {
        path
    };

    let canonicalized = dir
        .canonicalize()
        .expect("failed to get canonicalized")
        .to_string_lossy()
        .to_string();

    let dir_glob_str = format!("{}{}*", canonicalized, MAIN_SEPARATOR);

    let subdirs_glob_str = format!("{}{}*{}*", canonicalized, MAIN_SEPARATOR, MAIN_SEPARATOR);

    all_impl(
        conn,
        Some("path GLOB ? and path NOT GLOB ?"),
        &[&dir_glob_str, &subdirs_glob_str],
    )
}

pub fn all_in_dir_and_subdirs(
    path: &Path,
    conn: &Connection,
) -> Result<Vec<Result<CacheEntry, CacheError>>, CacheError> {
    let dir = if !path.is_dir() {
        path.parent().expect("failed to get parent dir")
    } else {
        path
    };

    let canonicalized = dir
        .canonicalize()
        .expect("failed to get canonicalized")
        .to_string_lossy()
        .to_string();

    let glob_str = format!("{}{}*", canonicalized, MAIN_SEPARATOR);

    all_impl(conn, Some("path GLOB ?"), &[&glob_str])
}

fn all_impl(
    conn: &Connection,
    where_clause: Option<&str>,
    where_params: &[&dyn ToSql],
) -> Result<Vec<Result<CacheEntry, CacheError>>, CacheError> {
    let mut query = "SELECT a.path, c.thumbnail_path, b.clip_vector, b.wav2vec2_vector
        FROM files a
        INNER JOIN video_features b ON b.id = a.id
        LEFT OUTER JOIN thumbnails c ON c.id = a.id"
        .to_owned();
    if let Some(where_clause) = where_clause {
        query = format!("{} WHERE {}", query, where_clause);
    }

    let mut stmt = conn.prepare(&query)?;

    let record_iter = stmt.query_map(where_params, |row| {
        let thumbnail_path = if let Ok(x) = row.get::<_, String>("thumbnail_path") {
            Some(x)
        } else {
            None
        };

        let image_vec = if let Ok(x) = row.get::<_, Vec<u8>>("clip_vector") {
            Some(x)
        } else {
            None
        };

        let audio_vec = if let Ok(x) = row.get::<_, Vec<u8>>("clip_vector") {
            Some(x)
        } else {
            None
        };

        Ok(CacheRawEntry {
            path: row.get::<_, String>("path")?,
            thumbnail_path,
            image_vec,
            audio_vec,
        })
    })?;

    let ret = record_iter
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|x| match x {
            Ok(x) => {
                let image_features = if let Some(vec) = x.image_vec {
                    if let Ok(features) = blob_to_vec(&vec) {
                        Some(features)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let audio_features = if let Some(vec) = x.audio_vec {
                    if let Ok(features) = blob_to_vec(&vec) {
                        Some(features)
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(CacheEntry {
                    path: x.path,
                    thumbnail_path: x.thumbnail_path,
                    image_features,
                    audio_features,
                })
            }
            Err(err) => Err(file_feature_cache::CacheError::Sqlite(err)),
        })
        .collect::<Vec<_>>();

    Ok(ret)
}
