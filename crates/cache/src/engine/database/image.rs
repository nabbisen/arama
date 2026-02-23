use std::path::PathBuf;

use crate::env::path::image::{DATABASE_FILE, cache_dir};
use rusqlite::Connection;

#[allow(dead_code)]
pub enum TableName {
    Image,
    TmpImagePaths,
}

macro_rules! table_name {
    (TableName::Image) => {
        "image"
    };
    (TableName::TmpImagePaths) => {
        "tmp_image_paths"
    };
}

const CREATE_TABLE_STMT: &str = concat!(
    "CREATE TABLE ",
    table_name!(TableName::Image),
    " (
        id   INTEGER PRIMARY KEY,
        path TEXT NOT NULL,
        last_modified INTEGER NOT NULL,
        cache_kind INTEGER NOT NULL,
        embedding BLOB
    )"
);

pub const INSERT_STMT: &str = concat!(
    "INSERT INTO ",
    table_name!(TableName::Image),
    " (path, last_modified, cache_kind) VALUES (?1, ?2, ?3)"
);

pub const UPDATE_LAST_MODIFIED_STMT: &str = concat!(
    "UPDATE ",
    table_name!(TableName::Image),
    " SET last_modified = ?1, embedding = NULL WHERE id = ?2"
);

pub const UPDATE_EMBEDDING_STMT: &str = concat!(
    "UPDATE ",
    table_name!(TableName::Image),
    " SET embedding = ?1 WHERE id = ?2"
);

pub const SELECT_ROW_BY_PATH_LIMIT_1_STMT: &str = concat!(
    "SELECT id, path, last_modified, cache_kind, embedding FROM ",
    table_name!(TableName::Image),
    " WHERE path = (?1)",
    " LIMIT 1"
);

pub const SELECT_ID_BY_PATH_STMT: &str = concat!(
    "SELECT id FROM ",
    table_name!(TableName::Image),
    " WHERE path = (?1)"
);

pub const SELECT_EMBEDDING_BY_ID_STMT: &str = concat!(
    "SELECT embedding FROM ",
    table_name!(TableName::Image),
    " WHERE id = (?1)"
);

pub const SELECT_ID_EMBEDDING_BY_PATHS_STMT: &str = concat!(
    "SELECT x.id, x.embedding FROM ",
    table_name!(TableName::Image),
    " x INNER JOIN ",
    table_name!(TableName::TmpImagePaths),
    " t ON t.path = x.path"
);

pub const CREATE_TABLE_TMP_IMAGE_PATHS_STMT: &str = concat!(
    "CREATE TEMP TABLE ",
    table_name!(TableName::TmpImagePaths),
    " (path TEXT PRIMARY KEY)"
);

pub const INSERT_TMP_IMAGE_PATHS_STMT: &str = concat!(
    "INSERT INTO ",
    table_name!(TableName::TmpImagePaths),
    " (path) VALUES (?1)"
);

pub fn table_ensure() -> anyhow::Result<()> {
    let conn = connection()?;
    if !conn.table_exists(None, table_name!(TableName::Image))? {
        conn.execute(CREATE_TABLE_STMT, ())?;
    }
    Ok(())
}

pub fn connection() -> anyhow::Result<Connection> {
    let path = database_file()?;
    Ok(Connection::open(&path)?)
}

fn database_file() -> anyhow::Result<PathBuf> {
    Ok(cache_dir()?.join(DATABASE_FILE))
}
