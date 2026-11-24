use tauri::{Manager, State};

use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::privacy::{
    CleanupTempFilesInput, DiagnosticsLogEntry, InspectOutboundDataInput, OutboundDataDisclosure,
    PrivacySettings, RecordLogInput, RetentionCleanupResult, RetentionStatus, SystemDiagnosticsInfo,
    UpdatePrivacySettingsInput,
};
use crate::services::PrivacyService;

#[tauri::command]
pub fn privacy_get_settings(database: State<'_, Database>) -> Result<PrivacySettings, AppError> {
    PrivacyService::get_privacy_settings(&database)
}

#[tauri::command]
pub fn privacy_update_settings(
    database: State<'_, Database>,
    input: UpdatePrivacySettingsInput,
) -> Result<PrivacySettings, AppError> {
    PrivacyService::update_privacy_settings(&database, input)
}

#[tauri::command]
pub fn privacy_inspect_outbound_data(
    database: State<'_, Database>,
    input: InspectOutboundDataInput,
) -> Result<OutboundDataDisclosure, AppError> {
    PrivacyService::inspect_outbound_data(&database, input)
}

#[tauri::command]
pub fn privacy_get_retention_status(
    app: tauri::AppHandle,
    database: State<'_, Database>,
) -> Result<RetentionStatus, AppError> {
    let app_data_dir = app.path().app_data_dir().ok();
    PrivacyService::get_retention_status(&database, app_data_dir.as_deref())
}

#[tauri::command]
pub fn privacy_cleanup_temp_files(
    app: tauri::AppHandle,
    database: State<'_, Database>,
    input: CleanupTempFilesInput,
) -> Result<RetentionCleanupResult, AppError> {
    let app_data_dir = app.path().app_data_dir().ok();
    PrivacyService::cleanup_temp_files(&database, app_data_dir.as_deref(), input)
}

#[tauri::command]
pub fn privacy_get_diagnostic_logs(
    database: State<'_, Database>,
    limit: Option<usize>,
    level: Option<String>,
    category: Option<String>,
) -> Result<Vec<DiagnosticsLogEntry>, AppError> {
    PrivacyService::list_logs(&database, limit, level, category)
}

#[tauri::command]
pub fn privacy_clear_diagnostic_logs(database: State<'_, Database>) -> Result<usize, AppError> {
    PrivacyService::clear_logs(&database)
}

#[tauri::command]
pub fn privacy_get_system_diagnostics(
    app: tauri::AppHandle,
    database: State<'_, Database>,
) -> Result<SystemDiagnosticsInfo, AppError> {
    let app_data_dir = app.path().app_data_dir().ok();
    PrivacyService::get_system_diagnostics(&database, app_data_dir.as_deref())
}

#[tauri::command]
pub fn privacy_record_diagnostic_log(
    database: State<'_, Database>,
    input: RecordLogInput,
) -> Result<DiagnosticsLogEntry, AppError> {
    PrivacyService::record_log(&database, input)
}
