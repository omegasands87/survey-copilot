pub mod commands;
pub mod db;
pub mod domain;
pub mod infrastructure;
pub mod migrations;
pub mod repository;
pub mod services;

#[derive(Clone)]
pub struct CredentialStoreState(
    pub std::sync::Arc<dyn infrastructure::credentials::CredentialStore>,
);

#[derive(Clone)]
pub struct HttpTransportState(pub std::sync::Arc<dyn infrastructure::http::HttpTransport>);

#[derive(Clone)]
pub struct ProviderPoolState(pub std::sync::Arc<services::ProviderPoolService>);

#[derive(Clone)]
pub struct CaptureEngineState(pub std::sync::Arc<dyn infrastructure::capture::ScreenCaptureEngine>);

#[derive(Clone)]
pub struct OcrEngineState(pub std::sync::Arc<dyn infrastructure::ocr::OcrEngine>);

use commands::{
    app_status, capture_cleanup, capture_discard, capture_get_preview, capture_list_windows,
    capture_region, capture_window, credential_delete, credential_get_status, credential_list,
    credential_save, credential_test, credential_update, data_get_retention_info,
    data_get_wipe_summary, data_wipe_all, export_generate_json, export_get_summary,
    fact_add_source, fact_archive, fact_create, fact_delete, fact_detect_conflicts,
    fact_get_detail, fact_get_eligible_for_ai, fact_list, fact_restore, fact_set_sensitivity,
    fact_set_status, fact_update, ocr_confirm_extraction, ocr_extract_capture, ocr_get_extraction,
    ocr_get_latest_for_capture, opportunity_delete, opportunity_extract_from_capture,
    opportunity_extract_from_text, opportunity_get, opportunity_list, opportunity_save,
    opportunity_update, opportunity_update_criteria, persona_archive, persona_create,
    persona_delete, persona_get, persona_get_dependency_summary, persona_list, persona_restore,
    persona_set_active, persona_update, provider_config_create, provider_config_delete,
    provider_config_get, provider_config_list, provider_config_test_connection,
    provider_config_update, provider_get_recommended_models, provider_pool_execute_request,
    provider_pool_get_settings, provider_pool_list_health, provider_pool_update_settings,
    survey_answer_confirm, survey_answer_copy, survey_answer_edit, survey_answer_generate,
    survey_answer_get_latest, survey_answer_history_list, survey_answer_regenerate,
    survey_answer_reject, survey_answer_revisions_list, survey_evidence_evaluate,
    survey_question_classify, survey_question_confirm_answer, survey_question_create,
    survey_question_delete, survey_question_get, survey_question_list, survey_question_save_draft,
    survey_question_update, survey_session_archive, survey_session_create, survey_session_delete,
    survey_session_get, survey_session_get_detail, survey_session_list, survey_session_restore,
    survey_session_update, survey_memory_recognize, survey_memory_get_detail,
    survey_predict_next_questions, survey_get_session_predictions, survey_get_session_branch_transitions,
    survey_fit_compare_personas, survey_fit_evaluate, survey_fit_get_results,
    survey_attempt_delete, survey_attempt_get, survey_attempt_get_learning_summary,
    survey_attempt_list, survey_attempt_record, survey_attempt_record_outcome,
    privacy_cleanup_temp_files, privacy_clear_diagnostic_logs, privacy_get_diagnostic_logs,
    privacy_get_retention_status, privacy_get_settings, privacy_get_system_diagnostics,
    privacy_inspect_outbound_data, privacy_record_diagnostic_log, privacy_update_settings,
};
use db::Database;
use std::fs;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&app_data_dir)?;
            let database_path = app_data_dir.join("survey-copilot.sqlite3");
            let database = Database::initialize(&database_path).map_err(std::io::Error::other)?;
            app.manage(database);

            let credential_store: Arc<dyn infrastructure::credentials::CredentialStore> =
                Arc::new(infrastructure::credentials::WindowsCredentialStore::new());
            app.manage(CredentialStoreState(credential_store));

            let http_transport: Arc<dyn infrastructure::http::HttpTransport> =
                Arc::new(infrastructure::http::DefaultHttpTransport::new());
            app.manage(HttpTransportState(http_transport));

            let pool_service = Arc::new(services::ProviderPoolService::new());
            app.manage(ProviderPoolState(pool_service));

            let capture_engine: Arc<dyn infrastructure::capture::ScreenCaptureEngine> =
                Arc::new(infrastructure::capture::WindowsScreenCaptureEngine::new());
            app.manage(CaptureEngineState(capture_engine));

            let primary_ocr = Arc::new(infrastructure::ocr::WindowsMediaOcrEngine::new());
            let fallback_ocr = Arc::new(infrastructure::ocr::FixtureOcrEngine::new());
            let composite_ocr: Arc<dyn infrastructure::ocr::OcrEngine> = Arc::new(
                infrastructure::ocr::CompositeOcrEngine::new(primary_ocr, Some(fallback_ocr)),
            );
            app.manage(OcrEngineState(composite_ocr));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_status,
            capture_list_windows,
            capture_region,
            capture_window,
            capture_discard,
            capture_cleanup,
            capture_get_preview,
            persona_list,
            persona_get,
            persona_create,
            persona_update,
            persona_set_active,
            persona_archive,
            persona_restore,
            persona_get_dependency_summary,
            persona_delete,
            fact_list,
            fact_get_detail,
            fact_create,
            fact_update,
            fact_set_status,
            fact_set_sensitivity,
            fact_add_source,
            fact_archive,
            fact_restore,
            fact_delete,
            fact_detect_conflicts,
            fact_get_eligible_for_ai,
            export_get_summary,
            export_generate_json,
            data_get_wipe_summary,
            data_wipe_all,
            data_get_retention_info,
            credential_save,
            credential_list,
            credential_get_status,
            credential_update,
            credential_delete,
            credential_test,
            provider_config_create,
            provider_config_get,
            provider_config_list,
            provider_config_update,
            provider_config_delete,
            provider_config_test_connection,
            provider_get_recommended_models,
            provider_pool_get_settings,
            provider_pool_update_settings,
            provider_pool_list_health,
            provider_pool_execute_request,
            survey_session_create,
            survey_session_list,
            survey_session_get,
            survey_session_update,
            survey_session_archive,
            survey_session_restore,
            survey_session_delete,
            survey_session_get_detail,
            survey_memory_recognize,
            survey_memory_get_detail,
            survey_predict_next_questions,
            survey_get_session_predictions,
            survey_get_session_branch_transitions,
            survey_question_create,
            survey_question_get,
            survey_question_list,
            survey_question_update,
            survey_question_save_draft,
            survey_question_confirm_answer,
            survey_question_delete,
            survey_question_classify,
            survey_evidence_evaluate,
            survey_answer_generate,
            survey_answer_get_latest,
            survey_answer_edit,
            survey_answer_confirm,
            survey_answer_copy,
            survey_answer_reject,
            survey_answer_regenerate,
            survey_answer_history_list,
            survey_answer_revisions_list,
            ocr_extract_capture,
            ocr_get_extraction,
            ocr_get_latest_for_capture,
            ocr_confirm_extraction,
            opportunity_extract_from_text,
            opportunity_extract_from_capture,
            opportunity_save,
            opportunity_get,
            opportunity_list,
            opportunity_update,
            opportunity_update_criteria,
            opportunity_delete,
            survey_fit_evaluate,
            survey_fit_get_results,
            survey_fit_compare_personas,
            survey_attempt_record,
            survey_attempt_record_outcome,
            survey_attempt_get,
            survey_attempt_list,
            survey_attempt_delete,
            survey_attempt_get_learning_summary,
            privacy_get_settings,
            privacy_update_settings,
            privacy_inspect_outbound_data,
            privacy_get_retention_status,
            privacy_cleanup_temp_files,
            privacy_get_diagnostic_logs,
            privacy_clear_diagnostic_logs,
            privacy_get_system_diagnostics,
            privacy_record_diagnostic_log
        ])
        .run(tauri::generate_context!())
        .expect("error while running Survey Copilot");
}
