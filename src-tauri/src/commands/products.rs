use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductListItem {
    pub id: i64,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub season_id: Option<i64>,
    pub season_name: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductInput {
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub season_id: Option<i64>,
}

fn ensure_admin(role: &str) -> Result<(), String> {
    if role != "admin" {
        return Err("Unauthorized".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn list_products(
    app: tauri::AppHandle,
    requester_role: String,
) -> Result<Vec<ProductListItem>, String> {
    ensure_admin(&requester_role)?;

    let connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;

    let mut stmt = connection
        .prepare(
            "
            SELECT
                p.id,
                p.sku,
                p.name,
                p.description,
                p.season_id,
                s.name AS season_name,
                p.is_active,
                p.created_at
            FROM products p
            LEFT JOIN seasons s ON s.id = p.season_id
            ORDER BY p.id DESC
            ",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let rows = stmt
        .query_map(params![], |row| {
            Ok(ProductListItem {
                id: row.get::<_, i64>(0)?,
                sku: row.get::<_, String>(1)?,
                name: row.get::<_, String>(2)?,
                description: row.get::<_, Option<String>>(3)?,
                season_id: row.get::<_, Option<i64>>(4)?,
                season_name: row.get::<_, Option<String>>(5)?,
                is_active: row.get::<_, i64>(6)? == 1,
                created_at: row.get::<_, String>(7)?,
            })
        })
        .map_err(|e| format!("query error: {e}"))?;

    let mut products: Vec<ProductListItem> = Vec::new();
    for row in rows {
        products.push(row.map_err(|e| format!("row map error: {e}"))?);
    }

    Ok(products)
}

#[tauri::command]
pub fn create_product(
    app: tauri::AppHandle,
    requester_role: String,
    input: CreateProductInput,
) -> Result<ProductListItem, String> {
    ensure_admin(&requester_role)?;

    let sku = input.sku.trim().to_uppercase();
    let name = input.name.trim().to_string();
    let description = input
        .description
        .as_ref()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    if sku.is_empty() || name.is_empty() {
        return Err("sku and name are required".to_string());
    }

    let connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;

    let sku_exists: Option<i64> = connection
        .query_row(
            "SELECT id FROM products WHERE sku = ?1 LIMIT 1",
            params![sku],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("query error: {e}"))?;

    if sku_exists.is_some() {
        return Err("sku already exists".to_string());
    }

    if let Some(season_id) = input.season_id {
        let season_exists: Option<i64> = connection
            .query_row(
                "SELECT id FROM seasons WHERE id = ?1 LIMIT 1",
                params![season_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("season check error: {e}"))?;

        if season_exists.is_none() {
            return Err("invalid seasonId".to_string());
        }
    }

    connection
        .execute(
            "
            INSERT INTO products (sku, name, description, season_id, is_active)
            VALUES (?1, ?2, ?3, ?4, 1)
            ",
            params![sku, name, description, input.season_id],
        )
        .map_err(|e| format!("insert error: {e}"))?;

    let id = connection.last_insert_rowid();

    let product = connection
        .query_row(
            "
            SELECT
                p.id,
                p.sku,
                p.name,
                p.description,
                p.season_id,
                s.name AS season_name,
                p.is_active,
                p.created_at
            FROM products p
            LEFT JOIN seasons s ON s.id = p.season_id
            WHERE p.id = ?1
            LIMIT 1
            ",
            params![id],
            |row| {
                Ok(ProductListItem {
                    id: row.get::<_, i64>(0)?,
                    sku: row.get::<_, String>(1)?,
                    name: row.get::<_, String>(2)?,
                    description: row.get::<_, Option<String>>(3)?,
                    season_id: row.get::<_, Option<i64>>(4)?,
                    season_name: row.get::<_, Option<String>>(5)?,
                    is_active: row.get::<_, i64>(6)? == 1,
                    created_at: row.get::<_, String>(7)?,
                })
            },
        )
        .map_err(|e| format!("fetch error: {e}"))?;

    Ok(product)
}
