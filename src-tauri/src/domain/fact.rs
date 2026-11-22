use super::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationStatus {
    Confirmed,
    Unverified,
    Disputed,
    Outdated,
}

impl VerificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Unverified => "unverified",
            Self::Disputed => "disputed",
            Self::Outdated => "outdated",
        }
    }

    pub fn parse(s: &str) -> Result<Self, AppError> {
        match s {
            "confirmed" => Ok(Self::Confirmed),
            "unverified" => Ok(Self::Unverified),
            "disputed" => Ok(Self::Disputed),
            "outdated" => Ok(Self::Outdated),
            _ => Err(AppError::validation(
                "verificationStatus",
                "Status verifikasi tidak valid. Pilihan: confirmed, unverified, disputed, outdated.",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sensitivity {
    Ordinary,
    Sensitive,
    HighlySensitive,
}

impl Sensitivity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ordinary => "ordinary",
            Self::Sensitive => "sensitive",
            Self::HighlySensitive => "highly_sensitive",
        }
    }

    pub fn parse(s: &str) -> Result<Self, AppError> {
        match s {
            "ordinary" => Ok(Self::Ordinary),
            "sensitive" => Ok(Self::Sensitive),
            "highly_sensitive" => Ok(Self::HighlySensitive),
            _ => Err(AppError::validation(
                "sensitivity",
                "Klasifikasi sensitivitas tidak valid. Pilihan: ordinary, sensitive, highly_sensitive.",
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FactSummary {
    pub id: String,
    pub category: String,
    pub attribute: String,
    pub value: serde_json::Value,
    pub verification_status: String,
    pub sensitivity: String,
    pub source_description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFactInput {
    pub persona_id: String,
    pub category: String,
    pub attribute: String,
    pub value: String,
    pub source_description: String,
    pub verification_status: String,
    pub sensitivity: String,
}

pub struct ValidatedFactInput {
    pub persona_id: String,
    pub category: String,
    pub attribute: String,
    pub value: String,
    pub source_description: String,
    pub verification_status: VerificationStatus,
    pub sensitivity: Sensitivity,
}

impl CreateFactInput {
    pub fn validate(&self) -> Result<ValidatedFactInput, AppError> {
        let persona_id = self.persona_id.trim();
        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "ID profil wajib diisi."));
        }

        let category = self.category.trim();
        let attribute = self.attribute.trim();
        let value = self.value.trim();
        let source_description = self.source_description.trim();

        if category.is_empty() || attribute.is_empty() || value.is_empty() {
            return Err(AppError::validation(
                "general",
                "Kategori, informasi, dan nilai wajib diisi.",
            ));
        }

        if source_description.is_empty() {
            return Err(AppError::validation(
                "sourceDescription",
                "Sumber informasi wajib diisi.",
            ));
        }

        if category.chars().count() > 80 {
            return Err(AppError::validation(
                "category",
                "Kategori maksimal 80 karakter.",
            ));
        }
        if attribute.chars().count() > 100 {
            return Err(AppError::validation(
                "attribute",
                "Nama informasi maksimal 100 karakter.",
            ));
        }
        if value.chars().count() > 4000 {
            return Err(AppError::validation(
                "value",
                "Nilai informasi maksimal 4000 karakter.",
            ));
        }
        if source_description.chars().count() > 500 {
            return Err(AppError::validation(
                "sourceDescription",
                "Deskripsi sumber maksimal 500 karakter.",
            ));
        }

        let verification_status = VerificationStatus::parse(&self.verification_status)?;
        let sensitivity = Sensitivity::parse(&self.sensitivity)?;

        Ok(ValidatedFactInput {
            persona_id: persona_id.to_string(),
            category: category.to_string(),
            attribute: attribute.to_string(),
            value: value.to_string(),
            source_description: source_description.to_string(),
            verification_status,
            sensitivity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_empty_fields() {
        let input = CreateFactInput {
            persona_id: "p1".to_string(),
            category: "".to_string(),
            attribute: "Pekerjaan".to_string(),
            value: "Insinyur".to_string(),
            source_description: "KTP".to_string(),
            verification_status: "confirmed".to_string(),
            sensitivity: "ordinary".to_string(),
        };
        assert!(input.validate().is_err());
    }

    #[test]
    fn validates_invalid_status_and_sensitivity() {
        let input_bad_status = CreateFactInput {
            persona_id: "p1".to_string(),
            category: "Pekerjaan".to_string(),
            attribute: "Jabatan".to_string(),
            value: "Lead".to_string(),
            source_description: "KTP".to_string(),
            verification_status: "invalid_status".to_string(),
            sensitivity: "ordinary".to_string(),
        };
        assert!(input_bad_status.validate().is_err());

        let input_bad_sensitivity = CreateFactInput {
            persona_id: "p1".to_string(),
            category: "Pekerjaan".to_string(),
            attribute: "Jabatan".to_string(),
            value: "Lead".to_string(),
            source_description: "KTP".to_string(),
            verification_status: "confirmed".to_string(),
            sensitivity: "super_secret".to_string(),
        };
        assert!(input_bad_sensitivity.validate().is_err());
    }

    #[test]
    fn accepts_valid_fact_input() {
        let input = CreateFactInput {
            persona_id: "p1".to_string(),
            category: " Pekerjaan ".to_string(),
            attribute: " Bidang Usaha ".to_string(),
            value: " Teknologi Informasi ".to_string(),
            source_description: " Manual input ".to_string(),
            verification_status: "confirmed".to_string(),
            sensitivity: "ordinary".to_string(),
        };
        let validated = input.validate().expect("should validate");
        assert_eq!(validated.category, "Pekerjaan");
        assert_eq!(validated.attribute, "Bidang Usaha");
        assert_eq!(validated.value, "Teknologi Informasi");
        assert_eq!(validated.verification_status, VerificationStatus::Confirmed);
        assert_eq!(validated.sensitivity, Sensitivity::Ordinary);
    }
}
