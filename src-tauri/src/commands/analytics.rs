use rusqlite::params;
use serde::Serialize;

use crate::db;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub total_products: i64,
    pub low_stock_count: i64,
    pub total_stock_units: i64,
    pub total_sales_amount: f64,
    pub total_purchase_amount: f64,
}

fn ensure_authorized(role: &str) -> Result<(), String> {
    if role != "admin" && role != "user" {
        return Err("Unauthorized".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn dashboard_summary(
    app: tauri::AppHandle,
    requester_role: String,
) -> Result<DashboardSummary, String> {
    ensure_authorized(&requester_role)?;

    let connection = db::open_database(&app).map_err(|e| format!("database error: {e}"))?;

    let total_products: i64 = connection
        .query_row(
            "SELECT COUNT(1) FROM products WHERE is_active = 1",
            params![],
            |row| row.get(0),
        )
        .map_err(|e| format!("products query error: {e}"))?;

    let low_stock_count: i64 = connection
        .query_row(
            "
            SELECT COUNT(1)
            FROM products p
            LEFT JOIN inventory_stocks s ON s.product_id = p.id
            WHERE p.is_active = 1 AND COALESCE(s.quantity, 0) <= 5
            ",
            params![],
            |row| row.get(0),
        )
        .map_err(|e| format!("low stock query error: {e}"))?;

    let total_stock_units: i64 = connection
        .query_row(
            "SELECT COALESCE(SUM(quantity), 0) FROM inventory_stocks",
            params![],
            |row| row.get(0),
        )
        .map_err(|e| format!("stock sum query error: {e}"))?;

    let total_sales_amount: f64 = connection
        .query_row(
            "SELECT COALESCE(SUM(total_amount), 0) FROM sales",
            params![],
            |row| row.get(0),
        )
        .map_err(|e| format!("sales sum query error: {e}"))?;

    let total_purchase_amount: f64 = connection
        .query_row(
            "SELECT COALESCE(SUM(total_amount), 0) FROM purchases",
            params![],
            |row| row.get(0),
        )
        .map_err(|e| format!("purchase sum query error: {e}"))?;

    Ok(DashboardSummary {
        total_products,
        low_stock_count,
        total_stock_units,
        total_sales_amount,
        total_purchase_amount,
    })
}
