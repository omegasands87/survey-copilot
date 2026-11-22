pub mod commands;
pub mod db;
pub mod domain;
pub mod repository;
pub mod services;

use commands::{
    app_status, fact_create, fact_delete, fact_list, fact_set_status, persona_create, persona_list,
    persona_set_active,
};
use db::Database;
use std::fs;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&app_data_dir)?;
            let database_path = app_data_dir.join("survey-copilot.sqlite3");
            let database = Database::initialize(&database_path).map_err(std::io::Error::other)?;
            app.manage(database);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_status,
            persona_list,
            persona_create,
            persona_set_active,
            fact_list,
            fact_create,
            fact_set_status,
            fact_delete
        ])
        .run(tauri::generate_context!())
        .expect("error while running Survey Copilot");
}
