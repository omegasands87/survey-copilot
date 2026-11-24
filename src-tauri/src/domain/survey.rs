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
    VersionChanged,
    LikelyMatch,
    PossibleMatch,
    NoMatch,
}

impl SurveyRecognitionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExactMatch => "exact_match",
            Self::VersionChanged => "version_changed",
            Self::LikelyMatch => "likely_match",
            Self::PossibleMatch => "possible_match",
            Self::NoMatch => "no_match",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "exact_match" => Self::ExactMatch,
            "version_changed" => Self::VersionChanged,
            "likely_match" => Self::LikelyMatch,
            "possible_match" => Self::PossibleMatch,
            _ => Self::NoMatch,
        }
    }
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
    pub source_domain: Option<String>,
    pub external_survey_id: Option<String>,
    pub status: SessionStatus,
    pub language: String,
    pub category: Option<String>,
    pub fingerprint: Option<String>,
    pub version: i64,
    pub recognized_from_id: Option<String>,
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
    pub source_domain: Option<String>,
    pub status: SessionStatus,
    pub language: String,
    pub category: Option<String>,
    pub fingerprint: Option<String>,
    pub version: i64,
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
    pub question_fingerprint: Option<String>,
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
    pub capture_id: Option<String>,
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

/// Tipe transisi percabangan pertanyaan yang diamati
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransitionType {
    Sequential,
    ConditionalBranch,
    SkipJump,
}

impl TransitionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sequential => "sequential",
            Self::ConditionalBranch => "conditional_branch",
            Self::SkipJump => "skip_jump",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "conditional_branch" => Self::ConditionalBranch,
            "skip_jump" => Self::SkipJump,
            _ => Self::Sequential,
        }
    }
}

/// Status prediksi pertanyaan dalam siklus NQI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PredictionStatus {
    PredictedNotDetected,
    DetectedMatched,
    DetectedDiverged,
    InsufficientEvidence,
}

impl PredictionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PredictedNotDetected => "predicted_not_detected",
            Self::DetectedMatched => "detected_matched",
            Self::DetectedDiverged => "detected_diverged",
            Self::InsufficientEvidence => "insufficient_evidence",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "detected_matched" => Self::DetectedMatched,
            "detected_diverged" => Self::DetectedDiverged,
            "insufficient_evidence" => Self::InsufficientEvidence,
            _ => Self::PredictedNotDetected,
        }
    }
}

/// Model Entitas Transisi Percabangan Pertanyaan Historis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SurveyBranchTransition {
    pub id: String,
    pub session_id: String,
    pub persona_id: String,
    pub from_question_id: String,
    pub from_question_fingerprint: Option<String>,
    pub from_question_order: i64,
    pub from_answer_value: Option<String>,
    pub to_question_id: Option<String>,
    pub to_question_fingerprint: Option<String>,
    pub to_question_text: String,
    pub to_question_type: QuestionType,
    pub to_options: Vec<String>,
    pub to_question_order: i64,
    pub branch_condition: Option<String>,
    pub transition_type: TransitionType,
    pub observed_at: String,
    pub created_at: String,
}

/// Model Entitas Rekaman Audit Prediksi Pertanyaan NQI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SurveyQuestionPredictionRecord {
    pub id: String,
    pub session_id: String,
    pub persona_id: String,
    pub current_question_id: String,
    pub predicted_question_text: String,
    pub predicted_question_type: QuestionType,
    pub predicted_options: Vec<String>,
    pub confidence_score: f64,
    pub evidence_source: String,
    pub explanation: String,
    pub leading_condition: Option<String>,
    pub status: PredictionStatus,
    pub matched_question_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Prediksi pertanyaan berikutnya (Next Question Intelligence)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NextQuestionPrediction {
    pub id: String,
    pub candidate_question_text: String,
    pub question_type: QuestionType,
    pub options: Vec<String>,
    pub evidence_source: String,
    pub confidence: f64,
    pub explanation: String,
    pub leading_condition: Option<String>,
    /// Penanda tegas: diprediksi, belum terdeteksi secara aktual (AT-065)
    pub is_predicted_not_detected: bool,
}

/// Hasil Evaluasi Next Question Intelligence (NQI) untuk Antarmuka Pengguna
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NextQuestionIntelligenceResult {
    pub session_id: String,
    pub persona_id: String,
    pub current_question_id: String,
    pub current_question_text: String,
    pub current_user_answer: Option<String>,
    pub status: PredictionStatus,
    pub predictions: Vec<NextQuestionPrediction>,
    pub observed_transitions_count: usize,
    pub explanation: String,
    pub evaluated_at: String,
}

/// DTO Input Prediksi Pertanyaan Berikutnya
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PredictNextQuestionInput {
    pub persona_id: String,
    pub session_id: String,
    pub current_question_id: String,
    pub current_answer: Option<String>,
}

// ==========================================
// SC-053: SURVEY MEMORY & RECOGNITION DOMAIN
// ==========================================

/// Rekaman riwayat hasil evaluasi pengenalan survei
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SurveyRecognitionRecord {
    pub id: String,
    pub session_id: String,
    pub persona_id: String,
    pub matched_session_id: Option<String>,
    pub match_status: SurveyRecognitionStatus,
    pub confidence_score: f64,
    pub structural_similarity: f64,
    pub title_similarity: f64,
    pub matched_question_count: i64,
    pub total_question_count: i64,
    pub differences: Vec<String>,
    pub provenance: String,
    pub recognized_at: String,
    pub created_at: String,
}

