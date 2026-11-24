use serde::{Deserialize, Serialize};

use crate::domain::classifier::QuestionClassificationResult;
use crate::domain::fact::{Sensitivity, VerificationStatus};

/// Status masa berlaku fakta relatif terhadap waktu evaluasi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidityStatus {
    /// Fakta masih berlaku saat ini
    Valid,
    /// Fakta telah melewati batas akhir masa berlaku
    Expired,
    /// Fakta baru berlaku pada masa mendatang
    Future,
    /// Tidak memiliki batasan tanggal berlaku eksplisit (tetap berlaku umum)
    Unspecified,
}

impl ValidityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Expired => "expired",
            Self::Future => "future",
            Self::Unspecified => "unspecified",
        }
    }
}

/// Tipe konflik bukti dan konsistensi
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    /// Dua fakta profil untuk atribut yang sama memiliki nilai yang saling bertentangan
    FactConflict,
    /// Jawaban terkonfirmasi di masa lalu bertentangan dengan fakta profil atau jawaban saat ini
    HistoricalContradiction,
    /// Periode waktu pada fakta tidak sesuai dengan timeframe yang ditanyakan survei
    TimeframeMismatch,
    /// Fakta mengalami perubahan nilai seiring waktu dan memerlukan konfirmasi periode
    ValueChangeOverTime,
    /// Satuan unit pengukuran tidak sesuai atau bertentangan
    UnitMismatch,
}

/// Tingkat keparahan konflik
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictSeverity {
    /// Menghalangi pemberian jawaban definitif; klarifikasi mutlak diperlukan
    Critical,
    /// Perbedaan potensial yang perlu diperhatikan pengguna
    Warning,
    /// Catatan historis atau variasi kecil yang tidak membatalkan bukti
    Note,
}

/// Indikator konflik bukti yang terdeteksi
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceConflictIndicator {
    pub conflict_type: ConflictType,
    pub severity: ConflictSeverity,
    pub description: String,
    pub primary_fact_id: Option<String>,
    pub primary_source_id: Option<String>,
    pub conflicting_fact_id: Option<String>,
    pub conflicting_source_id: Option<String>,
    pub historical_question_id: Option<String>,
    pub historical_question_text: Option<String>,
    pub historical_answer: Option<String>,
    pub resolution_prompt: Option<String>,
}

/// Tingkat keyakinan total bukti
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceConfidence {
    /// Bukti kuat: fakta confirmed, valid, relevan tinggi, dan bebas konflik
    High,
    /// Bukti sedang: fakta relevan ada namun memiliki keterbatasan kecil atau status unverified
    Medium,
    /// Bukti rendah: fakta relevan minim atau terdapat peringatan waktu
    Low,
    /// Tidak dapat dipastikan: terjadi konflik kritis atau tidak ada bukti sama sekali
    Uncertain,
}

impl EvidenceConfidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Uncertain => "uncertain",
        }
    }
}

/// Rekomendasi aksi berdasarkan kecukupan bukti
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRecommendation {
    /// Bukti cukup dan terverifikasi untuk menghasilkan draf jawaban
    Supported,
    /// Terjadi konflik atau ambiguitas; klarifikasi pengguna diperlukan sebelum menjawab
    NeedsClarification,
    /// Tidak ada fakta yang relevan; sistem harus abstain (tidak boleh mengarang)
    InsufficientEvidence,
}

impl EvidenceRecommendation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::NeedsClarification => "needs_clarification",
            Self::InsufficientEvidence => "insufficient_evidence",
        }
    }
}

/// Item bukti fakta individual yang berhasil diambil dan diperingkat
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievedEvidenceItem {
    pub fact_id: String,
    pub source_id: Option<String>,
    pub fact_key: String,
    pub attribute: String,
    pub category: String,
    pub value: serde_json::Value,
    pub normalized_value: String,
    pub unit: Option<String>,
    pub verification_status: VerificationStatus,
    pub validity_status: ValidityStatus,
    pub sensitivity: Sensitivity,
    /// Skor relevansi semantik dan kontekstual (0.0 sampai 1.0)
    pub relevance_score: f64,
    /// Alasan terperinci mengapa fakta ini dipilih
    pub selection_reason: String,
    pub source_type: Option<String>,
    pub source_title: Option<String>,
    pub source_description: Option<String>,
    pub source_date: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
}

/// Izin persetujuan pengguna untuk menyertakan fakta sensitif dan analisis visual eksternal (SC-060)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SensitivityConsent {
    /// Mengizinkan penggunaan fakta berlabel 'sensitive' (misal pendapatan, agama)
    pub allow_sensitive: bool,
    /// Mengizinkan penggunaan fakta berlabel 'highly_sensitive' (misal data medis, KTP)
    pub allow_highly_sensitive: bool,
    /// Mengizinkan pengiriman tangkapan layar/gambar ke model visual eksternal (External Vision)
    #[serde(default)]
    pub allow_external_vision: bool,
}

/// DTO Input Permintaan Pengambilan Bukti & Evaluasi Konsistensi
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRetrievalInput {
    pub persona_id: String,
    pub session_id: Option<String>,
    pub question_id: Option<String>,
    pub question_text: String,
    pub instructions: Option<String>,
    pub options: Option<Vec<String>>,
    pub classification: Option<QuestionClassificationResult>,
    pub consent: Option<SensitivityConsent>,
    pub max_evidence_items: Option<usize>,
}

/// Kontrak Output Paket Bukti Lengkap (Evidence Package) untuk SC-033
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidencePackage {
    pub question_id: Option<String>,
    pub active_persona_id: String,
    pub retrieved_evidence: Vec<RetrievedEvidenceItem>,
    pub conflict_indicators: Vec<EvidenceConflictIndicator>,
    pub missing_information: Vec<String>,
    pub evidence_confidence: EvidenceConfidence,
    pub recommendation: EvidenceRecommendation,
    pub evaluation_summary: String,
    pub evaluated_at: String,
}
