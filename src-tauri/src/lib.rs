mod db;
mod commands;

use commands::auth::login;
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
            create_user
        ])
        .setup(|app| {
            let _connection = db::open_database(&app.handle()).expect("failed to open database");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}