use serde::{Deserialize, Serialize};

use crate::domain::survey::{AnswerFormat, QuestionType};

/// Kategori intent pertanyaan survei (dapat bernilai multi-label)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionCategory {
    /// Demografi: usia, jenis kelamin, komposisi rumah tangga, tempat tinggal, status perkawinan
    Demographic,
    /// Pekerjaan dan pendidikan: pekerjaan, profesi, industri, status kepegawaian, jenjang pendidikan
    EmploymentEducation,
    /// Pendapatan dan kondisi ekonomi: penghasilan, pengeluaran, kelas sosial ekonomi, produk perbankan/keuangan
    FinancialEconomic,
    /// Kebiasaan konsumsi: makanan, minuman, belanja rutin, langganan, konsumsi media
    ConsumptionHabits,
    /// Kepemilikan aset: rumah, kendaraan, perangkat elektronik, properti
    AssetOwnership,
    /// Penggunaan produk dan merek: brand awareness, brand usage, merek yang pernah dibeli/digunakan
    BrandProductUsage,
    /// Pengalaman dan perilaku: aktivitas masa lalu, riwayat perjalanan, kebiasaan berlibur/olahraga
    ExperienceBehavior,
    /// Preferensi dan opini: pandangan pribadi, kesukaan, preferensi fitur/layanan, sikap
    PreferenceOpinion,
    /// Kepuasan dan skala penilaian: tingkat kepuasan, rating NPS (Net Promoter Score), evaluasi kualitas
    SatisfactionRating,
    /// Format pilihan ganda (single atau multiple)
    MultipleChoice,
    /// Format matriks/grid penilaian baris-kolom
    MatrixGrid,
    /// Format angka numerik murni
    Numeric,
    /// Format tanggal, bulan, atau tahun tertentu
    Date,
    /// Format isian teks bebas terbuka
    FreeText,
    /// Kategori lain yang belum terdefinisi atau tidak dapat dipastikan
    OtherUnknown,
}

impl QuestionCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Demographic => "demographic",
            Self::EmploymentEducation => "employment_education",
            Self::FinancialEconomic => "financial_economic",
            Self::ConsumptionHabits => "consumption_habits",
            Self::AssetOwnership => "asset_ownership",
            Self::BrandProductUsage => "brand_product_usage",
            Self::ExperienceBehavior => "experience_behavior",
            Self::PreferenceOpinion => "preference_opinion",
            Self::SatisfactionRating => "satisfaction_rating",
            Self::MultipleChoice => "multiple_choice",
            Self::MatrixGrid => "matrix_grid",
            Self::Numeric => "numeric",
            Self::Date => "date",
            Self::FreeText => "free_text",
            Self::OtherUnknown => "other_unknown",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        let normalized = s.trim().to_lowercase();
        match normalized.as_str() {
            "demographic" | "demografi" => Self::Demographic,
            "employment_education" | "employment" | "education" | "pekerjaan" | "pendidikan" => {
                Self::EmploymentEducation
            }
            "financial_economic" | "financial" | "economic" | "pendapatan" | "ekonomi" => {
                Self::FinancialEconomic
            }
            "consumption_habits" | "consumption" | "kebiasaan_konsumsi" => Self::ConsumptionHabits,
            "asset_ownership" | "assets" | "kepemilikan_aset" => Self::AssetOwnership,
            "brand_product_usage" | "brand_usage" | "brand" | "merek" | "produk" => {
                Self::BrandProductUsage
            }
            "experience_behavior"
            | "behavior_frequency"
            | "experience"
            | "perilaku"
            | "pengalaman" => Self::ExperienceBehavior,
            "preference_opinion" | "preference" | "opinion" | "opini" | "preferensi" => {
                Self::PreferenceOpinion
            }
            "satisfaction_rating" | "satisfaction" | "rating" | "kepuasan" => {
                Self::SatisfactionRating
            }
            "multiple_choice" | "pilihan_ganda" => Self::MultipleChoice,
            "matrix_grid" | "matrix" | "grid" | "matriks" => Self::MatrixGrid,
            "numeric" | "angka" | "number" => Self::Numeric,
            "date" | "tanggal" => Self::Date,
            "free_text" | "text" | "esai" => Self::FreeText,
            _ => Self::OtherUnknown,
        }
    }
}

/// Tingkat keyakinan hasil klasifikasi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassificationConfidence {
    High,
    Medium,
    Low,
    Uncertain,
}

impl ClassificationConfidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Uncertain => "uncertain",
        }
    }
}

/// Asal usul batasan (eksplisit tertulis vs inferensi)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintProvenance {
    /// Benar-benar tertulis dalam teks pertanyaan atau instruksi
    Explicit,
    /// Dihasilkan dari inferensi bentuk opsi atau pola struktur
    Inferred,
}

/// Moda pemilihan opsi jawaban
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", content = "count", rename_all = "snake_case")]
pub enum SelectionMode {
    Single,
    Multiple,
    Exact(u32),
    AtLeast(u32),
    AtMost(u32),
    Unspecified,
}

