pub mod commands;
pub mod db;
pub mod domain;
pub mod infrastructure;
pub mod migrations;
pub mod repository;
pub mod services;

#[derive(Clone)]
pub struct CredentialStoreState(
    pub std::sync::Arc<dyn infrastructure::credentials::CredentialStore>,
);

#[derive(Clone)]
pub struct HttpTransportState(pub std::sync::Arc<dyn infrastructure::http::HttpTransport>);

use commands::{
    app_status, credential_delete, credential_get_status, credential_list, credential_save,
    credential_test, credential_update, data_get_retention_info, data_get_wipe_summary,
    data_wipe_all, export_generate_json, export_get_summary, fact_add_source, fact_archive,
    fact_create, fact_delete, fact_detect_conflicts, fact_get_detail, fact_get_eligible_for_ai,
    fact_list, fact_restore, fact_set_sensitivity, fact_set_status, fact_update, persona_archive,
    persona_create, persona_delete, persona_get, persona_get_dependency_summary, persona_list,
    persona_restore, persona_set_active, persona_update, provider_config_create,
    provider_config_delete, provider_config_get, provider_config_list,
    provider_config_test_connection, provider_config_update, provider_get_recommended_models,
};
use db::Database;
use std::fs;
use std::sync::Arc;
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

            let credential_store: Arc<dyn infrastructure::credentials::CredentialStore> =
                Arc::new(infrastructure::credentials::WindowsCredentialStore::new());
            app.manage(CredentialStoreState(credential_store));

            let http_transport: Arc<dyn infrastructure::http::HttpTransport> =
                Arc::new(infrastructure::http::DefaultHttpTransport::new());
            app.manage(HttpTransportState(http_transport));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_status,
            persona_list,
            persona_get,
            persona_create,
            persona_update,
            persona_set_active,
            persona_archive,
            persona_restore,
            persona_get_dependency_summary,
            persona_delete,
            fact_list,
            fact_get_detail,
            fact_create,
            fact_update,
            fact_set_status,
            fact_set_sensitivity,
            fact_add_source,
            fact_archive,
            fact_restore,
            fact_delete,
            fact_detect_conflicts,
            fact_get_eligible_for_ai,
            export_get_summary,
            export_generate_json,
            data_get_wipe_summary,
            data_wipe_all,
            data_get_retention_info,
            credential_save,
            credential_list,
            credential_get_status,
            credential_update,
            credential_delete,
            credential_test,
            provider_config_create,
            provider_config_get,
            provider_config_list,
            provider_config_update,
            provider_config_delete,
            provider_config_test_connection,
            provider_get_recommended_models
        ])
        .run(tauri::generate_context!())
        .expect("error while running Survey Copilot");
}
