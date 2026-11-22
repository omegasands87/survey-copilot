use serde::{Deserialize, Serialize};

/// Status sesi survei
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Archived,
    Completed,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
            Self::Completed => "completed",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "archived" => Self::Archived,
            "completed" => Self::Completed,
            _ => Self::Active,
        }
    }
}

/// Status pengenalan survei historis terhadap survei yang sedang berjalan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurveyRecognitionStatus {
    ExactMatch,
    LikelyMatch,
    PossibleMatch,
    NoMatch,
}

/// Asal-usul data teks / pertanyaan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    UserEntered,
    ManuallyConfirmed,
    OcrDetected,
    AiInferred,
}

impl Provenance {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserEntered => "manual_entry",
            Self::ManuallyConfirmed => "manually_confirmed",
            Self::OcrDetected => "ocr_detected",
            Self::AiInferred => "ai_inferred",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "manually_confirmed" => Self::ManuallyConfirmed,
            "ocr_detected" => Self::OcrDetected,
            "ai_inferred" => Self::AiInferred,
            _ => Self::UserEntered,
        }
    }
}

/// Tipe pertanyaan survei
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionType {
    SingleChoice,
    MultipleChoice,
    FreeText,
    Scale,
    Matrix,
    Numeric,
    Date,
    Other,
    Unknown,
}

impl QuestionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SingleChoice => "single_choice",
            Self::MultipleChoice => "multiple_choice",
            Self::FreeText => "free_text",
            Self::Scale => "scale",
            Self::Matrix => "matrix",
            Self::Numeric => "numeric",
            Self::Date => "date",
            Self::Other => "other",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "single_choice" => Self::SingleChoice,
            "multiple_choice" => Self::MultipleChoice,
            "free_text" | "open_text" | "text" => Self::FreeText,
            "scale" | "rating" => Self::Scale,
            "matrix" | "grid" => Self::Matrix,
            "numeric" | "number" => Self::Numeric,
            "date" => Self::Date,
            "other" => Self::Other,
            _ => Self::Unknown,
        }
    }
}

/// Format jawaban survei
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerFormat {
    SingleChoice,
    MultipleSelection,
    Text,
    Number,
    RatingScale,
    Matrix,
    Boolean,
    Unknown,
}

impl AnswerFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SingleChoice => "single_choice",
            Self::MultipleSelection => "multiple_selection",
            Self::Text => "text",
            Self::Number => "number",
            Self::RatingScale => "rating_scale",
            Self::Matrix => "matrix",
            Self::Boolean => "boolean",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "single_choice" => Self::SingleChoice,
            "multiple_selection" => Self::MultipleSelection,
            "text" => Self::Text,
            "number" => Self::Number,
            "rating_scale" => Self::RatingScale,
            "matrix" => Self::Matrix,
            "boolean" => Self::Boolean,
            _ => Self::Unknown,
        }
    }
}

/// Status pertanyaan dalam siklus NQI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionStatus {
    /// Pertanyaan teramati pada sesi aktif dan dikonfirmasi pengguna
    Detected,
    /// Prediksi pertanyaan berikutnya berdasarkan urutan/percabangan historis
    Predicted,
    /// Rekaman pertanyaan dari sesi survei masa lalu
    Historical,
    /// Bukti belum cukup
    Unknown,
}

impl QuestionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Detected => "detected",
            Self::Predicted => "predicted",
            Self::Historical => "historical",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "predicted" => Self::Predicted,
            "historical" => Self::Historical,
            "unknown" => Self::Unknown,
            _ => Self::Detected,
        }
    }
}

/// Status konfirmasi pertanyaan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationStatus {
    Draft,
    Confirmed,
    Skipped,
    Rejected,
}

impl ConfirmationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Confirmed => "confirmed",
            Self::Skipped => "skipped",
            Self::Rejected => "rejected",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "confirmed" => Self::Confirmed,
            "skipped" => Self::Skipped,
            "rejected" => Self::Rejected,
            _ => Self::Draft,
        }
    }
}

/// Status jawaban pengguna
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerState {
    Draft,
    UserEdited,
    UserConfirmed,
    Skipped,
    NotAnswered,
    Unknown,
}

