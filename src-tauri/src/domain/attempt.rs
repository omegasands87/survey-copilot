use super::opportunity::OpportunityCriterion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status siklus dan hasil setiap percobaan survei (SC-052)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeLabel {
    Qualified,
    ScreenedOut,
    QuotaFull,
    Completed,
    QualityRejected,
    TechnicalFailure,
    AbandonedByUser,
    Pending,
    Unknown,
}

impl OutcomeLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Qualified => "qualified",
            Self::ScreenedOut => "screened_out",
            Self::QuotaFull => "quota_full",
            Self::Completed => "completed",
            Self::QualityRejected => "quality_rejected",
            Self::TechnicalFailure => "technical_failure",
            Self::AbandonedByUser => "abandoned_by_user",
            Self::Pending => "pending",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "qualified" | "lolos" | "lolos_skrining" => Self::Qualified,
            "screened_out" | "gagal_skrining" | "skrining_gagal" => Self::ScreenedOut,
            "quota_full" | "kuota_penuh" => Self::QuotaFull,
            "completed" | "selesai" | "selesai_penuh" => Self::Completed,
            "quality_rejected" | "ditolak_kualitas" | "reject" => Self::QualityRejected,
            "technical_failure" | "kegagalan_teknis" | "error" => Self::TechnicalFailure,
            "abandoned_by_user" | "dibatalkan_pengguna" | "batal" => Self::AbandonedByUser,
            "pending" | "sedang_berjalan" | "aktif" => Self::Pending,
            _ => Self::Unknown,
        }
    }

    pub fn display_name_id(&self) -> &'static str {
        match self {
            Self::Qualified => "Lolos Skrining (Qualified)",
            Self::ScreenedOut => "Gagal Skrining (Screened Out)",
            Self::QuotaFull => "Kuota Penuh (Quota Full)",
            Self::Completed => "Selesai Penuh (Completed)",
            Self::QualityRejected => "Ditolak Kualitas (Quality Rejected)",
            Self::TechnicalFailure => "Kegagalan Teknis (Technical Failure)",
            Self::AbandonedByUser => "Dibatalkan Pengguna (Abandoned)",
            Self::Pending => "Sedang Berjalan (Pending)",
            Self::Unknown => "Tidak Diketahui (Unknown)",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::ScreenedOut
                | Self::QuotaFull
                | Self::Completed
                | Self::QualityRejected
                | Self::TechnicalFailure
                | Self::AbandonedByUser
        )
    }
}

/// Derajat keyakinan atas bukti hasil survei yang teramati
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeConfidence {
    Observed,
    UserConfirmed,
    Uncertain,
}

impl OutcomeConfidence {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Observed => "observed",
            Self::UserConfirmed => "user_confirmed",
            Self::Uncertain => "uncertain",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "user_confirmed" | "dikonfirmasi" | "confirmed" => Self::UserConfirmed,
            "uncertain" | "ragu" | "tidak_pasti" => Self::Uncertain,
            _ => Self::Observed,
        }
    }

    pub fn display_name_id(&self) -> &'static str {
        match self {
            Self::Observed => "Teramati Langsung",
            Self::UserConfirmed => "Dikonfirmasi Pengguna",
            Self::Uncertain => "Belum Pasti",
        }
    }
}

/// Entitas utama percobaan survei (`survey_attempts`)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurveyAttempt {
    pub id: String,
    pub opportunity_id: String,
    pub persona_id: String,
    pub session_id: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub current_outcome: OutcomeLabel,
    pub created_at: String,
    pub updated_at: String,
}

/// Peristiwa kronologis perubahan hasil percobaan (`survey_outcome_events`)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurveyOutcomeEvent {
    pub id: String,
    pub attempt_id: String,
    pub outcome_label: OutcomeLabel,
    pub evidence_source: String,
    pub evidence_note: Option<String>,
    pub confidence: OutcomeConfidence,
    pub observed_at: String,
    pub created_at: String,
}

/// Tampilan detail lengkap percobaan survei beserta konteks peluang, persona, kriteria teramati, dan riwayat event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurveyAttemptDetail {
    pub id: String,
    pub opportunity_id: String,
    pub opportunity_title: String,
    pub source_platform: String,
    pub category: Option<String>,
    pub target_url: Option<String>,
    pub raw_text: Option<String>,
    pub persona_id: String,
    pub persona_name: String,
    pub session_id: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub current_outcome: OutcomeLabel,
    pub criteria: Vec<OpportunityCriterion>,
    pub events: Vec<SurveyOutcomeEvent>,
    pub created_at: String,
    pub updated_at: String,
}

/// Filter pencarian riwayat percobaan survei (persona, kategori, sumber, status, rentang waktu)
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttemptFilter {
    pub persona_id: Option<String>,
    pub category: Option<String>,
    pub source_platform: Option<String>,
    pub status: Option<OutcomeLabel>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

/// Input pencatatan percobaan survei baru
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAttemptInput {
    pub persona_id: String,
    pub opportunity_id: Option<String>,
    pub opportunity_title: Option<String>,
    pub source_platform: Option<String>,
    pub category: Option<String>,
    pub target_url: Option<String>,
    pub raw_text: Option<String>,
    pub session_id: Option<String>,
    pub started_at: Option<String>,
    pub initial_outcome: Option<OutcomeLabel>,
    pub evidence_source: Option<String>,
    pub evidence_note: Option<String>,
    pub confidence: Option<OutcomeConfidence>,
}

/// Input penambahan peristiwa hasil survei baru
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordOutcomeEventInput {
    pub attempt_id: String,
    pub persona_id: String,
    pub outcome_label: OutcomeLabel,
    pub evidence_source: String,
    pub evidence_note: Option<String>,
    pub confidence: Option<OutcomeConfidence>,
    pub observed_at: Option<String>,
}

/// Status ketersediaan data pembelajaran historis (AT-063)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningDataStatus {
    InsufficientData,
    SufficientData,
}

impl LearningDataStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InsufficientData => "insufficient_data",
            Self::SufficientData => "sufficient_data",
        }
    }
}

/// Ringkasan pembelajaran historis hasil survei tanpa manipulasi fakta atau fabrikasi probabilitas
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalLearningSummary {
    pub persona_id: String,
    pub persona_name: String,
    pub category: Option<String>,
    pub source_platform: Option<String>,
    pub sample_size: usize,
    pub minimum_required_sample: usize,
    pub status: LearningDataStatus,
    pub observation_window_start: Option<String>,
    pub observation_window_end: Option<String>,
    pub counts_by_outcome: HashMap<String, usize>,
    pub completion_rate: Option<f64>,
    pub qualification_rate: Option<f64>,
    pub validation_method: String,
    pub holdout_sample_size: usize,
    pub holdout_consistency_rate: Option<f64>,
    pub message: String,
    pub uncertainty_disclosure: String,
    pub disclaimers: Vec<String>,
}
