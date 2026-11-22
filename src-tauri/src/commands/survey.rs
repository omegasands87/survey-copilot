use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::survey::{
    ConfirmAnswerInput, CreateQuestionInput, CreateSessionInput, SaveQuestionDraftInput,
    SurveyQuestion, SurveySession, SurveySessionDetail, SurveySessionSummary, UpdateQuestionInput,
    UpdateSessionInput,
};
use crate::services::SurveyService;
use tauri::State;

// ==========================================
// SURVEY SESSIONS IPC COMMANDS
// ==========================================

#[tauri::command]
pub fn survey_session_create(
    input: CreateSessionInput,
    database: State<'_, Database>,
) -> Result<SurveySession, AppError> {
    SurveyService::db_create_session(&database, input)
}

#[tauri::command]
pub fn survey_session_list(
    persona_id: String,
    include_archived: Option<bool>,
    database: State<'_, Database>,
) -> Result<Vec<SurveySessionSummary>, AppError> {
    SurveyService::db_list_sessions(&database, &persona_id, include_archived.unwrap_or(false))
}

#[tauri::command]
pub fn survey_session_get(
    persona_id: String,
    session_id: String,
    database: State<'_, Database>,
) -> Result<SurveySession, AppError> {
    SurveyService::db_get_session(&database, &persona_id, &session_id)
}

#[tauri::command]
pub fn survey_session_update(
    input: UpdateSessionInput,
    database: State<'_, Database>,
) -> Result<SurveySession, AppError> {
    SurveyService::db_update_session(&database, input)
}

#[tauri::command]
pub fn survey_session_archive(
    persona_id: String,
    session_id: String,
    database: State<'_, Database>,
) -> Result<(), AppError> {
    SurveyService::db_archive_session(&database, &persona_id, &session_id)
}

#[tauri::command]
pub fn survey_session_restore(
    persona_id: String,
    session_id: String,
    database: State<'_, Database>,
) -> Result<(), AppError> {
    SurveyService::db_restore_session(&database, &persona_id, &session_id)
}

#[tauri::command]
pub fn survey_session_delete(
    persona_id: String,
    session_id: String,
    database: State<'_, Database>,
) -> Result<(), AppError> {
    SurveyService::db_delete_session(&database, &persona_id, &session_id)
}

#[tauri::command]
pub fn survey_session_get_detail(
    persona_id: String,
    session_id: String,
    database: State<'_, Database>,
) -> Result<SurveySessionDetail, AppError> {
    SurveyService::db_get_session_detail(&database, &persona_id, &session_id)
}

// ==========================================
// SURVEY QUESTIONS IPC COMMANDS
// ==========================================

#[tauri::command]
pub fn survey_question_create(
    input: CreateQuestionInput,
    database: State<'_, Database>,
) -> Result<SurveyQuestion, AppError> {
    SurveyService::db_create_question(&database, input)
}

#[tauri::command]
pub fn survey_question_get(
    persona_id: String,
    question_id: String,
    database: State<'_, Database>,
) -> Result<SurveyQuestion, AppError> {
    SurveyService::db_get_question(&database, &persona_id, &question_id)
}

#[tauri::command]
pub fn survey_question_list(
    persona_id: String,
    session_id: String,
    database: State<'_, Database>,
) -> Result<Vec<SurveyQuestion>, AppError> {
    SurveyService::db_list_questions(&database, &persona_id, &session_id)
}

#[tauri::command]
pub fn survey_question_update(
    input: UpdateQuestionInput,
    database: State<'_, Database>,
) -> Result<SurveyQuestion, AppError> {
    SurveyService::db_update_question(&database, input)
}

#[tauri::command]
pub fn survey_question_save_draft(
    input: SaveQuestionDraftInput,
    database: State<'_, Database>,
) -> Result<(), AppError> {
    SurveyService::db_save_question_draft(&database, input)
}

#[tauri::command]
pub fn survey_question_confirm_answer(
    input: ConfirmAnswerInput,
    database: State<'_, Database>,
) -> Result<(), AppError> {
    SurveyService::db_confirm_question_answer(&database, input)
}

#[tauri::command]
pub fn survey_question_delete(
    persona_id: String,
    question_id: String,
    database: State<'_, Database>,
) -> Result<(), AppError> {
    SurveyService::db_delete_question(&database, &persona_id, &question_id)
}
