use serde::{Deserialize, Serialize};

use crate::domain::evidence::{EvidenceConfidence, RetrievedEvidenceItem, SensitivityConsent};

/// Status dukungan bukti untuk rekomendasi jawaban AI (sesuai ANSWER_ENGINE.md & STRUCTURED_OUTPUT_SCHEMA.md)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    /// Bukti terkonfirmasi saat ini cukup dan valid untuk menjawab pertanyaan
    Supported,
    /// Bukti hanya menjawab sebagian pertanyaan; menyertakan gap atau klarifikasi
    PartiallySupported,
    /// Bukti yang dibutuhkan tidak ditemukan di profil responden; dilarang mengarang jawaban
    InsufficientInformation,
    /// Ditemukan bukti material yang saling bertentangan; dilarang memberi jawaban definitif
    ConflictingEvidence,
    /// Diperlukan konfirmasi atau input spesifik pengguna (misal preferensi subjektif/opini)
    UserInputRequired,
    /// Pertanyaan tidak relevan / tidak berlaku berdasarkan bukti terkonfirmasi eksplisit pengguna
    NotApplicable,
}

impl EvidenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::PartiallySupported => "partially_supported",
            Self::InsufficientInformation => "insufficient_information",
            Self::ConflictingEvidence => "conflicting_evidence",
            Self::UserInputRequired => "user_input_required",
            Self::NotApplicable => "not_applicable",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "supported" => Self::Supported,
            "partially_supported" => Self::PartiallySupported,
            "conflicting_evidence" | "conflict" => Self::ConflictingEvidence,
            "user_input_required" | "input_required" => Self::UserInputRequired,
            "not_applicable" | "na" => Self::NotApplicable,
            _ => Self::InsufficientInformation,
        }
    }

    /// Menentukan apakah status ini mengizinkan penayangan draf jawaban faktual
    pub fn permits_factual_draft(&self) -> bool {
        matches!(self, Self::Supported | Self::PartiallySupported)
    }
}

/// Aksi peninjauan (review) pada rekomendasi AI (SC-034)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewAction {
    Edited,
    Confirmed,
    Copied,
    Rejected,
    Regenerated,
}

impl ReviewAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Edited => "edited",
            Self::Confirmed => "confirmed",
            Self::Copied => "copied",
            Self::Rejected => "rejected",
            Self::Regenerated => "regenerated",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "edited" | "edit" => Self::Edited,
            "confirmed" | "confirm" => Self::Confirmed,
            "copied" | "copy" => Self::Copied,
            "rejected" | "reject" => Self::Rejected,
            "regenerated" | "regenerate" => Self::Regenerated,
            _ => Self::Edited,
        }
    }
}

/// Status peninjauan rekomendasi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Draft,
    Confirmed,
    Rejected,
    Edited,
}

impl ReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Confirmed => "confirmed",
            Self::Rejected => "rejected",
            Self::Edited => "edited",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "confirmed" => Self::Confirmed,
            "rejected" => Self::Rejected,
            "edited" => Self::Edited,
            _ => Self::Draft,
        }
    }
}

/// DTO Output Mentah dari AI Provider (sesuai schema terstruktur)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct RawAnswerRecommendation {
    pub status: String,
    pub question_intent: Option<String>,
    pub answer_format: Option<String>,
    pub interpretation: Option<String>,
    pub draft_answer: Option<String>,
    pub supporting_fact_ids: Option<Vec<String>>,
    pub rationale: Option<String>,
    pub missing_information: Option<Vec<String>>,
    pub conflicts: Option<Vec<String>>,
    pub assumptions: Option<Vec<String>>,
    pub clarifying_question: Option<String>,
}

/// Rekomendasi Jawaban AI yang telah Tervalidasi secara Lokal (Local Policy Enforced)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatedAnswerRecommendation {
    pub id: String,
    pub question_id: Option<String>,
    pub session_id: Option<String>,
    pub persona_id: String,
    pub status: EvidenceStatus,
    pub question_intent: String,
    pub answer_format: String,
    pub interpretation: String,
    /// Draft rekomendasi asli dari AI (permanen, tidak diubah oleh edit pengguna)
    pub draft_answer: Option<String>,
    /// Jawaban aktual yang diedit atau ditetapkan pengguna untuk rekomendasi ini
    pub user_answer: Option<String>,
    /// Status review rekomendasi ini: 'draft', 'confirmed', 'rejected', 'edited'
    pub review_status: Option<String>,
    /// Alasan penolakan atau catatan review jika diberikan
    pub review_reason: Option<String>,
    /// Waktu peninjauan dilakukan
    pub reviewed_at: Option<String>,
    /// Fakta-fakta profil yang terverifikasi dan lolos allowlist permintaan
    pub supporting_facts: Vec<RetrievedEvidenceItem>,
    /// Sitasi ID fakta yang ditolak karena halusinasi atau bukan milik persona (AT-043)
    pub rejected_citations: Vec<String>,
    pub rationale: Option<String>,
    pub missing_information: Vec<String>,
    pub conflicts: Vec<String>,
    pub assumptions: Vec<String>,
    pub clarifying_question: Option<String>,
    /// Tingkat keyakinan bukti lokal (tidak mempercayai keyakinan model secara buta)
    pub confidence: EvidenceConfidence,
    pub prompt_version: String,
    pub provider_config_id: Option<String>,
    pub model_id: Option<String>,
    pub created_at: String,
}

