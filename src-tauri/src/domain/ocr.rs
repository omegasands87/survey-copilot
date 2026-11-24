use serde::{Deserialize, Serialize};

use crate::domain::capture::CaptureMode;
use crate::domain::survey::{AnswerFormat, ConfirmationStatus, QuestionType};

/// Kotak pembatas (Bounding Box) untuk elemen teks dan layout
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl BoundingBox {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.bottom()
            && self.bottom() > other.y
    }

    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.right() && py >= self.y && py <= self.bottom()
    }

    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = self.right().max(other.right());
        let max_y = self.bottom().max(other.bottom());
        BoundingBox {
            x: min_x,
            y: min_y,
            width: (max_x - min_x).max(0.0),
            height: (max_y - min_y).max(0.0),
        }
    }
}

/// Kata individual hasil ekstraksi OCR beserta koordinat dan confidence
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrWord {
    pub text: String,
    pub bounding_box: BoundingBox,
    pub confidence: f32,
}

/// Baris teks hasil ekstraksi OCR
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrLine {
    pub text: String,
    pub bounding_box: BoundingBox,
    pub words: Vec<OcrWord>,
    pub confidence: f32,
}

/// Peran struktural suatu blok teks dalam tata letak survei
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockRole {
    Prompt,
    Instruction,
    Option,
    ScaleLabel,
    ScaleNumber,
    MatrixRow,
    MatrixColumn,
    Unit,
    RequiredIndicator,
    Noise,
    Unknown,
}

impl BlockRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Prompt => "prompt",
            Self::Instruction => "instruction",
            Self::Option => "option",
            Self::ScaleLabel => "scale_label",
            Self::ScaleNumber => "scale_number",
            Self::MatrixRow => "matrix_row",
            Self::MatrixColumn => "matrix_column",
            Self::Unit => "unit",
            Self::RequiredIndicator => "required_indicator",
            Self::Noise => "noise",
            Self::Unknown => "unknown",
        }
    }
}

/// Blok teks terstruktur hasil rekonstruksi layout
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrTextBlock {
    pub id: String,
    pub text: String,
    pub bounding_box: BoundingBox,
    pub confidence: f32,
    pub words: Vec<OcrWord>,
    pub role: BlockRole,
}

/// Alasan ketidakpastian / ambiguitas layout OCR
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyReason {
    LowConfidence,
    LikelyClipped,
    AmbiguousLayout,
    OverlappingElements,
}

impl UncertaintyReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LowConfidence => "low_confidence",
            Self::LikelyClipped => "likely_clipped",
            Self::AmbiguousLayout => "ambiguous_layout",
            Self::OverlappingElements => "overlapping_elements",
        }
    }
}

/// Wilayah gambar atau teks yang memiliki tingkat keyakinan rendah atau tata letak ambigu (AT-052)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UncertainRegion {
    pub id: String,
    pub reason: UncertaintyReason,
    pub bounding_box: Option<BoundingBox>,
    pub text: Option<String>,
    pub details: String,
}

/// Struktur skala penilaian (Rating Scale)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaleStructure {
    pub min_value: i64,
    pub max_value: i64,
    pub min_label: Option<String>,
    pub max_label: Option<String>,
    pub all_labels: Vec<String>,
    pub step: Option<i64>,
}

/// Struktur tabel matriks / kisi (Matrix Grid Table)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixStructure {
    pub rows: Vec<String>,
    pub columns: Vec<String>,
    pub cell_matrix: Vec<Vec<Option<String>>>,
    pub is_multi_select: bool,
}

/// Hasil mentah pemindaian dari OCR engine sebelum rekonstruksi layout
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrRawResult {
    pub raw_text: String,
    pub text_angle: f64,
    pub lines: Vec<OcrLine>,
    pub engine: String,
    pub image_width: u32,
    pub image_height: u32,
}

/// Entitas terstruktur hasil rekonstruksi layout survei dari OCR (SC-041)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedSurveyQuestion {
    pub extraction_id: String,
    pub capture_id: Option<String>,
    pub session_id: Option<String>,
    pub persona_id: Option<String>,
    pub source_mode: CaptureMode,
    pub captured_at: String,
    pub extracted_at: String,
    pub ocr_engine: String,
    pub application_label: Option<String>,
    pub image_width: u32,
    pub image_height: u32,

    // Teks Asli & Teks Koreksi Pengguna (Imutabilitas Teks Sumber)
    pub raw_ocr_text: String,
    pub question_text_original: String,
    pub question_text_corrected: Option<String>,
    pub instructions: Option<String>,
    pub answer_options: Vec<String>,
    pub question_type: QuestionType,
    pub answer_format: AnswerFormat,

    // Skala & Matriks
    pub scale: Option<ScaleStructure>,
    pub matrix: Option<MatrixStructure>,

    // Unit & Indikator Wajib
    pub visible_units: Option<String>,
    pub is_required: Option<bool>,

    // Blok Teks, Ambiguitas & Peringatan Kualitas (AT-052)
    pub text_blocks: Vec<OcrTextBlock>,
    pub uncertain_regions: Vec<UncertainRegion>,
    pub warnings: Vec<String>,
    pub overall_confidence: f32,

    // Status Konfirmasi
    pub confirmation_status: ConfirmationStatus,
}

/// DTO Input untuk menjalankan OCR pada tangkapan layar tertentu
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractOcrInput {
    pub capture_id: String,
    pub session_id: Option<String>,
    pub persona_id: Option<String>,
    pub engine: Option<String>,
}

/// DTO Input untuk mengonfirmasi / mengoreksi hasil ekstraksi OCR sebelum analisis AI (PR-008 / AT-052)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmOcrExtractionInput {
    pub extraction_id: String,
    pub session_id: String,
    pub persona_id: String,
    pub capture_id: Option<String>,
    pub question_text_corrected: String,
    pub instructions: Option<String>,
    pub options: Vec<String>,
    pub question_type: Option<QuestionType>,
    pub answer_format: Option<AnswerFormat>,
    pub scale_min: Option<i64>,
    pub scale_max: Option<i64>,
    pub scale_labels: Option<Vec<String>>,
    pub matrix_rows: Option<Vec<String>>,
    pub matrix_columns: Option<Vec<String>>,
    pub visible_units: Option<String>,
}
