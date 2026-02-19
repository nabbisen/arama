use std::{
    io::Result,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use image::ImageFormat;
use rusqlite::Connection;

use super::{Cache, dir::validate_dir};

const CACHE_SUBDIR: &str = "image";

pub struct ImageCacheManager {
    thumbnail_width: u32,
    thumbnail_height: u32,
}

impl ImageCacheManager {
    pub fn new(thumbnail_width: u32, thumbnail_height: u32) -> Self {
        Self {
            thumbnail_width,
            thumbnail_height,
        }
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

        let conn: Connection = Connection::open(super::database_file()?)?;

        let mut stmt =
            conn.prepare("SELECT id, path, last_modified FROM cache WHERE path = (?1)")?;
        match stmt.query_one([&canonicalized_path_str], |row| {
            Ok(Cache {
                id: row.get(0)?,
                path: row.get(1)?,
                last_modified: row.get(2)?,
            })
        }) {
            Ok(row) => {
                if row.last_modified == last_modified {
                    let cache_file_path = cache_dir()?.join(format!("{}.png", row.id));
                    return Ok(cache_file_path);
                }
            }
            Err(_) => (),
        };

        let img = image::open(path).expect("failed to open as image").resize(
            self.thumbnail_width,
            self.thumbnail_height,
            ::image::imageops::FilterType::Lanczos3,
        );

        conn.execute(
            "INSERT INTO cache (path, last_modified) VALUES (?1, ?2)",
            (&canonicalized_path_str, &last_modified),
        )?;
        let id = stmt
            .query_one([path.canonicalize()?.to_string_lossy()], |row| {
                Ok(Cache {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    last_modified: row.get(2)?,
                })
            })?
            .id;

        let cache_file_path = cache_dir()?.join(&format!("{}.png", id));

        img.save_with_format(&cache_file_path, ImageFormat::Png)?;

        Ok(cache_file_path)
    }
}

fn cache_dir() -> Result<PathBuf> {
    let path = super::cache_dir()?.join(CACHE_SUBDIR);
    validate_dir(&path)?;
    Ok(path)
}
