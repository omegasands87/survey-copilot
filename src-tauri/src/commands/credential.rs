use crate::db::Database;
use crate::domain::credential::{
    CredentialSummary, DeleteCredentialResult, SaveCredentialInput, TestCredentialResult,
    UpdateCredentialInput,
};
use crate::domain::error::AppError;
use crate::services::CredentialService;
use crate::CredentialStoreState;
use tauri::State;

#[tauri::command]
pub fn credential_save(
    database: State<'_, Database>,
    store: State<'_, CredentialStoreState>,
    input: SaveCredentialInput,
) -> Result<CredentialSummary, AppError> {
    CredentialService::save_credential(&database, store.0.as_ref(), input)
}

#[tauri::command]
pub fn credential_list(database: State<'_, Database>) -> Result<Vec<CredentialSummary>, AppError> {
    CredentialService::list_credentials(&database)
}

#[tauri::command]
pub fn credential_get_status(
    database: State<'_, Database>,
    store: State<'_, CredentialStoreState>,
    id: String,
) -> Result<CredentialSummary, AppError> {
    CredentialService::get_credential_status(&database, store.0.as_ref(), &id)
}

#[tauri::command]
pub fn credential_update(
    database: State<'_, Database>,
    store: State<'_, CredentialStoreState>,
    input: UpdateCredentialInput,
) -> Result<CredentialSummary, AppError> {
    CredentialService::update_credential(&database, store.0.as_ref(), input)
}

#[tauri::command]
pub fn credential_delete(
    database: State<'_, Database>,
    store: State<'_, CredentialStoreState>,
    id: String,
    force: Option<bool>,
) -> Result<DeleteCredentialResult, AppError> {
    CredentialService::delete_credential(&database, store.0.as_ref(), &id, force.unwrap_or(false))
}

#[tauri::command]
pub fn credential_test(
    database: State<'_, Database>,
    store: State<'_, CredentialStoreState>,
    id: String,
) -> Result<TestCredentialResult, AppError> {
    CredentialService::test_credential(&database, store.0.as_ref(), &id)
}
