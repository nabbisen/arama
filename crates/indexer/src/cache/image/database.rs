use std::path::PathBuf;

use super::path::{DATABASE_FILE, cache_dir};
use rusqlite::Connection;

macro_rules! table_name {
    () => {
        "image"
    };
}

const CREATE_TABLE_STMT: &str = concat!(
    "CREATE TABLE ",
    table_name!(),
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
    table_name!(),
    " (path, last_modified, cache_kind) VALUES (?1, ?2, ?3)"
);

pub const UPDATE_LAST_MODIFIED_STMT: &str = concat!(
    "UPDATE ",
    table_name!(),
    " SET last_modified = ?1, embedding = NULL WHERE id = ?2"
);

pub const UPDATE_EMBEDDING_STMT: &str = concat!(
    "UPDATE ",
    table_name!(),
    " SET embedding = ?1 WHERE id = ?2"
);

pub const SELECT_ROW_BY_PATH_STMT: &str = concat!(
    "SELECT id, path, last_modified, cache_kind, embedding FROM ",
    table_name!(),
    " WHERE path = (?1)"
);

pub const SELECT_ID_BY_PATH_STMT: &str =
    concat!("SELECT id FROM ", table_name!(), " WHERE path = (?1)");

pub const SELECT_EMBEDDING_BY_ID_STMT: &str =
    concat!("SELECT embedding FROM ", table_name!(), " WHERE id = (?1)");

pub fn table_prepare_if_necessary() -> anyhow::Result<()> {
    let conn = connection()?;
    if !conn.table_exists(None, table_name!())? {
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
