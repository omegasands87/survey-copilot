use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::fact::{CreateFactInput, FactSummary};
use crate::services::FactService;
use tauri::State;

#[tauri::command]
pub fn fact_list(
    persona_id: String,
    database: State<'_, Database>,
) -> Result<Vec<FactSummary>, AppError> {
    FactService::list(&database, &persona_id)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn fact_create(
    persona_id: String,
    category: String,
    attribute: String,
    value: String,
    source_description: String,
    verification_status: String,
    sensitivity: String,
    database: State<'_, Database>,
) -> Result<FactSummary, AppError> {
    FactService::create(
        &database,
        CreateFactInput {
            persona_id,
            category,
            attribute,
            value,
            source_description,
            verification_status,
            sensitivity,
        },
    )
}

#[tauri::command]
pub fn fact_set_status(
    persona_id: String,
    fact_id: String,
    status: String,
    database: State<'_, Database>,
) -> Result<(), AppError> {
    FactService::set_status(&database, &persona_id, &fact_id, &status)
}

#[tauri::command]
pub fn fact_delete(
    persona_id: String,
    fact_id: String,
    database: State<'_, Database>,
) -> Result<(), AppError> {
    FactService::delete(&database, &persona_id, &fact_id)
}
