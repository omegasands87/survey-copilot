use serde::{Deserialize, Serialize};

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

/// Entitas Survei dalam Survey Memory (berorientasi pada kepemilikan persona)
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
