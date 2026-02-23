use std::{
    path::Path,
    process::Command,
    sync::{Arc, Mutex},
    time::UNIX_EPOCH,
};

use anyhow::anyhow;
use image::ImageFormat;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rusqlite::Connection;
use swdir::DirNode;

use crate::{
    engine::{
        database::image::{
            DELETE_BY_ID_STMT, INSERT_STMT, SELECT_ROW_BY_PATH_LIMIT_1_STMT,
            UPDATE_LAST_MODIFIED_STMT, connection,
        },
        media::MediaType,
    },
    env::{
        cache::{CacheKind, image::Cache},
        path::image::cache_thumbnail_file_path,
    },
};

#[derive(Clone)]
pub struct CacheRefresh {
    dir_node: DirNode,
    thumbnail_width: u32,
    thumbnail_height: u32,
}

impl CacheRefresh {
    pub fn new(dir_node: DirNode, thumbnail_width: u32, thumbnail_height: u32) -> Self {
        Self {
            dir_node,
            thumbnail_width,
            thumbnail_height,
        }
    }

    pub async fn caches_refresh(self) -> anyhow::Result<Vec<String>> {
        tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<String>> {
            let conn = Arc::new(Mutex::new(connection()?));
            let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
            self.dir_node.files.par_iter().for_each(|file| {
                if let Err(err) = self.clone().cache_refresh(&file, &conn) {
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

    fn cache_refresh(&self, path: &Path, conn: &Arc<Mutex<Connection>>) -> anyhow::Result<()> {
        let last_modified = path
            .metadata()?
            .modified()?
            .duration_since(UNIX_EPOCH)
            .expect("failed to get unix timestamp")
            .as_secs() as u32;

        let canonicalized_path = path.canonicalize()?;
        let canonicalized_path_str = canonicalized_path.to_string_lossy();

        let conn = conn.lock().unwrap();

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
            Ok(row) => {
                let cache_path = cache_thumbnail_file_path(row.id)?;

                if row.last_modified != last_modified {
                    self.thumbnail_store(path, &cache_path)?;

                    conn.execute(UPDATE_LAST_MODIFIED_STMT, [&last_modified, &row.id])?;
                }

                return Ok(());
            }
            Err(_) => (),
        };

        conn.execute(
            INSERT_STMT,
            (
                &canonicalized_path_str,
                &last_modified,
                CacheKind::Image as u32,
            ),
        )?;
        let id = stmt.query_one([path.canonicalize()?.to_string_lossy()], |row| row.get(0))?;

        let cache_path = cache_thumbnail_file_path(id)?;

        match self.thumbnail_store(path, &cache_path) {
            Ok(_) => (),
            // error handling: rescue case when database was saved but file wasn't
            Err(_) => {
                conn.execute(DELETE_BY_ID_STMT, [&id])?;
            }
        };

        Ok(())
    }

    fn thumbnail_store(&self, original_path: &Path, cache_path: &Path) -> anyhow::Result<()> {
        println!("-- {:?}, {:?}", original_path, cache_path);
        let media_type = MediaType::inspect(original_path);

        let img = match media_type {
            MediaType::Image => image::open(original_path)
                .expect("failed to open as image")
                .resize(
                    self.thumbnail_width,
                    self.thumbnail_height,
                    ::image::imageops::FilterType::Lanczos3,
                ),
            MediaType::Video => {
                let input_path = original_path.to_string_lossy().to_string();
                let output_path = cache_path.to_string_lossy().to_string();

                // FFmpegコマンドの構築
                let output = Command::new("ffmpeg")
                    .args([
                        "-y", // 上書き許可
                        // -ss を -i より入力前に置くことで高速シーク（デコード最小化）
                        "-ss",
                        "10", // 開始位置（0秒）
                        "-t",
                        "1", // 期間 (1秒)
                        "-i",
                        &input_path, // 入力ファイル
                        "-frames:v",
                        "1", // 【重要】1枚だけ取り出す
                        "-q:v",
                        "2", // 画質（1-31, 小さいほど高画質で高速）
                        "-f",
                        "image2",     // フォーマット指定
                        &output_path, // 出力ファイル名
                    ])
                    .output()?;

                if !output.status.success() {
                    return Err(anyhow!(
                        "ffmpeg image crop failed {}",
                        original_path.to_string_lossy()
                    ));
                }

                image::open(cache_path)
                    .expect("failed to open as image")
                    .resize(
                        self.thumbnail_width,
                        self.thumbnail_height,
                        ::image::imageops::FilterType::Lanczos3,
                    )
            }
            MediaType::Other => {
                return Err(anyhow!(
                    "invalid file type {}",
                    original_path.to_string_lossy()
                ));
            }
        };

        img.save_with_format(&cache_path, ImageFormat::Png)?;

        Ok(())
    }
}