/// DTO Input IPC untuk Permintaan Perumusan Rekomendasi Jawaban Terstruktur
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateAnswerInput {
    pub persona_id: String,
    pub session_id: Option<String>,
    pub question_id: Option<String>,
    pub question_text: String,
    pub instructions: Option<String>,
    pub options: Option<Vec<String>>,
    pub consent: Option<SensitivityConsent>,
    pub provider_config_id: Option<String>,
    pub prompt_version: Option<String>,
}

/// Rekaman database untuk entitas `answer_recommendations`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredRecommendation {
    pub id: String,
    pub question_id: String,
    pub persona_id: Option<String>,
    pub status: String,
    pub draft_text: Option<String>,
    pub user_answer: Option<String>,
    pub review_status: String,
    pub review_reason: Option<String>,
    pub reviewed_at: Option<String>,
    pub interpretation: String,
    pub rationale: Option<String>,
    pub missing_information_json: String,
    pub conflicts_json: String,
    pub assumptions_json: String,
    pub clarifying_question: Option<String>,
    pub confidence: String,
    pub rejected_citations_json: String,
    pub provider_config_id: Option<String>,
    pub model_id: Option<String>,
    pub prompt_version: Option<String>,
    pub created_at: String,
}

/// Rekaman sumber rekomendasi `recommendation_sources`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredRecommendationSource {
    pub recommendation_id: String,
    pub fact_id: String,
    pub persona_id: String,
    pub support_note: Option<String>,
}

/// Item rekaman riwayat perubahan jawaban survei (SC-034)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerHistoryItem {
    pub id: String,
    pub persona_id: String,
    pub question_id: String,
    pub recommendation_id: Option<String>,
    pub action: ReviewAction,
    /// Draft rekomendasi asli dari AI
    pub draft_answer: Option<String>,
    /// Jawaban aktual yang diedit atau dikonfirmasi pengguna
    pub actual_answer: String,
    pub review_status: String,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub prompt_version: Option<String>,
    pub evidence_sources_json: String,
    pub reason: Option<String>,
    pub created_at: String,
}

/// DTO Input untuk Aksi Edit Jawaban (SC-034)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditAnswerInput {
    pub persona_id: String,
    pub question_id: String,
    pub recommendation_id: Option<String>,
    pub actual_answer: String,
    pub reason: Option<String>,
}

/// DTO Input untuk Aksi Konfirmasi Rekomendasi (SC-034)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmRecommendationInput {
    pub persona_id: String,
    pub question_id: String,
    pub recommendation_id: Option<String>,
    pub actual_answer: String,
}

/// DTO Input untuk Aksi Salin Jawaban (SC-034)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyAnswerInput {
    pub persona_id: String,
    pub question_id: String,
    pub recommendation_id: Option<String>,
    pub copied_text: String,
}

/// DTO Input untuk Aksi Tolak Rekomendasi (SC-034)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectAnswerInput {
    pub persona_id: String,
    pub question_id: String,
    pub recommendation_id: Option<String>,
    pub reason: Option<String>,
}

/// DTO Input untuk Aksi Regenerasi Jawaban (SC-034)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegenerateAnswerInput {
    pub persona_id: String,
    pub session_id: Option<String>,
    pub question_id: String,
    pub question_text: String,
    pub instructions: Option<String>,
    pub options: Option<Vec<String>>,
    pub consent: Option<SensitivityConsent>,
    pub provider_config_id: Option<String>,
    pub prompt_version: Option<String>,
    pub reason: Option<String>,
}

/// DTO Input untuk Query Riwayat Perubahan Jawaban (SC-034)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAnswerHistoryInput {
    pub persona_id: String,
    pub question_id: Option<String>,
    pub action: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}
