use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

impl AppError {
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: "VALIDATION_ERROR".to_string(),
            message: message.into(),
            field: Some(field.into()),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            code: "NOT_FOUND".to_string(),
            message: message.into(),
            field: None,
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            code: "CONFLICT_ERROR".to_string(),
            message: message.into(),
            field: None,
        }
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self {
            code: "DATABASE_ERROR".to_string(),
            message: message.into(),
            field: None,
        }
    }

    pub fn migration(message: impl Into<String>) -> Self {
        Self {
            code: "MIGRATION_ERROR".to_string(),
            message: message.into(),
            field: None,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
            field: None,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref field) = self.field {
            write!(f, "[{}] {}: {}", self.code, field, self.message)
        } else {
            write!(f, "[{}] {}", self.code, self.message)
        }
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        Self::database(format!("Kesalahan basis data lokal: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_error_formats_properly() {
        let err = AppError::validation("displayName", "Nama wajib diisi.");
        assert_eq!(err.code, "VALIDATION_ERROR");
        assert_eq!(err.field.as_deref(), Some("displayName"));
        assert_eq!(
            err.to_string(),
            "[VALIDATION_ERROR] displayName: Nama wajib diisi."
        );

        let not_found = AppError::not_found("Profil tidak ditemukan.");
        assert_eq!(not_found.code, "NOT_FOUND");
        assert_eq!(not_found.field, None);
        assert_eq!(not_found.to_string(), "[NOT_FOUND] Profil tidak ditemukan.");
    }

    #[test]
    fn app_error_serializes_to_camel_case() {
        let err = AppError::validation("testField", "Error message");
        let json = serde_json::to_string(&err).expect("serialize should succeed");
        assert!(json.contains("\"code\":\"VALIDATION_ERROR\""));
        assert!(json.contains("\"field\":\"testField\""));
        assert!(json.contains("\"message\":\"Error message\""));
    }
}
