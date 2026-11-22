use super::fact::{FactHistoryEntry, FactSource};
use super::survey::SurveyRecord;
use serde::{Deserialize, Serialize};

/// Cakupan ekspor data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportScope {
    FullApplication,
    SinglePersona,
}

impl ExportScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FullApplication => "full_application",
            Self::SinglePersona => "single_persona",
        }
    }
}

/// Ringkasan statistik ekspor sebelum file disimpan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportSummary {
    pub export_scope: String,
    pub target_persona_id: Option<String>,
    pub target_persona_name: Option<String>,
    pub total_personas: i64,
    pub total_facts: i64,
    pub total_sources: i64,
    pub total_history_entries: i64,
    pub total_preferences: i64,
    pub total_sessions: i64,
    pub total_questions: i64,
    pub total_recommendations: i64,
    pub total_answers: i64,
    pub total_fit_results: i64,
    pub total_attempts: i64,
    pub sensitive_facts_count: i64,
    pub highly_sensitive_facts_count: i64,
    pub has_sensitive_data: bool,
    pub estimated_records_count: i64,
    pub warning_message: Option<String>,
}

/// Entitas persona dalam berkas ekspor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportPersona {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

/// Entitas fakta dalam berkas ekspor (termasuk nilai ternormalisasi dan klasifikasi privasi)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportFact {
    pub id: String,
    pub persona_id: String,
    pub category: String,
    pub fact_key: String,
    pub attribute: String,
    pub data_type: String,
    pub value: serde_json::Value,
    pub normalized_value: String,
    pub unit: Option<String>,
    pub verification_status: String,
    pub sensitivity: String,
    pub source_date: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

/// Entitas preferensi persona dalam berkas ekspor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportPreference {
    pub id: String,
    pub persona_id: String,
    pub preference_key: String,
    pub value: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

/// Entitas sesi survei dalam berkas ekspor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportSession {
    pub id: String,
    pub persona_id: String,
    pub title: Option<String>,
    pub platform_label: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

/// Metadata tangkapan layar (tanpa binary mentah dan tanpa membocorkan path lokal)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportCaptureMetadata {
    pub id: String,
    pub session_id: String,
    pub capture_mode: String,
    pub captured_at: String,
    pub application_label: Option<String>,
    pub created_at: String,
}

/// Entitas pertanyaan survei dalam berkas ekspor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportQuestion {
    pub id: String,
    pub session_id: String,
    pub capture_id: Option<String>,
    pub question_text_original: String,
    pub question_text_corrected: Option<String>,
    pub instructions: Option<String>,
    pub options: serde_json::Value,
    pub structure: Option<serde_json::Value>,
    pub extraction_status: String,
    pub question_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

/// Entitas rekomendasi jawaban AI dalam berkas ekspor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportRecommendation {
    pub id: String,
    pub question_id: String,
    pub status: String,
    pub draft_text: Option<String>,
    pub interpretation: String,
    pub rationale: Option<String>,
    pub missing_information: serde_json::Value,
    pub conflicts: serde_json::Value,
    pub assumptions: serde_json::Value,
    pub model_id: Option<String>,
    pub prompt_version: Option<String>,
    pub created_at: String,
}

/// Entitas riwayat jawaban pengguna dalam berkas ekspor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportAnswerHistory {
    pub id: String,
    pub question_id: String,
    pub recommendation_id: Option<String>,
    pub answer_text: String,
    pub action: String,
    pub created_at: String,
}

/// Entitas peluang survei
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportOpportunity {
    pub id: String,
    pub title: Option<String>,
    pub platform_label: Option<String>,
    pub topic: Option<String>,
    pub description: Option<String>,
    pub captured_at: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Kriteria peluang survei
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportOpportunityCriterion {
    pub id: String,
    pub opportunity_id: String,
    pub criterion_text: String,
    pub normalized_attribute: Option<String>,
    pub expected_value: Option<serde_json::Value>,
    pub criterion_order: i64,
}

/// Evaluasi kecocokan survei
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportFitResult {
    pub id: String,
    pub opportunity_id: String,
    pub persona_id: String,
    pub criterion_id: String,
    pub match_state: String,
    pub supporting_fact_ids: serde_json::Value,
    pub explanation: String,
    pub evaluated_at: String,
}

/// Percobaan survei
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportAttempt {
    pub id: String,
    pub opportunity_id: String,
    pub persona_id: String,
    pub session_id: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub current_outcome: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Peristiwa hasil akhir survei
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportOutcomeEvent {
    pub id: String,
    pub attempt_id: String,
    pub outcome_label: String,
    pub evidence_source: String,
    pub evidence_note: Option<String>,
    pub confidence: String,
    pub observed_at: String,
    pub created_at: String,
}

/// Koleksi data lengkap yang diekspor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportDataCollections {
    pub personas: Vec<ExportPersona>,
    pub facts: Vec<ExportFact>,
    pub fact_sources: Vec<FactSource>,
    pub fact_history: Vec<FactHistoryEntry>,
    pub persona_preferences: Vec<ExportPreference>,
    pub survey_sessions: Vec<ExportSession>,
    pub screen_captures: Vec<ExportCaptureMetadata>,
    pub survey_questions: Vec<ExportQuestion>,
    pub answer_recommendations: Vec<ExportRecommendation>,
    pub answer_history: Vec<ExportAnswerHistory>,
    pub survey_opportunities: Vec<ExportOpportunity>,
    pub survey_opportunity_criteria: Vec<ExportOpportunityCriterion>,
    pub survey_fit_results: Vec<ExportFitResult>,
    pub survey_attempts: Vec<ExportAttempt>,
    pub survey_outcome_events: Vec<ExportOutcomeEvent>,
    pub survey_memory: Vec<SurveyRecord>,
}

/// Dokumen JSON Ekspor Lengkap
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportPayload {
    #[serde(rename = "$schema")]
    pub schema_url: String,
    pub schema_version: String,
    pub app_version: String,
    pub export_timestamp: String,
    pub export_scope: String,
    pub target_persona_id: Option<String>,
    pub target_persona_name: Option<String>,
    pub summary: ExportSummary,
    pub data: ExportDataCollections,
}

/// Ringkasan penghapusan menyeluruh data aplikasi (Wipe Report)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WipeReport {
    pub deleted_personas_count: i64,
    pub deleted_facts_count: i64,
    pub deleted_sources_count: i64,
    pub deleted_history_count: i64,
    pub deleted_sessions_count: i64,
    pub deleted_questions_count: i64,
    pub deleted_recommendations_count: i64,
    pub deleted_answers_count: i64,
    pub deleted_preferences_count: i64,
    pub deleted_fit_results_count: i64,
    pub deleted_attempts_count: i64,
    pub deleted_provider_configs_count: i64,
    pub deleted_temp_files_count: i64,
    pub wal_checkpoint_status: String,
    pub vacuum_status: String,
    pub wiped_at: String,
    pub unaffected_external_items: Vec<String>,
}

/// Informasi kebijakan retensi dan siklus hidup privasi data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicyInfo {
    pub active_facts_policy: String,
    pub archived_facts_policy: String,
    pub screen_capture_policy: String,
    pub ocr_data_policy: String,
    pub survey_history_policy: String,
    pub export_file_policy: String,
    pub logs_and_diagnostics_policy: String,
    pub deletion_scope_disclaimer: String,
}
