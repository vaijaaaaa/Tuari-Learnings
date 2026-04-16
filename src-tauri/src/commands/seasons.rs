use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonListItem {
    pub id: i64,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSeasonInput {
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

fn ensure_admin(role: &str) -> Result<(), String> {
    if role != "admin" {
        return Err("Unauthorized".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn list_seasons(
    app: tauri::AppHandle,
    requester_role: String,
) -> Result<Vec<SeasonListItem>, String> {
    ensure_admin(&requester_role)?;

    let connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;

    let mut stmt = connection
        .prepare(
            "
            SELECT id, name, start_date, end_date, is_active, created_at
            FROM seasons
            ORDER BY id DESC
            ",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let rows = stmt
        .query_map(params![], |row| {
            Ok(SeasonListItem {
                id: row.get::<_, i64>(0)?,
                name: row.get::<_, String>(1)?,
                start_date: row.get::<_, Option<String>>(2)?,
                end_date: row.get::<_, Option<String>>(3)?,
                is_active: row.get::<_, i64>(4)? == 1,
                created_at: row.get::<_, String>(5)?,
            })
        })
        .map_err(|e| format!("query error: {e}"))?;

    let mut seasons: Vec<SeasonListItem> = Vec::new();
    for row in rows {
        seasons.push(row.map_err(|e| format!("row map error: {e}"))?);
    }

    Ok(seasons)
}

#[tauri::command]
pub fn create_season(
    app: tauri::AppHandle,
    requester_role: String,
    input: CreateSeasonInput,
) -> Result<SeasonListItem, String> {
    ensure_admin(&requester_role)?;

    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err("name is required".to_string());
    }

    let start_date = input
        .start_date
        .as_ref()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
    let end_date = input
        .end_date
        .as_ref()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    let connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;

    let exists: Option<i64> = connection
        .query_row(
            "SELECT id FROM seasons WHERE name = ?1 LIMIT 1",
            params![name],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("query error: {e}"))?;

    if exists.is_some() {
        return Err("season name already exists".to_string());
    }

    connection
        .execute(
            "
            INSERT INTO seasons (name, start_date, end_date, is_active)
            VALUES (?1, ?2, ?3, 1)
            ",
            params![name, start_date, end_date],
        )
        .map_err(|e| format!("insert error: {e}"))?;

    let id = connection.last_insert_rowid();

    let season = connection
        .query_row(
            "
            SELECT id, name, start_date, end_date, is_active, created_at
            FROM seasons
            WHERE id = ?1
            LIMIT 1
            ",
            params![id],
            |row| {
                Ok(SeasonListItem {
                    id: row.get::<_, i64>(0)?,
                    name: row.get::<_, String>(1)?,
                    start_date: row.get::<_, Option<String>>(2)?,
                    end_date: row.get::<_, Option<String>>(3)?,
                    is_active: row.get::<_, i64>(4)? == 1,
                    created_at: row.get::<_, String>(5)?,
                })
            },
        )
        .map_err(|e| format!("fetch error: {e}"))?;

    Ok(season)
}
