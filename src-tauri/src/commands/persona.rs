use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::persona::{CreatePersonaInput, PersonaSummary};
use crate::services::PersonaService;
use tauri::State;

#[tauri::command]
pub fn persona_list(database: State<'_, Database>) -> Result<Vec<PersonaSummary>, AppError> {
    PersonaService::list(&database)
}

#[tauri::command]
pub fn persona_create(
    display_name: String,
    database: State<'_, Database>,
) -> Result<PersonaSummary, AppError> {
    PersonaService::create(&database, CreatePersonaInput::new(display_name))
}

#[tauri::command]
pub fn persona_set_active(
    persona_id: String,
    database: State<'_, Database>,
) -> Result<PersonaSummary, AppError> {
    PersonaService::set_active(&database, &persona_id)
}
