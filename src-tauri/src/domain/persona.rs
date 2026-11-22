use super::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersonaSummary {
    pub id: String,
    pub display_name: String,
    pub is_active: bool,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePersonaInput {
    pub display_name: String,
}

impl CreatePersonaInput {
    pub fn new(display_name: impl Into<String>) -> Self {
        Self {
            display_name: display_name.into(),
        }
    }

    pub fn validate(&self) -> Result<String, AppError> {
        let trimmed = self.display_name.trim();
        let length = trimmed.chars().count();

        if length == 0 || length > 80 {
            return Err(AppError::validation(
                "displayName",
                "Nama profil harus berisi 1 sampai 80 karakter.",
            ));
        }

        Ok(trimmed.to_string())
    }
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
    fn accepts_valid_display_name() {
        let input = CreatePersonaInput::new("  Responden Utama  ");
        let result = input.validate();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Responden Utama");
    }
}
