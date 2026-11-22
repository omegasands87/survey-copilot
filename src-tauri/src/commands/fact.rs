use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::fact::{
    AddFactSourceInput, ConflictWarning, CreateFactInput, FactSource, FactSummary,
    ProfileFactDetail, UpdateFactInput,
};
use crate::services::FactService;
use tauri::State;

#[tauri::command]
pub fn fact_list(
    persona_id: String,
    include_archived: Option<bool>,
    database: State<'_, Database>,
) -> Result<Vec<FactSummary>, AppError> {
    FactService::list(&database, &persona_id, include_archived.unwrap_or(false))
}

#[tauri::command]
pub fn fact_get_detail(
    persona_id: String,
    fact_id: String,
    database: State<'_, Database>,
) -> Result<ProfileFactDetail, AppError> {
    FactService::get_detail(&database, &persona_id, &fact_id)
}

#[tauri::command]
pub fn fact_create(
    input: CreateFactInput,
    database: State<'_, Database>,
) -> Result<FactSummary, AppError> {
    FactService::create(&database, input)
}

#[tauri::command]
pub fn fact_update(
    input: UpdateFactInput,
    database: State<'_, Database>,
) -> Result<ProfileFactDetail, AppError> {
    FactService::update(&database, input)
}

#[tauri::command]
pub fn fact_set_status(
    persona_id: String,
    fact_id: String,
    status: String,
    database: State<'_, Database>,
) -> Result<ProfileFactDetail, AppError> {
    FactService::set_status(&database, &persona_id, &fact_id, &status)
}

#[tauri::command]
pub fn fact_set_sensitivity(
    persona_id: String,
    fact_id: String,
    sensitivity: String,
    database: State<'_, Database>,
) -> Result<(), AppError> {
    FactService::set_sensitivity(&database, &persona_id, &fact_id, &sensitivity)
}

#[tauri::command]
pub fn fact_add_source(
    input: AddFactSourceInput,
    database: State<'_, Database>,
) -> Result<FactSource, AppError> {
    FactService::add_source(&database, input)
}

#[tauri::command]
pub fn fact_archive(
    persona_id: String,
    fact_id: String,
    database: State<'_, Database>,
) -> Result<(), AppError> {
    FactService::archive(&database, &persona_id, &fact_id)
}

#[tauri::command]
pub fn fact_restore(
    persona_id: String,
    fact_id: String,
    database: State<'_, Database>,
) -> Result<(), AppError> {
    FactService::restore(&database, &persona_id, &fact_id)
}

#[tauri::command]
pub fn fact_delete(
    persona_id: String,
    fact_id: String,
    database: State<'_, Database>,
) -> Result<(), AppError> {
    FactService::delete(&database, &persona_id, &fact_id)
}

#[tauri::command]
pub fn fact_detect_conflicts(
    persona_id: String,
    database: State<'_, Database>,
) -> Result<Vec<ConflictWarning>, AppError> {
    FactService::detect_conflicts(&database, &persona_id)
}

#[tauri::command]
pub fn fact_get_eligible_for_ai(
    persona_id: String,
    allow_sensitive: Option<bool>,
    database: State<'_, Database>,
) -> Result<Vec<FactSummary>, AppError> {
    FactService::get_eligible_facts_for_ai(&database, &persona_id, allow_sensitive.unwrap_or(false))
}
