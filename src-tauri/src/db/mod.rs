use std::path::PathBuf;

use rusqlite::{Connection, Result};
use tauri::Manager;

pub fn database_path(app: &tauri::AppHandle) -> Result<PathBuf> {
    let mut path = app
        .path()
        .app_data_dir()
        .expect("failed to resolve app data directory");

    path.push("framstack-poc.db");
    Ok(path)
}

pub fn open_database(app: &tauri::AppHandle) -> Result<Connection> {
    let path = database_path(app)?;
    let connection = Connection::open(path)?;

    connection.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        ",
    )?;

    Ok(connection)
}