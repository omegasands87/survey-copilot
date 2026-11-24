use super::opportunity::{CriterionOperator, CriterionType};
use serde::{Deserialize, Serialize};

/// Status kesesuaian per kriteria (SC-051 / AT-060 / AT-061 / AT-062)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchState {
    Match,
    Mismatch,
    Unknown,
    Stale,
    Conflict,
}

impl MatchState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::Mismatch => "mismatch",
            Self::Unknown => "unknown",
            Self::Stale => "stale",
            Self::Conflict => "conflict",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "match" | "cocok" => Self::Match,
            "mismatch" | "tidak_cocok" => Self::Mismatch,
            "stale" | "kadaluarsa" => Self::Stale,
            "conflict" | "konflik" => Self::Conflict,
            _ => Self::Unknown,
        }
    }

    pub fn display_name_id(&self) -> &'static str {
        match self {
            Self::Match => "Cocok",
            Self::Mismatch => "Tidak Cocok",
            Self::Unknown => "Belum Diketahui",
            Self::Stale => "Perlu Pembaruan",
            Self::Conflict => "Konflik Data",
        }
    }
}

/// Kesimpulan kesesuaian keseluruhan untuk suatu persona terhadap peluang survei
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverallFitState {
    Eligible,
    Ineligible,
    Conflict,
    Stale,
    Incomplete,
    NoCriteria,
}

impl OverallFitState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Eligible => "eligible",
            Self::Ineligible => "ineligible",
            Self::Conflict => "conflict",
            Self::Stale => "stale",
            Self::Incomplete => "incomplete",
            Self::NoCriteria => "no_criteria",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "eligible" | "cocok" => Self::Eligible,
            "ineligible" | "tidak_cocok" => Self::Ineligible,
            "conflict" | "konflik" => Self::Conflict,
            "stale" | "kadaluarsa" => Self::Stale,
            "no_criteria" | "tidak_ada_kriteria" => Self::NoCriteria,
            _ => Self::Incomplete,
        }
    }

    pub fn display_name_id(&self) -> &'static str {
        match self {
            Self::Eligible => "Cocok Memenuhi Syarat",
            Self::Ineligible => "Tidak Memenuhi Syarat",
            Self::Conflict => "Konflik Data Profil",
            Self::Stale => "Data Profil Kadaluarsa",
            Self::Incomplete => "Data Profil Belum Lengkap",
            Self::NoCriteria => "Tidak Ada Kriteria Tertangkap",
        }
    }
}

/// Rekaman entitas mentah di tabel `survey_fit_results`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SurveyFitRecord {
    pub id: String,
    pub opportunity_id: String,
    pub persona_id: String,
    pub criterion_id: String,
    pub match_state: MatchState,
    pub supporting_fact_ids_json: String,
    pub explanation: String,
    pub evaluated_at: String,
}

/// Hasil evaluasi per kriteria yang disajikan ke UI / pemanggil
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FitCriterionResult {
    pub criterion_id: String,
    pub criterion_text: String,
    pub raw_text: Option<String>,
    pub criterion_type: CriterionType,
    pub operator: CriterionOperator,
    pub match_state: MatchState,
    pub supporting_fact_ids: Vec<String>,
    pub supporting_facts_summary: Vec<String>,
    pub explanation: String,
}

/// Detail lengkap hasil evaluasi kesesuaian satu peluang terhadap satu persona
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersonaOpportunityFitDetail {
    pub opportunity_id: String,
    pub persona_id: String,
    pub persona_name: String,
    pub overall_fit: OverallFitState,
    pub match_count: usize,
    pub mismatch_count: usize,
    pub unknown_count: usize,
    pub stale_count: usize,
    pub conflict_count: usize,
    pub total_criteria: usize,
    pub criteria_results: Vec<FitCriterionResult>,
    pub evaluated_at: String,
}

/// Item perbandingan kesesuaian lintas persona untuk truthful respondent selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersonaFitComparisonItem {
    pub persona_id: String,
    pub persona_name: String,
    pub overall_fit: OverallFitState,
    pub match_count: usize,
    pub mismatch_count: usize,
    pub unknown_count: usize,
    pub stale_count: usize,
    pub conflict_count: usize,
    pub total_criteria: usize,
    pub key_reasons: Vec<String>,
    pub evaluated_at: Option<String>,
}
