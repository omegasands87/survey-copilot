use tauri::State;

use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::opportunity::{
    OpportunityCriterion, OpportunityDetail, SaveOpportunityWithCriteriaInput, SurveyOpportunity,
    UpdateCriteriaInput, UpdateOpportunityInput,
};
use crate::services::OpportunityService;

#[tauri::command]
pub fn opportunity_extract_from_text(
    raw_text: String,
    platform_label: Option<String>,
) -> Result<OpportunityDetail, AppError> {
    OpportunityService::extract_from_text(&raw_text, platform_label.as_deref())
}

#[tauri::command]
pub fn opportunity_extract_from_capture(
    capture_id: String,
    ocr_id: Option<String>,
    database: State<'_, Database>,
) -> Result<OpportunityDetail, AppError> {
    let conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    OpportunityService::extract_from_capture(&conn, &capture_id, ocr_id.as_deref())
}

#[tauri::command]
pub fn opportunity_save(
    input: SaveOpportunityWithCriteriaInput,
    database: State<'_, Database>,
) -> Result<OpportunityDetail, AppError> {
    let mut conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    OpportunityService::save_opportunity(&mut conn, input)
}

#[tauri::command]
pub fn opportunity_get(
    id: String,
    database: State<'_, Database>,
) -> Result<Option<OpportunityDetail>, AppError> {
    let conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    OpportunityService::get_opportunity(&conn, &id)
}

#[tauri::command]
pub fn opportunity_list(
    limit: Option<i64>,
    offset: Option<i64>,
    database: State<'_, Database>,
) -> Result<Vec<SurveyOpportunity>, AppError> {
    let conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    OpportunityService::list_opportunities(&conn, limit, offset)
}

#[tauri::command]
pub fn opportunity_update(
    input: UpdateOpportunityInput,
    database: State<'_, Database>,
) -> Result<SurveyOpportunity, AppError> {
    let conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    OpportunityService::update_opportunity(&conn, input)
}

#[tauri::command]
pub fn opportunity_update_criteria(
    input: UpdateCriteriaInput,
    database: State<'_, Database>,
) -> Result<Vec<OpportunityCriterion>, AppError> {
    let mut conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    OpportunityService::update_criteria(&mut conn, input)
}

#[tauri::command]
pub fn opportunity_delete(id: String, database: State<'_, Database>) -> Result<bool, AppError> {
    let conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    OpportunityService::delete_opportunity(&conn, &id)
}
