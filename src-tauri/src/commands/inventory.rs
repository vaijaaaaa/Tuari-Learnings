use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItem {
    pub product_id: i64,
    pub sku: String,
    pub name: String,
    pub quantity: i64,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdjustStockInput {
    pub product_id: i64,
    pub quantity_delta: i64,
    pub note: Option<String>,
}

fn ensure_authorized(role: &str) -> Result<(), String> {
    if role != "admin" && role != "user" {
        return Err("Unauthorized".to_string());
    }
    Ok(())
}

fn ensure_admin(role: &str) -> Result<(), String> {
    if role != "admin" {
        return Err("Unauthorized".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn list_inventory(
    app: tauri::AppHandle,
    requester_role: String,
) -> Result<Vec<InventoryItem>, String> {
    ensure_authorized(&requester_role)?;

    let connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;

    let mut stmt = connection
        .prepare(
            "
            SELECT
                p.id,
                p.sku,
                p.name,
                COALESCE(s.quantity, 0) AS quantity,
                s.updated_at
            FROM products p
            LEFT JOIN inventory_stocks s ON s.product_id = p.id
            WHERE p.is_active = 1
            ORDER BY p.name ASC
            ",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let rows = stmt
        .query_map(params![], |row| {
            Ok(InventoryItem {
                product_id: row.get::<_, i64>(0)?,
                sku: row.get::<_, String>(1)?,
                name: row.get::<_, String>(2)?,
                quantity: row.get::<_, i64>(3)?,
                updated_at: row.get::<_, Option<String>>(4)?,
            })
        })
        .map_err(|e| format!("query error: {e}"))?;

    let mut inventory: Vec<InventoryItem> = Vec::new();
    for row in rows {
        inventory.push(row.map_err(|e| format!("row map error: {e}"))?);
    }

    Ok(inventory)
}

#[tauri::command]
pub fn adjust_stock(
    app: tauri::AppHandle,
    requester_role: String,
    requester_username: String,
    input: AdjustStockInput,
) -> Result<InventoryItem, String> {
    ensure_admin(&requester_role)?;

    if input.quantity_delta == 0 {
        return Err("quantityDelta cannot be zero".to_string());
    }

    let mut connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;
    let tx = connection
        .transaction()
        .map_err(|e| format!("transaction error: {e}"))?;

    let exists: Option<i64> = tx
        .query_row(
            "SELECT id FROM products WHERE id = ?1 AND is_active = 1 LIMIT 1",
            params![input.product_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("product check error: {e}"))?;

    if exists.is_none() {
        return Err("invalid productId".to_string());
    }

    tx.execute(
        "INSERT INTO inventory_stocks (product_id, quantity) VALUES (?1, 0)
         ON CONFLICT(product_id) DO NOTHING",
        params![input.product_id],
    )
    .map_err(|e| format!("stock ensure error: {e}"))?;

    let current_qty: i64 = tx
        .query_row(
            "SELECT quantity FROM inventory_stocks WHERE product_id = ?1",
            params![input.product_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("stock fetch error: {e}"))?;

    let new_qty = current_qty + input.quantity_delta;
    if new_qty < 0 {
        return Err("insufficient stock for adjustment".to_string());
    }

    tx.execute(
        "UPDATE inventory_stocks
         SET quantity = ?1, updated_at = CURRENT_TIMESTAMP
         WHERE product_id = ?2",
        params![new_qty, input.product_id],
    )
    .map_err(|e| format!("stock update error: {e}"))?;

    let movement_type = if input.quantity_delta > 0 { "in" } else { "out" };
    tx.execute(
        "
        INSERT INTO stock_movements (
            product_id,
            quantity_delta,
            movement_type,
            reference_type,
            reference_id,
            note,
            created_by
        )
        VALUES (?1, ?2, ?3, 'adjustment', NULL, ?4, ?5)
        ",
        params![
            input.product_id,
            input.quantity_delta,
            movement_type,
            input.note,
            requester_username
        ],
    )
    .map_err(|e| format!("movement insert error: {e}"))?;

    let item = tx
        .query_row(
            "
            SELECT
                p.id,
                p.sku,
                p.name,
                s.quantity,
                s.updated_at
            FROM products p
            INNER JOIN inventory_stocks s ON s.product_id = p.id
            WHERE p.id = ?1
            LIMIT 1
            ",
            params![input.product_id],
            |row| {
                Ok(InventoryItem {
                    product_id: row.get::<_, i64>(0)?,
                    sku: row.get::<_, String>(1)?,
                    name: row.get::<_, String>(2)?,
                    quantity: row.get::<_, i64>(3)?,
                    updated_at: row.get::<_, Option<String>>(4)?,
                })
            },
        )
        .map_err(|e| format!("fetch error: {e}"))?;

    tx.commit().map_err(|e| format!("commit error: {e}"))?;

    Ok(item)
}
