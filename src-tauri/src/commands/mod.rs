pub mod app;
pub mod capture;
pub mod credential;
pub mod data_management;
pub mod fact;
pub mod ocr;
pub mod opportunity;
pub mod persona;
pub mod privacy;
pub mod provider;
pub mod provider_pool;
pub mod survey;
pub mod survey_attempt;
pub mod survey_fit;

pub use app::app_status;
pub use capture::{
    capture_cleanup, capture_discard, capture_get_preview, capture_list_windows, capture_region,
    capture_window,
};
pub use credential::{
    credential_delete, credential_get_status, credential_list, credential_save, credential_test,
    credential_update,
};
pub use data_management::{
    data_get_retention_info, data_get_wipe_summary, data_wipe_all, export_generate_json,
    export_get_summary,
};
pub use fact::{
    fact_add_source, fact_archive, fact_create, fact_delete, fact_detect_conflicts,
    fact_get_detail, fact_get_eligible_for_ai, fact_list, fact_restore, fact_set_sensitivity,
    fact_set_status, fact_update,
};
pub use ocr::{
    ocr_confirm_extraction, ocr_extract_capture, ocr_get_extraction, ocr_get_latest_for_capture,
};
pub use opportunity::{
    opportunity_delete, opportunity_extract_from_capture, opportunity_extract_from_text,
    opportunity_get, opportunity_list, opportunity_save, opportunity_update,
    opportunity_update_criteria,
};
pub use persona::{
    persona_archive, persona_create, persona_delete, persona_get, persona_get_dependency_summary,
    persona_list, persona_restore, persona_set_active, persona_update,
};
pub use privacy::{
    privacy_cleanup_temp_files, privacy_clear_diagnostic_logs, privacy_get_diagnostic_logs,
    privacy_get_retention_status, privacy_get_settings, privacy_get_system_diagnostics,
    privacy_inspect_outbound_data, privacy_record_diagnostic_log, privacy_update_settings,
};
pub use provider::{
    provider_config_create, provider_config_delete, provider_config_get, provider_config_list,
    provider_config_test_connection, provider_config_update, provider_get_recommended_models,
};
pub use provider_pool::{
    provider_pool_execute_request, provider_pool_get_settings, provider_pool_list_health,
    provider_pool_update_settings,
};
pub use survey::{
    survey_answer_confirm, survey_answer_copy, survey_answer_edit, survey_answer_generate,
    survey_answer_get_latest, survey_answer_history_list, survey_answer_regenerate,
    survey_answer_reject, survey_answer_revisions_list, survey_evidence_evaluate,
    survey_question_classify, survey_question_confirm_answer, survey_question_create,
    survey_question_delete, survey_question_get, survey_question_list, survey_question_save_draft,
    survey_question_update, survey_session_archive, survey_session_create, survey_session_delete,
    survey_session_get, survey_session_get_detail, survey_session_list, survey_session_restore,
    survey_session_update, survey_memory_recognize, survey_memory_get_detail,
    survey_predict_next_questions, survey_get_session_predictions, survey_get_session_branch_transitions,
};
pub use survey_attempt::{
    survey_attempt_delete, survey_attempt_get, survey_attempt_get_learning_summary,
    survey_attempt_list, survey_attempt_record, survey_attempt_record_outcome,
};
pub use survey_fit::{
    survey_fit_compare_personas, survey_fit_evaluate, survey_fit_get_results,
};
