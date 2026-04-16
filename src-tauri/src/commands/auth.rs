use rusqlite::{params, OptionalExtension};
use serde::Serialize;

use crate::db;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthUser {
    pub id: i64,
    pub full_name: String,
    pub username: String,
    pub role: String,
}

#[tauri::command]
pub fn login(
    app: tauri::AppHandle,
    username: String,
    password: String,
) -> Result<AuthUser, String> {
    let connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;

    let row = connection
        .query_row(
            "
            SELECT u.id, u.full_name, u.username, u.password_hash, r.name
            FROM users u
            INNER JOIN roles r ON r.id = u.role_id
            WHERE u.username = ?1 AND u.is_active = 1
            LIMIT 1
            ",
            params![username],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .optional()
        .map_err(|e| format!("query error: {e}"))?;

    let Some((id, full_name, username, stored_hash, role)) = row else {
        return Err("Invalid username or password".to_string());
    };

    let candidate_hash = db::hash_password(&password);
    if candidate_hash != stored_hash {
        return Err("Invalid username or password".to_string());
    }

    Ok(AuthUser {
        id,
        full_name,
        username,
        role,
    })
}