/// Evaluasi Pengenalan Survei Lengkap untuk UI dan Memory
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SurveyRecognitionEvaluation {
    pub current_session_id: String,
    pub current_fingerprint: Option<String>,
    pub match_status: SurveyRecognitionStatus,
    pub confidence_score: f64,
    pub structural_similarity: f64,
    pub title_similarity: f64,
    pub matched_session_id: Option<String>,
    pub matched_session_title: Option<String>,
    pub matched_platform: Option<String>,
    pub matched_version: Option<i64>,
    pub matched_question_count: i64,
    pub total_question_count: i64,
    pub differences: Vec<String>,
    pub provenance: String,
    pub explanation: String,
    /// Jawaban historis nyata yang dikonfirmasi responden untuk pertanyaan yang cocok (persona-isolated)
    pub recognized_answers: Vec<RecognizedQuestionAnswer>,
    pub recognized_at: String,
}

/// Detail jawaban historis nyata untuk pertanyaan yang teridentifikasi berulang
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RecognizedQuestionAnswer {
    pub question_order: i64,
    pub question_text: String,
    pub matched_historical_question_id: String,
    pub confirmed_answer: String,
    pub confirmed_at: String,
    pub provenance: Provenance,
}

/// DTO Input Permintaan Pengenalan Survei
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecognizeSurveyInput {
    pub persona_id: String,
    pub session_id: String,
}

/// Tampilan pertanyaan Survey Memory yang mempertegas pemisahan jawaban nyata vs rekomendasi AI (AT-064)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SurveyMemoryQuestionItem {
    pub id: String,
    pub session_id: String,
    pub persona_id: String,
    pub question_order: i64,
    pub prompt_text: String,
    pub instructions: Option<String>,
    pub options: Vec<String>,
    pub question_type: QuestionType,
    pub answer_format: AnswerFormat,
    pub provenance: Provenance,
    /// JAWABAN AKTUAL TERKONFIRMASI PENGGUNA (Terpisah tegas dari AI recommendation)
    pub user_answer_confirmed: Option<String>,
    pub confirmed_at: Option<String>,
    /// DRAF SEMENTARA PENGGUNA
    pub user_answer_draft: Option<String>,
    /// REKOMENDASI AI TERBARU (Jika ada - berlabel draf saran AI)
    pub ai_recommendation_draft: Option<String>,
    pub ai_confidence: Option<String>,
    pub ai_rationale: Option<String>,
    pub question_fingerprint: Option<String>,
}

/// Paket detail Survey Memory untuk sesi survei
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SurveyMemorySessionDetail {
    pub session: SurveySession,
    pub questions: Vec<SurveyMemoryQuestionItem>,
    pub latest_recognition: Option<SurveyRecognitionRecord>,
}

// ==========================================
// SC-053: HELPER FUNCTIONS
// ==========================================

/// Mengekstraksi nama domain dari URL (misal: "https://survey.toluna.com/v2" -> "survey.toluna.com")
pub fn extract_domain_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }
    let without_proto = if let Some(stripped) = trimmed.strip_prefix("https://") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("http://") {
        stripped
    } else {
        trimmed
    };

    let domain_part = without_proto
        .split('/')
        .next()?
        .split(':')
        .next()?
        .split('?')
        .next()?;
    let domain = domain_part.trim().to_lowercase();
    if domain.is_empty() {
        None
    } else {
        Some(domain)
    }
}

/// Menghitung sidik jari struktural untuk satu pertanyaan
pub fn compute_question_fingerprint(
    prompt: &str,
    q_type: QuestionType,
    options: &[String],
) -> String {
    use sha2::{Digest, Sha256};
    let norm_prompt: String = prompt
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    let norm_prompt = norm_prompt.split_whitespace().collect::<Vec<_>>().join(" ");

    let mut norm_options: Vec<String> = options
        .iter()
        .map(|opt| {
            let s: String = opt
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect();
            s.split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .filter(|s| !s.is_empty())
        .collect();
    norm_options.sort();

    let combined = format!(
        "{}:{}:{}",
        norm_prompt,
        q_type.as_str(),
        norm_options.join("|")
    );
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}

/// Menghitung sidik jari struktural urutan pertanyaan seluruh sesi survei
pub fn compute_session_structural_fingerprint(
    questions: &[(i64, &str, QuestionType, &[String])],
) -> String {
    use sha2::{Digest, Sha256};
    if questions.is_empty() {
        return "empty".to_string();
    }

    let mut ordered = questions.to_vec();
    ordered.sort_by_key(|q| q.0);

    let mut q_hashes = Vec::new();
    for (order, prompt, qtype, options) in ordered {
        let q_fp = compute_question_fingerprint(prompt, qtype, options);
        q_hashes.push(format!("{}:{}", order, q_fp));
    }

    let combined = q_hashes.join(";");
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();
    let hex = format!("{:x}", result);
    let short_hex = if hex.len() >= 16 { &hex[..16] } else { &hex };
    format!("v1:len={}:{}", questions.len(), short_hex)
}