/// Tipe periode referensi (timeframe)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeframeType {
    PastNDays,
    PastNWeeks,
    PastNMonths,
    PastNYears,
    Recurring,
    SpecificYear,
    Custom,
}

/// Batasan periode referensi waktu (Timeframe)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeframeConstraint {
    pub raw_text: String,
    pub timeframe_type: TimeframeType,
    pub count: Option<u32>,
    pub provenance: ConstraintProvenance,
}

/// Batasan unit/satuan pengukuran
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitConstraint {
    pub unit_name: String,
    pub raw_text: String,
    pub provenance: ConstraintProvenance,
}

/// Batasan rentang angka
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumericRangeConstraint {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub provenance: ConstraintProvenance,
}

/// Arah skala penilaian
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScaleDirection {
    LowToHigh,
    HighToLow,
    Unspecified,
}

/// Label jangkar skala (misal 1 = Sangat Tidak Setuju, 5 = Sangat Setuju)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaleAnchor {
    pub value: i32,
    pub label: String,
}

/// Batasan skala penilaian
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaleConstraint {
    pub min_scale: i32,
    pub max_scale: i32,
    pub direction: ScaleDirection,
    pub anchors: Vec<ScaleAnchor>,
    pub provenance: ConstraintProvenance,
}

/// Batasan panjang teks
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextLengthConstraint {
    pub min_chars: Option<u32>,
    pub max_chars: Option<u32>,
    pub max_words: Option<u32>,
    pub provenance: ConstraintProvenance,
}

/// Kumpulan seluruh batasan yang berhasil diekstraksi
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedConstraints {
    pub units: Option<UnitConstraint>,
    pub timeframe: Option<TimeframeConstraint>,
    pub min_selections: Option<u32>,
    pub max_selections: Option<u32>,
    pub selection_mode: SelectionMode,
    pub is_required: Option<bool>,
    pub expected_format: AnswerFormat,
    pub numeric_range: Option<NumericRangeConstraint>,
    pub scale: Option<ScaleConstraint>,
    pub special_instructions: Vec<String>,
    pub text_length: Option<TextLengthConstraint>,
}

impl Default for ExtractedConstraints {
    fn default() -> Self {
        Self {
            units: None,
            timeframe: None,
            min_selections: None,
            max_selections: None,
            selection_mode: SelectionMode::Unspecified,
            is_required: None,
            expected_format: AnswerFormat::Unknown,
            numeric_range: None,
            scale: None,
            special_instructions: Vec::new(),
            text_length: None,
        }
    }
}

/// Tipe ambiguitas pertanyaan survei
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmbiguityType {
    /// Pertanyaan tidak lengkap atau terpotong
    IncompleteQuestion,
    /// Memiliki lebih dari satu interpretasi yang masuk akal
    MultipleInterpretations,
    /// Instruksi dan pilihan jawaban saling bertentangan
    ConflictingInstructions,
    /// Periode referensi (timeframe) tidak disebutkan padahal pertanyaan menanyakan frekuensi/perilaku (AT-041)
    MissingTimeframe,
    /// Unit/satuan pengukuran tidak disebutkan untuk pertanyaan bernilai kuantitas/uang (AT-041)
    MissingUnit,
    /// Pilihan opsi tidak sesuai atau tidak mencakup konteks pertanyaan
    MismatchedOptions,
}

/// Rincian alasan ambiguitas
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmbiguityReason {
    pub ambiguity_type: AmbiguityType,
    pub explanation: String,
    pub context_snippet: Option<String>,
}

/// Laporan deteksi ambiguitas
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AmbiguityReport {
    pub is_ambiguous: bool,
    pub needs_clarification: bool,
    pub reasons: Vec<AmbiguityReason>,
    pub clarification_prompt: Option<String>,
}

/// Output lengkap hasil klasifikasi dan ekstraksi batasan pertanyaan survei
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionClassificationResult {
    /// Versi parser dan classifier deterministik
    pub classifier_version: String,
    /// Teks pertanyaan yang telah dinormalisasi secara terpisah dari teks asli
    pub normalized_text: String,
    /// Kategori-kategori relevan (multi-label)
    pub categories: Vec<QuestionCategory>,
    /// Kategori utama
    pub primary_category: QuestionCategory,
    /// Tingkat keyakinan klasifikasi
    pub confidence: ClassificationConfidence,
    /// Tipe pertanyaan survei
    pub question_type: QuestionType,
    /// Format jawaban yang diharapkan
    pub answer_format: AnswerFormat,
    /// Batasan-batasan yang teridentifikasi
    pub constraints: ExtractedConstraints,
    /// Laporan deteksi ambiguitas
    pub ambiguity: AmbiguityReport,
}

/// Input DTO untuk permintaan klasifikasi pertanyaan
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassifyQuestionInput {
    pub persona_id: Option<String>,
    pub session_id: Option<String>,
    pub question_id: Option<String>,
    pub question_text: String,
    pub instructions: Option<String>,
    pub options: Option<Vec<String>>,
}