/// Model Entitas Sesi Survei ber-scope persona aktif
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SurveySession {
    pub id: String,
    pub persona_id: String,
    pub title: String,
    pub platform_label: Option<String>,
    pub source_url: Option<String>,
    pub external_survey_id: Option<String>,
    pub status: SessionStatus,
    pub language: String,
    pub category: Option<String>,
    pub fingerprint: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

/// Ringkasan Sesi Survei untuk daftar dan statistik
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SurveySessionSummary {
    pub id: String,
    pub persona_id: String,
    pub title: String,
    pub platform_label: Option<String>,
    pub source_url: Option<String>,
    pub status: SessionStatus,
    pub language: String,
    pub category: Option<String>,
    pub question_count: i64,
    pub confirmed_answer_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

/// Entitas Pertanyaan Survei ber-scope persona dan sesi
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SurveyQuestion {
    pub id: String,
    pub session_id: String,
    pub persona_id: String,
    pub capture_id: Option<String>,
    pub question_order: i64,
    pub question_text_original: String,
    pub question_text_corrected: Option<String>,
    pub instructions: Option<String>,
    pub options: Vec<String>,
    pub structure_json: Option<serde_json::Value>,
    pub extraction_status: String,
    pub question_type: QuestionType,
    pub answer_format: AnswerFormat,
    pub provenance: Provenance,
    pub observation_type: QuestionStatus,
    pub confirmation_status: ConfirmationStatus,
    pub user_answer_draft: Option<String>,
    pub user_answer_confirmed: Option<String>,
    pub confirmed_at: Option<String>,
    pub preceding_question_id: Option<String>,
    pub branch_logic: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Detail Sesi Survei beserta seluruh pertanyaan terurut
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SurveySessionDetail {
    pub session: SurveySession,
    pub questions: Vec<SurveyQuestion>,
}

/// DTO Input Pembuatan Sesi Survei Baru
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionInput {
    pub persona_id: String,
    pub title: String,
    pub platform_label: Option<String>,
    pub source_url: Option<String>,
    pub external_survey_id: Option<String>,
    pub language: Option<String>,
    pub category: Option<String>,
    pub notes: Option<String>,
}

/// DTO Input Pembaruan Sesi Survei
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSessionInput {
    pub persona_id: String,
    pub session_id: String,
    pub title: String,
    pub platform_label: Option<String>,
    pub source_url: Option<String>,
    pub external_survey_id: Option<String>,
    pub language: Option<String>,
    pub category: Option<String>,
    pub notes: Option<String>,
    pub status: Option<SessionStatus>,
}

/// DTO Input Pembuatan Pertanyaan Survei Manual
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateQuestionInput {
    pub persona_id: String,
    pub session_id: String,
    pub question_text_original: String,
    pub instructions: Option<String>,
    pub options: Option<Vec<String>>,
    pub question_type: Option<QuestionType>,
    pub answer_format: Option<AnswerFormat>,
    pub provenance: Option<Provenance>,
    pub question_order: Option<i64>,
    pub preceding_question_id: Option<String>,
    pub branch_logic: Option<String>,
}

/// DTO Input Pembaruan Pertanyaan Survei
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateQuestionInput {
    pub persona_id: String,
    pub question_id: String,
    pub question_text_original: Option<String>,
    pub question_text_corrected: Option<String>,
    pub instructions: Option<String>,
    pub options: Option<Vec<String>>,
    pub question_type: Option<QuestionType>,
    pub answer_format: Option<AnswerFormat>,
    pub confirmation_status: Option<ConfirmationStatus>,
    pub user_answer_draft: Option<String>,
    pub preceding_question_id: Option<String>,
    pub branch_logic: Option<String>,
}

/// DTO Input Penyimpanan Draf Jawaban Pengguna (Auto-save)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveQuestionDraftInput {
    pub persona_id: String,
    pub question_id: String,
    pub user_answer_draft: String,
}

/// DTO Input Konfirmasi Jawaban Aktual Pengguna
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmAnswerInput {
    pub persona_id: String,
    pub question_id: String,
    pub user_answer: String,
}

/// Entitas Survei dalam Survey Memory (berorientasi pada kepemilikan persona untuk ekspor)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SurveyRecord {
    pub id: String,
    pub persona_id: String,
    pub title: String,
    pub source_platform: Option<String>,
    pub source_url: Option<String>,
    pub provider_survey_id: Option<String>,
    pub recognition_fingerprint: Option<String>,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

impl From<&SurveySession> for SurveyRecord {
    fn from(s: &SurveySession) -> Self {
        Self {
            id: s.id.clone(),
            persona_id: s.persona_id.clone(),
            title: s.title.clone(),
            source_platform: s.platform_label.clone(),
            source_url: s.source_url.clone(),
            provider_survey_id: s.external_survey_id.clone(),
            recognition_fingerprint: s.fingerprint.clone(),
            first_seen_at: s.created_at.clone(),
            last_seen_at: s.updated_at.clone(),
        }
    }
}

/// Entitas Pertanyaan Survei yang tersimpan secara terurut
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurveyQuestionRecord {
    pub id: String,
    pub survey_id: String,
    pub persona_id: String,
    pub sequence_order: i64,
    pub prompt_text: String,
    pub normalized_text: Option<String>,
    pub instructions: Option<String>,
    pub options_json: Option<serde_json::Value>,
    pub question_type: QuestionType,
    pub question_status: QuestionStatus,
    pub provenance: Provenance,
    pub observed_at: String,
}

/// Jawaban aktual pengguna yang tersimpan terpisah dari draft saran AI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActualUserAnswer {
    pub id: String,
    pub question_id: String,
    pub persona_id: String,
    pub answer_text: String,
    pub answer_state: AnswerState,
    pub confirmed_at: String,
}

/// Rekomendasi jawaban AI (draft)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiAnswerDraft {
    pub id: String,
    pub question_id: String,
    pub recommended_answer: String,
    pub confidence_score: f64,
    pub rationale: String,
    pub cited_fact_ids: Vec<String>,
    pub generated_at: String,
}

/// Prediksi pertanyaan berikutnya (Next Question Intelligence)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextQuestionPrediction {
    pub candidate_question_text: String,
    pub question_type: QuestionType,
    pub evidence_source: String,
    pub confidence: f64,
    pub explanation: String,
    pub leading_condition: Option<String>,
    /// Penanda tegas: diprediksi, belum terdeteksi secara aktual
    pub is_predicted_not_detected: bool,
}
