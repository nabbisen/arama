use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::UNIX_EPOCH,
};

use image::ImageFormat;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rusqlite::Connection;
use swdir::DirNode;

use super::super::{
    Cache, CacheKind,
    database::{INSERT_STMT, SELECT_ROW_BY_PATH_STMT, UPDATE_LAST_MODIFIED_STMT, connection},
    path::cache_thumbnail_file_path,
};

pub async fn refresh_caches(
    dir_node: DirNode,
    thumbnail_width: u32,
    thumbnail_height: u32,
) -> anyhow::Result<Vec<String>> {
    tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<String>> {
        let conn = Arc::new(Mutex::new(connection()?));
        let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        dir_node.files.par_iter().for_each(|file| {
            if let Err(err) = refresh_cache(&file, thumbnail_width, thumbnail_height, &conn) {
                errors
                    .lock()
                    .unwrap()
                    .push(format!("{}: {}", file.to_string_lossy(), err));
            };
        });
        let errors = errors
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .collect::<Vec<String>>();
        Ok(errors)
    })
    .await
    .map_err(|e| anyhow::anyhow!("join error: {e}"))?
}

fn refresh_cache(
    path: &Path,
    thumbnail_width: u32,
    thumbnail_height: u32,
    conn: &Arc<Mutex<Connection>>,
) -> anyhow::Result<()> {
    let last_modified = path
        .metadata()?
        .modified()?
        .duration_since(UNIX_EPOCH)
        .expect("failed to get unix timestamp")
        .as_secs() as u32;

    let canonicalized_path = path.canonicalize()?;
    let canonicalized_path_str = canonicalized_path.to_string_lossy();

    let conn = conn.lock().unwrap();

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
        Ok(row) => {
            let cache_file_path = cache_thumbnail_file_path(row.id)?;

            if row.last_modified != last_modified {
                let img = image::open(path).expect("failed to open as image").resize(
                    thumbnail_width,
                    thumbnail_height,
                    ::image::imageops::FilterType::Lanczos3,
                );
                img.save_with_format(&cache_file_path, ImageFormat::Png)?;

                conn.execute(UPDATE_LAST_MODIFIED_STMT, (&last_modified, &row.id))?;
            }

            return Ok(());
        }
        Err(_) => (),
    };

    let img = image::open(path).expect("failed to open as image").resize(
        thumbnail_width,
        thumbnail_height,
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
    let id = stmt.query_one([path.canonicalize()?.to_string_lossy()], |row| row.get(0))?;

    let cache_file_path = cache_thumbnail_file_path(id)?;

    img.save_with_format(&cache_file_path, ImageFormat::Png)?;

    Ok(())
}
