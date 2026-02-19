use std::{env, io::Result, path::PathBuf};

use rusqlite::Connection;

mod dir;
pub mod image_cache_manager;

const CACHE_DIR: &str = ".cache";
const DATABASE_FILE: &str = "cache.sqlite3";

const CREATE_TABLE_STMT: &str = "CREATE TABLE cache (
        id   INTEGER PRIMARY KEY,
        path TEXT NOT NULL,
        last_modified INTEGER NOT NULL
    )";

#[derive(Debug)]
struct Cache {
    #[allow(dead_code)]
    id: i32,
    #[allow(dead_code)]
    path: String,
    last_modified: u32,
}

pub fn cache_dir() -> Result<PathBuf> {
    let path = exe_parent_dir()?.join(CACHE_DIR);

    dir::validate_dir(&path)?;

    Ok(path.to_path_buf())
}

pub fn database_file() -> anyhow::Result<PathBuf> {
    let path = exe_parent_dir()?.join(DATABASE_FILE);

    if !path.exists() {
        let conn = Connection::open(&path)?;
        conn.execute(CREATE_TABLE_STMT, ())?;
        let _ = conn.close();
    }

    Ok(path.to_path_buf())
}

fn exe_parent_dir() -> Result<PathBuf> {
    let current_exe = env::current_exe()?;
    let path = current_exe
        .parent()
        .expect("failed to get exe parent directory");
    Ok(path.to_path_buf())
}
