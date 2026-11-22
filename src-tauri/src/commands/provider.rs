use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::provider::{
    ConnectionTestResult, CreateProviderConfigInput, ModelOption, ProviderConfigSummary,
    UpdateProviderConfigInput,
};
use crate::services::ProviderService;
use crate::{CredentialStoreState, HttpTransportState};
use tauri::State;

#[tauri::command]
pub fn provider_config_create(
    database: State<'_, Database>,
    input: CreateProviderConfigInput,
) -> Result<ProviderConfigSummary, AppError> {
    ProviderService::create_provider_config(&database, input)
}

#[tauri::command]
pub fn provider_config_get(
    database: State<'_, Database>,
    id: String,
) -> Result<ProviderConfigSummary, AppError> {
    ProviderService::get_provider_config(&database, &id)
}

#[tauri::command]
pub fn provider_config_list(
    database: State<'_, Database>,
) -> Result<Vec<ProviderConfigSummary>, AppError> {
    ProviderService::list_provider_configs(&database)
}

#[tauri::command]
pub fn provider_config_update(
    database: State<'_, Database>,
    input: UpdateProviderConfigInput,
) -> Result<ProviderConfigSummary, AppError> {
    ProviderService::update_provider_config(&database, input)
}

#[tauri::command]
pub fn provider_config_delete(database: State<'_, Database>, id: String) -> Result<bool, AppError> {
    ProviderService::delete_provider_config(&database, &id)
}

#[tauri::command]
pub fn provider_config_test_connection(
    database: State<'_, Database>,
    store: State<'_, CredentialStoreState>,
    transport: State<'_, HttpTransportState>,
    id: String,
) -> Result<ConnectionTestResult, AppError> {
    ProviderService::test_provider_connection(&database, store.0.as_ref(), transport.0.clone(), &id)
}

#[tauri::command]
pub fn provider_get_recommended_models(
    provider_type: String,
) -> Result<Vec<ModelOption>, AppError> {
    Ok(ProviderService::get_recommended_models(&provider_type))
}
