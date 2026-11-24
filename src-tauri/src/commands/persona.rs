use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::persona::{
    CreatePersonaInput, PersonaDependencySummary, PersonaDetail, PersonaSummary,
    ProfileImportResult, UpdatePersonaInput,
};
use crate::services::{PersonaService, ProfileImportService};
use tauri::State;

#[tauri::command]
pub fn persona_list(
    include_archived: Option<bool>,
    database: State<'_, Database>,
) -> Result<Vec<PersonaSummary>, AppError> {
    PersonaService::list(&database, include_archived.unwrap_or(false))
}

#[tauri::command]
pub fn persona_get(
    persona_id: String,
    database: State<'_, Database>,
) -> Result<PersonaDetail, AppError> {
    PersonaService::get_detail(&database, &persona_id)
}

#[tauri::command]
pub fn persona_create(
    display_name: String,
    description: Option<String>,
    database: State<'_, Database>,
) -> Result<PersonaSummary, AppError> {
    PersonaService::create(
        &database,
        CreatePersonaInput::with_description(display_name, description),
    )
}

#[tauri::command]
pub fn persona_import_profile(
    json_content: String,
    display_name: Option<String>,
    database: State<'_, Database>,
) -> Result<ProfileImportResult, AppError> {
    ProfileImportService::import_single_persona_json(
        &database,
        &json_content,
        display_name.as_deref(),
    )
}

#[tauri::command]
pub fn persona_update(
    persona_id: String,
    display_name: String,
    description: Option<String>,
    database: State<'_, Database>,
) -> Result<PersonaDetail, AppError> {
    PersonaService::update(
        &database,
        &persona_id,
        UpdatePersonaInput::new(display_name, description),
    )
}

#[tauri::command]
pub fn persona_set_active(
    persona_id: String,
    database: State<'_, Database>,
) -> Result<PersonaSummary, AppError> {
    PersonaService::set_active(&database, &persona_id)
}

#[tauri::command]
pub fn persona_archive(
    persona_id: String,
    database: State<'_, Database>,
) -> Result<PersonaSummary, AppError> {
    PersonaService::archive(&database, &persona_id)
}

#[tauri::command]
pub fn persona_restore(
    persona_id: String,
    database: State<'_, Database>,
) -> Result<PersonaSummary, AppError> {
    PersonaService::restore(&database, &persona_id)
}

#[tauri::command]
pub fn persona_get_dependency_summary(
    persona_id: String,
    database: State<'_, Database>,
) -> Result<PersonaDependencySummary, AppError> {
    PersonaService::get_dependency_summary(&database, &persona_id)
}

#[tauri::command]
pub fn persona_delete(
    persona_id: String,
    database: State<'_, Database>,
) -> Result<PersonaDependencySummary, AppError> {
    PersonaService::delete(&database, &persona_id)
}
