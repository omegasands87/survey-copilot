use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::export::{ExportSummary, RetentionPolicyInfo, WipeReport};
use crate::repository::AllRecordCounts;
use crate::services::{DeletionService, ExportService};
use crate::CredentialStoreState;
use tauri::{Manager, State};

#[tauri::command]
pub fn export_get_summary(
    persona_id: Option<String>,
    database: State<'_, Database>,
) -> Result<ExportSummary, AppError> {
    ExportService::get_summary(&database, persona_id.as_deref())
}

#[tauri::command]
pub fn export_generate_json(
    persona_id: Option<String>,
    database: State<'_, Database>,
) -> Result<String, AppError> {
    ExportService::export_data_as_json_string(&database, persona_id.as_deref())
}

#[tauri::command]
pub fn data_get_wipe_summary(database: State<'_, Database>) -> Result<AllRecordCounts, AppError> {
    DeletionService::get_wipe_summary(&database)
}

#[tauri::command]
pub fn data_wipe_all(
    app: tauri::AppHandle,
    database: State<'_, Database>,
    store: State<'_, CredentialStoreState>,
    confirmation_text: String,
) -> Result<WipeReport, AppError> {
    let app_data_dir = app.path().app_data_dir().ok();
    DeletionService::wipe_all_data(
        &database,
        store.0.as_ref(),
        app_data_dir.as_deref(),
        &confirmation_text,
    )
}

#[tauri::command]
pub fn data_get_retention_info() -> Result<RetentionPolicyInfo, AppError> {
    Ok(DeletionService::get_retention_policy_info())
}
