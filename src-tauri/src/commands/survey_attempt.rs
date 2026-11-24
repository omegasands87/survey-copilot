use tauri::State;

use crate::db::Database;
use crate::domain::attempt::{
    AttemptFilter, CreateAttemptInput, HistoricalLearningSummary, RecordOutcomeEventInput,
    SurveyAttemptDetail,
};
use crate::domain::error::AppError;
use crate::services::OutcomeLearningService;

#[tauri::command]
pub fn survey_attempt_record(
    input: CreateAttemptInput,
    database: State<'_, Database>,
) -> Result<SurveyAttemptDetail, AppError> {
    let mut conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    OutcomeLearningService::record_attempt(&mut conn, input)
}

#[tauri::command]
pub fn survey_attempt_record_outcome(
    input: RecordOutcomeEventInput,
    database: State<'_, Database>,
) -> Result<SurveyAttemptDetail, AppError> {
    let mut conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    OutcomeLearningService::record_outcome_event(&mut conn, input)
}

#[tauri::command]
pub fn survey_attempt_get(
    id: String,
    persona_id: Option<String>,
    database: State<'_, Database>,
) -> Result<SurveyAttemptDetail, AppError> {
    let conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    OutcomeLearningService::get_attempt_detail(&conn, &id, persona_id.as_deref())
}

#[tauri::command]
pub fn survey_attempt_list(
    filter: AttemptFilter,
    database: State<'_, Database>,
) -> Result<Vec<SurveyAttemptDetail>, AppError> {
    let conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    OutcomeLearningService::list_attempts(&conn, filter)
}

#[tauri::command]
pub fn survey_attempt_delete(
    id: String,
    persona_id: String,
    database: State<'_, Database>,
) -> Result<bool, AppError> {
    let conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    OutcomeLearningService::delete_attempt(&conn, &id, &persona_id)
}

#[tauri::command]
pub fn survey_attempt_get_learning_summary(
    persona_id: String,
    category: Option<String>,
    source_platform: Option<String>,
    database: State<'_, Database>,
) -> Result<HistoricalLearningSummary, AppError> {
    let conn = database
        .connection
        .lock()
        .map_err(|e| AppError::database(format!("Gagal mengunci basis data: {e}")))?;
    OutcomeLearningService::evaluate_historical_learning(
        &conn,
        &persona_id,
        category.as_deref(),
        source_platform.as_deref(),
    )
}
