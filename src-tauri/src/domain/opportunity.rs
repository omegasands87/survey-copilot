use serde::{Deserialize, Serialize};

/// Status siklus peluang survei (SC-050 / SC-051)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityStatus {
    Captured,
    CriteriaExtracted,
    Confirmed,
    Evaluated,
    Archived,
}

impl OpportunityStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Captured => "captured",
            Self::CriteriaExtracted => "criteria_extracted",
            Self::Confirmed => "confirmed",
            Self::Evaluated => "evaluated",
            Self::Archived => "archived",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "criteria_extracted" => Self::CriteriaExtracted,
            "confirmed" => Self::Confirmed,
            "evaluated" => Self::Evaluated,
            "archived" => Self::Archived,
            _ => Self::Captured,
        }
    }
}

/// Asal-usul data tangkapan peluang survei
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityProvenance {
    ManualEntry,
    ScreenCapture,
    OcrDetected,
}

impl OpportunityProvenance {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ManualEntry => "manual_entry",
            Self::ScreenCapture => "screen_capture",
            Self::OcrDetected => "ocr_detected",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "screen_capture" => Self::ScreenCapture,
            "ocr_detected" => Self::OcrDetected,
            _ => Self::ManualEntry,
        }
    }
}

/// Tipe kriteria kelayakan eksplisit survei
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriterionType {
    Age,
    Gender,
    Location,
    Occupation,
    ProductUsage,
    Education,
    Income,
    MaritalStatus,
    General,
}

impl CriterionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Age => "age",
            Self::Gender => "gender",
            Self::Location => "location",
            Self::Occupation => "occupation",
            Self::ProductUsage => "product_usage",
            Self::Education => "education",
            Self::Income => "income",
            Self::MaritalStatus => "marital_status",
            Self::General => "general",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "age" => Self::Age,
            "gender" => Self::Gender,
            "location" => Self::Location,
            "occupation" => Self::Occupation,
            "product_usage" => Self::ProductUsage,
            "education" => Self::Education,
            "income" => Self::Income,
            "marital_status" => Self::MaritalStatus,
            _ => Self::General,
        }
    }
}

/// Operator perbandingan untuk kriteria deterministik
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriterionOperator {
    Equals,
    Range,
    GreaterThanOrEqual,
    LessThanOrEqual,
    In,
    Contains,
    NotEquals,
}

impl CriterionOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Equals => "equals",
            Self::Range => "range",
            Self::GreaterThanOrEqual => "greater_than_or_equal",
            Self::LessThanOrEqual => "less_than_or_equal",
            Self::In => "in",
            Self::Contains => "contains",
            Self::NotEquals => "not_equals",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        match s {
            "range" => Self::Range,
            "greater_than_or_equal" | "gte" | "min" => Self::GreaterThanOrEqual,
            "less_than_or_equal" | "lte" | "max" => Self::LessThanOrEqual,
            "in" => Self::In,
            "contains" => Self::Contains,
            "not_equals" | "excludes" => Self::NotEquals,
            _ => Self::Equals,
        }
    }
}

/// Entitas Peluang Survei yang tertangkap
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SurveyOpportunity {
    pub id: String,
    pub title: Option<String>,
    pub platform_label: Option<String>,
    pub topic: Option<String>,
    pub description: Option<String>,
    pub raw_text: Option<String>,
    pub provenance: OpportunityProvenance,
    pub status: OpportunityStatus,
    pub source_capture_id: Option<String>,
    pub source_ocr_id: Option<String>,
    pub captured_at: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Entitas Kriteria Kelayakan Eksplisit yang Terkait dengan Peluang Survei
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpportunityCriterion {
    pub id: String,
    pub opportunity_id: String,
    pub criterion_text: String,
    pub raw_text: Option<String>,
    pub criterion_type: CriterionType,
    pub normalized_attribute: Option<String>,
    pub expected_value_json: Option<serde_json::Value>,
    pub operator: CriterionOperator,
    pub criterion_order: i64,
}

/// Struktur lengkap peluang survei beserta daftar kriteria eksplisitnya
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpportunityDetail {
    pub opportunity: SurveyOpportunity,
    pub criteria: Vec<OpportunityCriterion>,
}

/// Input DTO untuk pembuatan peluang survei baru
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOpportunityInput {
    pub title: Option<String>,
    pub platform_label: Option<String>,
    pub topic: Option<String>,
    pub description: Option<String>,
    pub raw_text: String,
    pub provenance: Option<OpportunityProvenance>,
    pub source_capture_id: Option<String>,
    pub source_ocr_id: Option<String>,
    pub captured_at: Option<String>,
}

/// Input DTO untuk pembaruan metadata peluang survei
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOpportunityInput {
    pub id: String,
    pub title: Option<String>,
    pub platform_label: Option<String>,
    pub topic: Option<String>,
    pub description: Option<String>,
    pub status: Option<OpportunityStatus>,
}

/// Input DTO untuk kriteria individu
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CriterionInput {
    pub id: Option<String>,
    pub criterion_text: String,
    pub raw_text: Option<String>,
    pub criterion_type: CriterionType,
    pub normalized_attribute: Option<String>,
    pub expected_value_json: Option<serde_json::Value>,
    pub operator: CriterionOperator,
    pub criterion_order: i64,
}

/// Input DTO untuk menyimpan peluang survei sekaligus seluruh kriterianya
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveOpportunityWithCriteriaInput {
    pub opportunity: CreateOpportunityInput,
    pub criteria: Vec<CriterionInput>,
}

/// Input DTO untuk memperbarui daftar kriteria suatu peluang survei
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCriteriaInput {
    pub opportunity_id: String,
    pub criteria: Vec<CriterionInput>,
}
