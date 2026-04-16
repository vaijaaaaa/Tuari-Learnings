mod db;
mod commands;

use commands::analytics::dashboard_summary;
use commands::auth::login;
use commands::inventory::{adjust_stock, list_inventory};
use commands::products::{create_product, list_products};
use commands::purchase::{create_purchase, list_purchases};
use commands::sales::{create_sale, list_sales};
use commands::seasons::{create_season, list_seasons};
use commands::users::{create_user, list_users};

#[tauri::command]
fn health_check() -> String {
    "ok".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            health_check,
            login,
            list_users,
            create_user,
            list_products,
            create_product,
            list_seasons,
            create_season,
            list_inventory,
            adjust_stock,
            list_purchases,
            create_purchase,
            list_sales,
            create_sale,
            dashboard_summary
        ])
        .setup(|app| {
            let _connection = db::open_database(&app.handle()).expect("failed to open database");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}