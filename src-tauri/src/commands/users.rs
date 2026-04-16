use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserInput {
    pub full_name: String,
    pub username: String,
    pub password: String,
    pub role: String,
}

fn ensure_admin(role: &str) -> Result<(), String> {
    if role != "admin" {
        return Err("Unauthorized".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn list_users(app: tauri::AppHandle, requester_role: String) -> Result<Vec<UserListItem>, String> {
    ensure_admin(&requester_role)?;

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

#[tauri::command]
pub fn create_user(
    app: tauri::AppHandle,
    requester_role: String,
    input: CreateUserInput,
) -> Result<UserListItem, String> {
    ensure_admin(&requester_role)?;

    let full_name = input.full_name.trim();
    let username = input.username.trim().to_lowercase();
    let password = input.password.trim();
    let role = input.role.trim().to_lowercase();

    if full_name.is_empty() || username.is_empty() || password.is_empty() {
        return Err("fullName, username, and password are required".to_string());
    }

    if role != "admin" && role != "user" {
        return Err("role must be admin or user".to_string());
    }

    let connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;

    let username_exists: Option<i64> = connection
        .query_row(
            "SELECT id FROM users WHERE username = ?1 LIMIT 1",
            params![username],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("query error: {e}"))?;

    if username_exists.is_some() {
        return Err("username already exists".to_string());
    }

    let role_id: i64 = connection
        .query_row(
            "SELECT id FROM roles WHERE name = ?1 LIMIT 1",
            params![role],
            |row| row.get(0),
        )
        .map_err(|e| format!("role lookup error: {e}"))?;

    let password_hash = db::hash_password(password);

    connection
        .execute(
            "
            INSERT INTO users (full_name, username, password_hash, role_id, is_active)
            VALUES (?1, ?2, ?3, ?4, 1)
            ",
            params![full_name, username, password_hash, role_id],
        )
        .map_err(|e| format!("insert error: {e}"))?;

    let id = connection.last_insert_rowid();

    let user = connection
        .query_row(
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
            WHERE u.id = ?1
            LIMIT 1
            ",
            params![id],
            |row| {
                Ok(UserListItem {
                    id: row.get::<_, i64>(0)?,
                    full_name: row.get::<_, String>(1)?,
                    username: row.get::<_, String>(2)?,
                    role: row.get::<_, String>(3)?,
                    is_active: row.get::<_, i64>(4)? == 1,
                    created_at: row.get::<_, String>(5)?,
                })
            },
        )
        .map_err(|e| format!("fetch error: {e}"))?;

    Ok(user)
}