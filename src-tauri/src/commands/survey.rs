use crate::db::Database;
use crate::domain::answer_engine::{
    AnswerHistoryItem, ConfirmRecommendationInput, CopyAnswerInput, EditAnswerInput,
    GenerateAnswerInput, ListAnswerHistoryInput, RegenerateAnswerInput, RejectAnswerInput,
    ValidatedAnswerRecommendation,
};
use crate::domain::classifier::{ClassifyQuestionInput, QuestionClassificationResult};
use crate::domain::error::AppError;
use crate::domain::evidence::{EvidencePackage, EvidenceRetrievalInput};
use crate::domain::survey::{
    ConfirmAnswerInput, CreateQuestionInput, CreateSessionInput, NextQuestionIntelligenceResult,
    PredictNextQuestionInput, RecognizeSurveyInput, SaveQuestionDraftInput, SurveyBranchTransition,
    SurveyMemorySessionDetail, SurveyQuestion, SurveyQuestionPredictionRecord,
    SurveyRecognitionEvaluation, SurveySession, SurveySessionDetail, SurveySessionSummary,
    UpdateQuestionInput, UpdateSessionInput,
};
use crate::repository::AnswerRepository;
use crate::services::{AnswerEngineService, SurveyService};
use crate::{CredentialStoreState, HttpTransportState, ProviderPoolState};
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

#[tauri::command]
pub fn survey_question_classify(
    input: ClassifyQuestionInput,
    database: State<'_, Database>,
) -> Result<QuestionClassificationResult, AppError> {
    SurveyService::db_classify_question(&database, input)
}

#[tauri::command]
pub fn survey_evidence_evaluate(
    input: EvidenceRetrievalInput,
    database: State<'_, Database>,
) -> Result<EvidencePackage, AppError> {
    SurveyService::db_evaluate_evidence(&database, input)
}

#[tauri::command]
pub fn survey_answer_generate(
    input: GenerateAnswerInput,
    database: State<'_, Database>,
    store: State<'_, CredentialStoreState>,
    transport: State<'_, HttpTransportState>,
    pool_state: State<'_, ProviderPoolState>,
) -> Result<ValidatedAnswerRecommendation, AppError> {
    AnswerEngineService::generate_recommendation(
        &database,
        store.0.as_ref(),
        transport.0.clone(),
        pool_state.0.as_ref(),
        input,
        None,
    )
}

#[tauri::command]
pub fn survey_answer_get_latest(
    persona_id: String,
    question_id: String,
    database: State<'_, Database>,
) -> Result<Option<ValidatedAnswerRecommendation>, AppError> {
    let conn = database
        .connection
        .lock()
        .map_err(|_| AppError::database("Basis data sedang tidak tersedia."))?;

    AnswerRepository::get_latest_recommendation_by_question(&conn, &persona_id, &question_id)
}

// ==========================================
// REVIEW ACTIONS & HISTORY IPC COMMANDS (SC-034)
// ==========================================

#[tauri::command]
pub fn survey_answer_edit(
    input: EditAnswerInput,
    database: State<'_, Database>,
) -> Result<AnswerHistoryItem, AppError> {
    AnswerEngineService::edit_answer(&database, input)
}

#[tauri::command]
pub fn survey_answer_confirm(
    input: ConfirmRecommendationInput,
    database: State<'_, Database>,
) -> Result<AnswerHistoryItem, AppError> {
    AnswerEngineService::confirm_answer(&database, input)
}

#[tauri::command]
pub fn survey_answer_copy(
    input: CopyAnswerInput,
    database: State<'_, Database>,
) -> Result<AnswerHistoryItem, AppError> {
    AnswerEngineService::copy_answer(&database, input)
}

#[tauri::command]
pub fn survey_answer_reject(
    input: RejectAnswerInput,
    database: State<'_, Database>,
) -> Result<AnswerHistoryItem, AppError> {
    AnswerEngineService::reject_answer(&database, input)
}

#[tauri::command]
pub fn survey_answer_regenerate(
    input: RegenerateAnswerInput,
    database: State<'_, Database>,
    store: State<'_, CredentialStoreState>,
    transport: State<'_, HttpTransportState>,
    pool_state: State<'_, ProviderPoolState>,
) -> Result<ValidatedAnswerRecommendation, AppError> {
    AnswerEngineService::regenerate_answer(
        &database,
        store.0.as_ref(),
        transport.0.clone(),
        pool_state.0.as_ref(),
        input,
        None,
    )
}

#[tauri::command]
pub fn survey_answer_history_list(
    input: ListAnswerHistoryInput,
    database: State<'_, Database>,
) -> Result<Vec<AnswerHistoryItem>, AppError> {
    AnswerEngineService::list_history(&database, input)
}

#[tauri::command]
pub fn survey_answer_revisions_list(
    persona_id: String,
    question_id: String,
    database: State<'_, Database>,
) -> Result<Vec<ValidatedAnswerRecommendation>, AppError> {
    AnswerEngineService::list_recommendation_revisions(&database, &persona_id, &question_id)
}

// ==========================================
// SC-053: SURVEY MEMORY & RECOGNITION COMMANDS
// ==========================================

#[tauri::command]
pub fn survey_memory_recognize(
    input: RecognizeSurveyInput,
    database: State<'_, Database>,
) -> Result<SurveyRecognitionEvaluation, AppError> {
    SurveyService::db_recognize_survey(&database, input)
}

#[tauri::command]
pub fn survey_memory_get_detail(
    persona_id: String,
    session_id: String,
    database: State<'_, Database>,
) -> Result<SurveyMemorySessionDetail, AppError> {
    SurveyService::db_get_memory_session_detail(&database, &persona_id, &session_id)
}

// ==========================================
// SC-054: NEXT QUESTION INTELLIGENCE COMMANDS
// ==========================================

#[tauri::command]
pub fn survey_predict_next_questions(
    input: PredictNextQuestionInput,
    database: State<'_, Database>,
) -> Result<NextQuestionIntelligenceResult, AppError> {
    SurveyService::db_predict_next_questions(&database, input)
}

#[tauri::command]
pub fn survey_get_session_predictions(
    persona_id: String,
    session_id: String,
    database: State<'_, Database>,
) -> Result<Vec<SurveyQuestionPredictionRecord>, AppError> {
    SurveyService::db_get_session_predictions(&database, &persona_id, &session_id)
}

#[tauri::command]
pub fn survey_get_session_branch_transitions(
    persona_id: String,
    session_id: String,
    database: State<'_, Database>,
) -> Result<Vec<SurveyBranchTransition>, AppError> {
    SurveyService::db_get_session_branch_transitions(&database, &persona_id, &session_id)
}
