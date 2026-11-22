use super::error::AppError;
use serde::{Deserialize, Serialize};

/// Ringkasan persona untuk daftar dan selector
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersonaSummary {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
    pub facts_count: i64,
    pub sessions_count: i64,
}

/// Detail lengkap persona beserta ringkasan metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersonaDetail {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
    pub facts_count: i64,
    pub sessions_count: i64,
}

/// Input pembuatan persona baru
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePersonaInput {
    pub display_name: String,
    pub description: Option<String>,
}

impl CreatePersonaInput {
    pub fn new(display_name: impl Into<String>) -> Self {
        Self {
            display_name: display_name.into(),
            description: None,
        }
    }

    pub fn with_description(display_name: impl Into<String>, description: Option<String>) -> Self {
        Self {
            display_name: display_name.into(),
            description,
        }
    }

    pub fn validate(&self) -> Result<(String, Option<String>), AppError> {
        let trimmed_name = self.display_name.trim();
        let length = trimmed_name.chars().count();

        if length == 0 || length > 80 {
            return Err(AppError::validation(
                "displayName",
                "Nama profil harus berisi 1 sampai 80 karakter.",
            ));
        }

        let clean_desc = self.description.as_ref().and_then(|desc| {
            let t = desc.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        });

        if let Some(ref d) = clean_desc {
            if d.chars().count() > 500 {
                return Err(AppError::validation(
                    "description",
                    "Deskripsi profil tidak boleh melebihi 500 karakter.",
                ));
            }
        }

        Ok((trimmed_name.to_string(), clean_desc))
    }
}

/// Input pembaruan/rename persona
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePersonaInput {
    pub display_name: String,
    pub description: Option<String>,
}

impl UpdatePersonaInput {
    pub fn new(display_name: impl Into<String>, description: Option<String>) -> Self {
        Self {
            display_name: display_name.into(),
            description,
        }
    }

    pub fn validate(&self) -> Result<(String, Option<String>), AppError> {
        let trimmed_name = self.display_name.trim();
        let length = trimmed_name.chars().count();

        if length == 0 || length > 80 {
            return Err(AppError::validation(
                "displayName",
                "Nama profil harus berisi 1 sampai 80 karakter.",
            ));
        }

        let clean_desc = self.description.as_ref().and_then(|desc| {
            let t = desc.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        });

        if let Some(ref d) = clean_desc {
            if d.chars().count() > 500 {
                return Err(AppError::validation(
                    "description",
                    "Deskripsi profil tidak boleh melebihi 500 karakter.",
                ));
            }
        }

        Ok((trimmed_name.to_string(), clean_desc))
    }
}

/// Ringkasan dampak ketergantungan data sebelum penghapusan permanen
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersonaDependencySummary {
    pub persona_id: String,
    pub display_name: String,
    pub facts_count: i64,
    pub sources_count: i64,
    pub sessions_count: i64,
    pub questions_count: i64,
    pub recommendations_count: i64,
    pub answers_count: i64,
    pub fit_results_count: i64,
    pub attempts_count: i64,
    pub total_associated_records: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_empty_display_name() {
        let input = CreatePersonaInput::new("   ");
        let result = input.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
    }

    #[test]
    fn validates_oversized_display_name() {
        let long_name = "a".repeat(81);
        let input = CreatePersonaInput::new(long_name);
        let result = input.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
    }

    #[test]
    fn accepts_valid_display_name_and_description() {
        let input = CreatePersonaInput::with_description(
            "  Responden Utama  ",
            Some("  Catatan profil asli  ".to_string()),
        );
        let result = input.validate();
        assert!(result.is_ok());
        let (name, desc) = result.unwrap();
        assert_eq!(name, "Responden Utama");
        assert_eq!(desc, Some("Catatan profil asli".to_string()));
    }

    #[test]
    fn validates_oversized_description() {
        let long_desc = "d".repeat(501);
        let input = UpdatePersonaInput::new("Budi", Some(long_desc));
        let result = input.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
    }
}
