use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::pool::{
    PoolExecutionResult, PoolSettings, ProviderHealthInfo, UpdatePoolSettingsInput,
};
use crate::domain::provider::NormalizedRequest;
use crate::{CredentialStoreState, HttpTransportState, ProviderPoolState};
use tauri::State;

#[tauri::command]
pub fn provider_pool_get_settings(
    database: State<'_, Database>,
    pool_state: State<'_, ProviderPoolState>,
) -> Result<PoolSettings, AppError> {
    pool_state.0.get_settings(&database)
}

#[tauri::command]
pub fn provider_pool_update_settings(
    database: State<'_, Database>,
    pool_state: State<'_, ProviderPoolState>,
    input: UpdatePoolSettingsInput,
) -> Result<PoolSettings, AppError> {
    pool_state.0.update_settings(&database, input)
}

#[tauri::command]
pub fn provider_pool_list_health(
    database: State<'_, Database>,
    pool_state: State<'_, ProviderPoolState>,
) -> Result<Vec<ProviderHealthInfo>, AppError> {
    pool_state.0.list_health(&database)
}

#[tauri::command]
pub fn provider_pool_execute_request(
    database: State<'_, Database>,
    store: State<'_, CredentialStoreState>,
    transport: State<'_, HttpTransportState>,
    pool_state: State<'_, ProviderPoolState>,
    request: NormalizedRequest,
) -> Result<PoolExecutionResult, AppError> {
    Ok(pool_state.0.execute_pooled_request(
        &database,
        store.0.as_ref(),
        transport.0.clone(),
        request,
        None, // Cancellation token can be injected via Tauri event/channel if extended
    ))
}
