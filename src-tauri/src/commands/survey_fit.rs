use tauri::State;

use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::fit::{PersonaFitComparisonItem, PersonaOpportunityFitDetail};
use crate::services::FitService;

#[tauri::command]
pub fn survey_fit_evaluate(
    opportunity_id: String,
    persona_id: String,
    database: State<'_, Database>,
) -> Result<PersonaOpportunityFitDetail, AppError> {
    let mut conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    FitService::evaluate_opportunity_fit(&mut conn, &opportunity_id, &persona_id)
}

#[tauri::command]
pub fn survey_fit_get_results(
    opportunity_id: String,
    persona_id: String,
    database: State<'_, Database>,
) -> Result<Option<PersonaOpportunityFitDetail>, AppError> {
    let conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    FitService::get_opportunity_fit(&conn, &opportunity_id, &persona_id)
}

#[tauri::command]
pub fn survey_fit_compare_personas(
    opportunity_id: String,
    database: State<'_, Database>,
) -> Result<Vec<PersonaFitComparisonItem>, AppError> {
    let mut conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    FitService::compare_personas_fit(&mut conn, &opportunity_id)
}
