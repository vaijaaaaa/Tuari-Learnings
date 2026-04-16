use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleListItem {
    pub id: i64,
    pub total_amount: f64,
    pub note: Option<String>,
    pub created_by: Option<String>,
    pub item_count: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleSummary {
    pub id: i64,
    pub total_amount: f64,
    pub item_count: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleItemInput {
    pub product_id: i64,
    pub quantity: i64,
    pub unit_price: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSaleInput {
    pub items: Vec<SaleItemInput>,
    pub note: Option<String>,
}

fn ensure_authorized(role: &str) -> Result<(), String> {
    if role != "admin" && role != "user" {
        return Err("Unauthorized".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn list_sales(app: tauri::AppHandle, requester_role: String) -> Result<Vec<SaleListItem>, String> {
    ensure_authorized(&requester_role)?;

    let connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;

    let mut stmt = connection
        .prepare(
            "
            SELECT
                s.id,
                s.total_amount,
                s.note,
                s.created_by,
                COALESCE(COUNT(si.id), 0) AS item_count,
                s.created_at
            FROM sales s
            LEFT JOIN sale_items si ON si.sale_id = s.id
            GROUP BY s.id
            ORDER BY s.id DESC
            ",
        )
        .map_err(|e| format!("prepare error: {e}"))?;

    let rows = stmt
        .query_map(params![], |row| {
            Ok(SaleListItem {
                id: row.get::<_, i64>(0)?,
                total_amount: row.get::<_, f64>(1)?,
                note: row.get::<_, Option<String>>(2)?,
                created_by: row.get::<_, Option<String>>(3)?,
                item_count: row.get::<_, i64>(4)?,
                created_at: row.get::<_, String>(5)?,
            })
        })
        .map_err(|e| format!("query error: {e}"))?;

    let mut sales: Vec<SaleListItem> = Vec::new();
    for row in rows {
        sales.push(row.map_err(|e| format!("row map error: {e}"))?);
    }

    Ok(sales)
}

#[tauri::command]
pub fn create_sale(
    app: tauri::AppHandle,
    requester_role: String,
    requester_username: String,
    input: CreateSaleInput,
) -> Result<SaleSummary, String> {
    ensure_authorized(&requester_role)?;

    if input.items.is_empty() {
        return Err("at least one item is required".to_string());
    }

    let mut connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;
    let tx = connection
        .transaction()
        .map_err(|e| format!("transaction error: {e}"))?;

    tx.execute(
        "INSERT INTO sales (total_amount, note, created_by) VALUES (0, ?1, ?2)",
        params![input.note, requester_username],
    )
    .map_err(|e| format!("sale insert error: {e}"))?;

    let sale_id = tx.last_insert_rowid();
    let mut total_amount: f64 = 0.0;

    for item in &input.items {
        if item.quantity <= 0 {
            return Err("item quantity must be greater than 0".to_string());
        }
        if item.unit_price < 0.0 {
            return Err("unitPrice cannot be negative".to_string());
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

        tx.execute(
            "INSERT INTO inventory_stocks (product_id, quantity) VALUES (?1, 0)
             ON CONFLICT(product_id) DO NOTHING",
            params![item.product_id],
        )
        .map_err(|e| format!("inventory ensure error: {e}"))?;

        let current_qty: i64 = tx
            .query_row(
                "SELECT quantity FROM inventory_stocks WHERE product_id = ?1",
                params![item.product_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("stock fetch error: {e}"))?;

        if current_qty < item.quantity {
            return Err(format!(
                "insufficient stock for productId {} (available {}, required {})",
                item.product_id, current_qty, item.quantity
            ));
        }

        let line_total = (item.quantity as f64) * item.unit_price;
        total_amount += line_total;

        tx.execute(
            "
            INSERT INTO sale_items (sale_id, product_id, quantity, unit_price, line_total)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ",
            params![sale_id, item.product_id, item.quantity, item.unit_price, line_total],
        )
        .map_err(|e| format!("sale item insert error: {e}"))?;

        tx.execute(
            "
            UPDATE inventory_stocks
            SET quantity = quantity - ?1, updated_at = CURRENT_TIMESTAMP
            WHERE product_id = ?2
            ",
            params![item.quantity, item.product_id],
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
            VALUES (?1, ?2, 'out', 'sale', ?3, ?4, ?5)
            ",
            params![
                item.product_id,
                -item.quantity,
                sale_id,
                input.note,
                requester_username
            ],
        )
        .map_err(|e| format!("movement insert error: {e}"))?;
    }

    tx.execute(
        "UPDATE sales SET total_amount = ?1 WHERE id = ?2",
        params![total_amount, sale_id],
    )
    .map_err(|e| format!("sale update error: {e}"))?;

    let summary = tx
        .query_row(
            "
            SELECT s.id, s.total_amount, COALESCE(COUNT(si.id), 0), s.created_at
            FROM sales s
            LEFT JOIN sale_items si ON si.sale_id = s.id
            WHERE s.id = ?1
            GROUP BY s.id
            ",
            params![sale_id],
            |row| {
                Ok(SaleSummary {
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
