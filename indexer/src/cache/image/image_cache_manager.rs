use std::{
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use arama_env::validate_dir;
use image::ImageFormat;
use rusqlite::Connection;

use super::{
    Cache, CacheKind,
    database::{INSERT_STMT, SELECT_STMT, UPDATE_STMT, connection, table_prepare_if_necessary},
    path::{cache_thumbnail_dir, cache_thumbnail_file_path},
};

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

    pub fn cache_path(&self, path: &Path) -> anyhow::Result<PathBuf> {
        let last_modified = path
            .metadata()?
            .modified()?
            .duration_since(UNIX_EPOCH)
            .expect("failed to get unix timestamp")
            .as_secs() as u32;

        let canonicalized_path = path.canonicalize()?;
        let canonicalized_path_str = canonicalized_path.to_string_lossy();

        let conn: Connection = connection()?;

        let mut stmt = conn.prepare(SELECT_STMT)?;
        match stmt.query_one([&canonicalized_path_str], |row| {
            Ok(Cache {
                id: row.get(0)?,
                path: row.get(1)?,
                last_modified: row.get(2)?,
                cache_kind: row.get(3)?,
            })
        }) {
            Ok(row) => {
                let cache_file_path = cache_thumbnail_file_path(row.id)?;

                if row.last_modified != last_modified {
                    let img = image::open(path).expect("failed to open as image").resize(
                        self.thumbnail_width,
                        self.thumbnail_height,
                        ::image::imageops::FilterType::Lanczos3,
                    );
                    img.save_with_format(&cache_file_path, ImageFormat::Png)?;

                    conn.execute(UPDATE_STMT, (&last_modified, &row.id))?;
                }

                return Ok(cache_file_path);
            }
            Err(_) => (),
        };

        let img = image::open(path).expect("failed to open as image").resize(
            self.thumbnail_width,
            self.thumbnail_height,
            ::image::imageops::FilterType::Lanczos3,
        );

        conn.execute(
            INSERT_STMT,
            (
                &canonicalized_path_str,
                &last_modified,
                CacheKind::Image as u32,
            ),
        )?;
        let id = stmt
            .query_one([path.canonicalize()?.to_string_lossy()], |row| {
                Ok(Cache {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    last_modified: row.get(2)?,
                    cache_kind: row.get(3)?,
                })
            })?
            .id;

        let cache_file_path = cache_thumbnail_file_path(id)?;

        img.save_with_format(&cache_file_path, ImageFormat::Png)?;

        Ok(cache_file_path)
    }

    // todo: delete where cache_kind = 'image'
    pub fn clear() {}
}
