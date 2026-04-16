use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseListItem {
    pub id: i64,
    pub total_amount: f64,
    pub note: Option<String>,
    pub created_by: Option<String>,
    pub item_count: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseSummary {
    pub id: i64,
    pub total_amount: f64,
    pub item_count: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseItemInput {
    pub product_id: i64,
    pub quantity: i64,
    pub unit_cost: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePurchaseInput {
    pub items: Vec<PurchaseItemInput>,
    pub note: Option<String>,
}

fn ensure_authorized(role: &str) -> Result<(), String> {
    if role != "admin" && role != "user" {
        return Err("Unauthorized".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn list_purchases(
    app: tauri::AppHandle,
    requester_role: String,
) -> Result<Vec<PurchaseListItem>, String> {
    ensure_authorized(&requester_role)?;

    let connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;

    let mut stmt = connection
        .prepare(
            "
            SELECT
                p.id,
                p.total_amount,
                p.note,
                p.created_by,
                COALESCE(COUNT(pi.id), 0) AS item_count,
                p.created_at
            FROM purchases p
            LEFT JOIN purchase_items pi ON pi.purchase_id = p.id
            GROUP BY p.id
            ORDER BY p.id DESC
            ",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let rows = stmt
        .query_map(params![], |row| {
            Ok(PurchaseListItem {
                id: row.get::<_, i64>(0)?,
                total_amount: row.get::<_, f64>(1)?,
                note: row.get::<_, Option<String>>(2)?,
                created_by: row.get::<_, Option<String>>(3)?,
                item_count: row.get::<_, i64>(4)?,
                created_at: row.get::<_, String>(5)?,
            })
        })
        .map_err(|e| format!("query error: {e}"))?;

    let mut purchases: Vec<PurchaseListItem> = Vec::new();
    for row in rows {
        purchases.push(row.map_err(|e| format!("row map error: {e}"))?);
    }

    Ok(purchases)
}

#[tauri::command]
pub fn create_purchase(
    app: tauri::AppHandle,
    requester_role: String,
    requester_username: String,
    input: CreatePurchaseInput,
) -> Result<PurchaseSummary, String> {
    ensure_authorized(&requester_role)?;

    if input.items.is_empty() {
        return Err("at least one item is required".to_string());
    }

    let mut connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;
    let tx = connection
        .transaction()
        .map_err(|e| format!("transaction error: {e}"))?;

    tx.execute(
        "INSERT INTO purchases (total_amount, note, created_by) VALUES (0, ?1, ?2)",
        params![input.note, requester_username],
    )
    .map_err(|e| format!("purchase insert error: {e}"))?;

    let purchase_id = tx.last_insert_rowid();
    let mut total_amount: f64 = 0.0;

    for item in &input.items {
        if item.quantity <= 0 {
            return Err("item quantity must be greater than 0".to_string());
        }
        if item.unit_cost < 0.0 {
            return Err("unitCost cannot be negative".to_string());
        }

        let product_exists: Option<i64> = tx
            .query_row(
                "SELECT id FROM products WHERE id = ?1 AND is_active = 1 LIMIT 1",
                params![item.product_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("product check error: {e}"))?;

        if product_exists.is_none() {
            return Err(format!("invalid productId {}", item.product_id));
        }

        let line_total = (item.quantity as f64) * item.unit_cost;
        total_amount += line_total;

        tx.execute(
            "
            INSERT INTO purchase_items (purchase_id, product_id, quantity, unit_cost, line_total)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![purchase_id, item.product_id, item.quantity, item.unit_cost, line_total],
        )
        .map_err(|e| format!("purchase item insert error: {e}"))?;

        tx.execute(
            "
            INSERT INTO inventory_stocks (product_id, quantity)
            VALUES (?1, ?2)
            ON CONFLICT(product_id)
            DO UPDATE SET quantity = inventory_stocks.quantity + excluded.quantity,
                          updated_at = CURRENT_TIMESTAMP
            ",
            params![item.product_id, item.quantity],
        )
        .map_err(|e| format!("inventory update error: {e}"))?;

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
            VALUES (?1, ?2, 'in', 'purchase', ?3, ?4, ?5)
            ",
            params![item.product_id, item.quantity, purchase_id, input.note, requester_username],
        )
        .map_err(|e| format!("movement insert error: {e}"))?;
    }

    tx.execute(
        "UPDATE purchases SET total_amount = ?1 WHERE id = ?2",
        params![total_amount, purchase_id],
    )
    .map_err(|e| format!("purchase update error: {e}"))?;

    let summary = tx
        .query_row(
            "
            SELECT p.id, p.total_amount, COALESCE(COUNT(pi.id), 0), p.created_at
            FROM purchases p
            LEFT JOIN purchase_items pi ON pi.purchase_id = p.id
            WHERE p.id = ?1
            GROUP BY p.id
            ",
            params![purchase_id],
            |row| {
                Ok(PurchaseSummary {
                    id: row.get::<_, i64>(0)?,
                    total_amount: row.get::<_, f64>(1)?,
                    item_count: row.get::<_, i64>(2)?,
                    created_at: row.get::<_, String>(3)?,
                })
            },
        )
        .map_err(|e| format!("fetch error: {e}"))?;

    tx.commit().map_err(|e| format!("commit error: {e}"))?;

    Ok(summary)
}
