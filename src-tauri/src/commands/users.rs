use rusqlite::params;
use serde::Serialize;

use crate::db;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListItem {
    pub id: i64,
    pub full_name: String,
    pub username: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: String,
}

#[tauri::command]
pub fn list_users(app: tauri::AppHandle, requester_role: String) -> Result<Vec<UserListItem>, String> {
    if requester_role != "admin" {
        return Err("Unauthorized".to_string());
    }

    let connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;

    let mut stmt = connection
        .prepare(
            "
            SELECT
                u.id,
                u.full_name,
                u.username,
                r.name AS role_name,
                u.is_active,
                u.created_at
            FROM users u
            INNER JOIN roles r ON r.id = u.role_id
            ORDER BY u.id DESC
            ",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let rows = stmt
        .query_map(params![], |row| {
            Ok(UserListItem {
                id: row.get::<_, i64>(0)?,
                full_name: row.get::<_, String>(1)?,
                username: row.get::<_, String>(2)?,
                role: row.get::<_, String>(3)?,
                is_active: row.get::<_, i64>(4)? == 1,
                created_at: row.get::<_, String>(5)?,
            })
        })
        .map_err(|e| format!("query error: {e}"))?;

    let mut users: Vec<UserListItem> = Vec::new();
    for row in rows {
        users.push(row.map_err(|e| format!("row map error: {e}"))?);
    }

    Ok(users)
}