use super::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    String,
    Number,
    Boolean,
    Date,
    Json,
}

impl DataType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Date => "date",
            Self::Json => "json",
        }
    }

    pub fn parse(s: &str) -> Result<Self, AppError> {
        match s.trim().to_lowercase().as_str() {
            "string" | "text" | "teks" => Ok(Self::String),
            "number" | "numeric" | "integer" | "decimal" | "angka" => Ok(Self::Number),
            "boolean" | "bool" | "ya_tidak" => Ok(Self::Boolean),
            "date" | "tanggal" => Ok(Self::Date),
            "json" | "structured" => Ok(Self::Json),
            _ => Err(AppError::validation(
                "dataType",
                format!(
                    "Tipe data '{s}' tidak didukung. Pilihan: string, number, boolean, date, json."
                ),
            )),
        }
    }
}

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
        match s.trim().to_lowercase().as_str() {
            "confirmed" | "terkonfirmasi" => Ok(Self::Confirmed),
            "unverified" | "belum_terverifikasi" => Ok(Self::Unverified),
            "disputed" | "diperdebatkan" => Ok(Self::Disputed),
            "outdated" | "kadaluarsa" => Ok(Self::Outdated),
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
        match s.trim().to_lowercase().as_str() {
            "ordinary" | "biasa" => Ok(Self::Ordinary),
            "sensitive" | "sensitif" => Ok(Self::Sensitive),
            "highly_sensitive" | "sangat_sensitif" => Ok(Self::HighlySensitive),
            _ => Err(AppError::validation(
                "sensitivity",
                "Klasifikasi sensitivitas tidak valid. Pilihan: ordinary, sensitive, highly_sensitive.",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    UserDeclaration,
    UploadedDocument,
    ManualEntry,
    ObservedAction,
    ExternalRecord,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserDeclaration => "user_declaration",
            Self::UploadedDocument => "uploaded_document",
            Self::ManualEntry => "manual_entry",
            Self::ObservedAction => "observed_action",
            Self::ExternalRecord => "external_record",
        }
    }

    pub fn parse(s: &str) -> Result<Self, AppError> {
        match s.trim().to_lowercase().as_str() {
            "user_declaration" | "pernyataan_pengguna" => Ok(Self::UserDeclaration),
            "uploaded_document" | "dokumen_unggahan" | "dokumen" => Ok(Self::UploadedDocument),
            "manual_entry" | "input_manual" | "user_input" => Ok(Self::ManualEntry),
            "observed_action" | "observasi" => Ok(Self::ObservedAction),
            "external_record" | "bukti_lain" => Ok(Self::ExternalRecord),
            _ => Ok(Self::ManualEntry),
        }
    }
}

/// Normalizes and parses raw user input based on the declared data type.
pub fn normalize_typed_value(
    data_type: DataType,
    raw_value: &str,
) -> Result<(serde_json::Value, String), AppError> {
    let trimmed = raw_value.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(
            "value",
            "Nilai fakta tidak boleh kosong.",
        ));
    }

    match data_type {
        DataType::String => {
            let normalized = trimmed.to_lowercase();
            Ok((serde_json::Value::String(trimmed.to_string()), normalized))
        }
        DataType::Number => {
            // Clean thousand separators (dots or commas) if plain number format
            // e.g. "15.000.000" or "15,000,000" or "28" or "3.14"
            let sanitized = if trimmed.contains('.') && !trimmed.contains(',') {
                // Check if it's decimal or thousand separator:
                let dot_count = trimmed.matches('.').count();
                if dot_count > 1 {
                    trimmed.replace('.', "")
                } else {
                    // Single dot: could be decimal like 3.14 or thousands like 1.500
                    // If 3 digits after dot at end, check length; or standard float parse:
                    trimmed.to_string()
                }
            } else if trimmed.contains(',') && !trimmed.contains('.') {
                let comma_count = trimmed.matches(',').count();
                if comma_count > 1 {
                    trimmed.replace(',', "")
                } else {
                    trimmed.replace(',', ".")
                }
            } else if trimmed.contains('.') && trimmed.contains(',') {
                // e.g. 15.000.000,50 -> Indonesian format
                trimmed.replace('.', "").replace(',', ".")
            } else {
                trimmed.to_string()
            };

            if let Ok(int_val) = sanitized.parse::<i64>() {
                Ok((
                    serde_json::Value::Number(serde_json::Number::from(int_val)),
                    int_val.to_string(),
                ))
            } else if let Ok(float_val) = sanitized.parse::<f64>() {
                if let Some(num) = serde_json::Number::from_f64(float_val) {
                    Ok((serde_json::Value::Number(num), float_val.to_string()))
                } else {
                    Err(AppError::validation(
                        "value",
                        "Nilai angka tidak valid (finite floating point number required).",
                    ))
                }
            } else {
                Err(AppError::validation(
                    "value",
                    format!("Format angka tidak valid: '{trimmed}'."),
                ))
            }
        }
        DataType::Boolean => {
            match trimmed.to_lowercase().as_str() {
                "true" | "ya" | "yes" | "1" | "benar" => {
                    Ok((serde_json::Value::Bool(true), "true".to_string()))
                }
                "false" | "tidak" | "no" | "0" | "salah" => {
                    Ok((serde_json::Value::Bool(false), "false".to_string()))
                }
                _ => Err(AppError::validation(
                    "value",
                    format!("Format boolean tidak valid: '{trimmed}'. Pilihan: true, false, ya, tidak, 1, 0."),
                )),
            }
        }
        DataType::Date => {
            // Check ISO date format YYYY-MM-DD
            if trimmed.len() >= 10 {
                let date_part = &trimmed[0..10];
                let parts: Vec<&str> = date_part.split('-').collect();
                if parts.len() == 3
                    && parts[0].len() == 4
                    && parts[1].len() == 2
                    && parts[2].len() == 2
                    && parts[0].chars().all(|c| c.is_ascii_digit())
                    && parts[1].chars().all(|c| c.is_ascii_digit())
                    && parts[2].chars().all(|c| c.is_ascii_digit())
                {
                    let y: u32 = parts[0].parse().unwrap_or(0);
                    let m: u32 = parts[1].parse().unwrap_or(0);
                    let d: u32 = parts[2].parse().unwrap_or(0);
                    if (1900..=2200).contains(&y) && (1..=12).contains(&m) && (1..=31).contains(&d)
                    {
                        return Ok((
                            serde_json::Value::String(trimmed.to_string()),
                            date_part.to_string(),
                        ));
                    }
                }
            }
            Err(AppError::validation(
                "value",
                format!("Format tanggal tidak valid: '{trimmed}'. Gunakan format YYYY-MM-DD."),
            ))
        }
        DataType::Json => {
            let parsed: serde_json::Value = serde_json::from_str(trimmed).map_err(|e| {
                AppError::validation("value", format!("Format JSON tidak valid: {e}."))
            })?;
            let minified = serde_json::to_string(&parsed).unwrap_or_else(|_| trimmed.to_string());
            Ok((parsed, minified))
        }
    }
}

/// Derives a slugified fact_key from attribute name if not explicitly specified.
pub fn derive_fact_key(attribute: &str) -> String {
    let mut key = String::new();
    let mut prev_underscore = false;

    for ch in attribute.trim().chars() {
        if ch.is_alphanumeric() {
            key.push(ch.to_ascii_lowercase());
            prev_underscore = false;
        } else if !prev_underscore {
            key.push('_');
            prev_underscore = true;
        }
    }

    let trimmed_key = key.trim_matches('_');
    if trimmed_key.is_empty() {
        "fact".to_string()
    } else {
        trimmed_key.to_string()
    }
}

/// Metadata sumber bukti fakta
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FactSource {
    pub id: String,
    pub persona_id: String,
    pub fact_id: String,
    pub source_type: String,
    pub title: String,
    pub description: String,
    pub reference_uri: Option<String>,
    pub observed_at: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
}

/// Riwayat perubahan fakta untuk audit trail
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FactHistoryEntry {
    pub id: String,
    pub persona_id: String,
    pub fact_id: String,
    pub previous_value_json: Option<String>,
    pub new_value_json: String,
    pub previous_status: Option<String>,
    pub new_status: String,
    pub change_reason: Option<String>,
    pub source_id: Option<String>,
    pub changed_at: String,
}

/// Ringkasan fakta profil untuk list view
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FactSummary {
    pub id: String,
    pub persona_id: String,
    pub category: String,
    pub fact_key: String,
    pub attribute: String, // display name
    pub data_type: String,
    pub value: serde_json::Value,
    pub normalized_value: String,
    pub unit: Option<String>,
    pub verification_status: String,
    pub sensitivity: String,
    pub source_description: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub is_valid: bool,
    pub has_conflict: bool,
    pub sources_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
}

/// Detail lengkap fakta beserta seluruh sumber dan riwayat perubahannya
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileFactDetail {
    pub id: String,
    pub persona_id: String,
    pub category: String,
    pub fact_key: String,
    pub attribute: String,
    pub data_type: String,
    pub value: serde_json::Value,
    pub normalized_value: String,
    pub unit: Option<String>,
    pub verification_status: String,
    pub sensitivity: String,
    pub source_date: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub notes: Option<String>,
    pub is_valid: bool,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
    pub sources: Vec<FactSource>,
    pub history: Vec<FactHistoryEntry>,
}

/// Peringatan konflik fakta duplikat/bertentangan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConflictWarning {
    pub fact_key: String,
    pub category: String,
    pub existing_fact_id: String,
    pub existing_display_name: String,
    pub existing_value: String,
    pub existing_status: String,
    pub conflicting_fact_id: String,
    pub conflicting_display_name: String,
    pub conflicting_value: String,
    pub conflicting_status: String,
    pub message: String,
}

/// Input pembuatan fakta baru
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFactInput {
    pub persona_id: String,
    pub category: String,
    pub fact_key: Option<String>,
    pub attribute: String,
    pub data_type: Option<String>,
    pub value: String,
    pub unit: Option<String>,
    pub source_type: Option<String>,
    pub source_title: Option<String>,
    pub source_description: String,
    pub reference_uri: Option<String>,
    pub observed_at: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub notes: Option<String>,
    pub verification_status: Option<String>,
    pub sensitivity: Option<String>,
}

impl CreateFactInput {
    pub fn new(
        persona_id: impl Into<String>,
        category: impl Into<String>,
        attribute: impl Into<String>,
        value: impl Into<String>,
        source_description: impl Into<String>,
    ) -> Self {
        Self {
            persona_id: persona_id.into(),
            category: category.into(),
            fact_key: None,
            attribute: attribute.into(),
            data_type: None,
            value: value.into(),
            unit: None,
            source_type: None,
            source_title: None,
            source_description: source_description.into(),
            reference_uri: None,
            observed_at: None,
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: None,
            sensitivity: None,
        }
    }

    pub fn validate(&self) -> Result<ValidatedFactInput, AppError> {
        let persona_id = self.persona_id.trim();
        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "ID profil wajib diisi."));
        }

        let category = self.category.trim();
        let attribute = self.attribute.trim();
        let raw_value = self.value.trim();
        let source_desc = self.source_description.trim();

        if category.is_empty() || attribute.is_empty() || raw_value.is_empty() {
            return Err(AppError::validation(
                "general",
                "Kategori, nama informasi, dan nilai wajib diisi.",
            ));
        }

        if source_desc.is_empty() {
            return Err(AppError::validation(
                "sourceDescription",
                "Keterangan sumber bukti wajib diisi.",
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
        if raw_value.chars().count() > 4000 {
            return Err(AppError::validation(
                "value",
                "Nilai informasi maksimal 4000 karakter.",
            ));
        }
        if source_desc.chars().count() > 500 {
            return Err(AppError::validation(
                "sourceDescription",
                "Deskripsi sumber maksimal 500 karakter.",
            ));
        }

        let data_type = match self.data_type.as_deref() {
            Some(dt) => DataType::parse(dt)?,
            None => DataType::String,
        };

        let (typed_value, normalized_value) = normalize_typed_value(data_type, raw_value)?;

        let fact_key = match self.fact_key.as_deref() {
            Some(k) if !k.trim().is_empty() => derive_fact_key(k),
            _ => derive_fact_key(attribute),
        };

        let source_type = match self.source_type.as_deref() {
            Some(st) => SourceType::parse(st)?,
            None => SourceType::ManualEntry,
        };

        let verification_status = match self.verification_status.as_deref() {
            Some(vs) => VerificationStatus::parse(vs)?,
            // ATURAN SC-011: Fakta baru otomatis berstatus unverified
            None => VerificationStatus::Unverified,
        };

        let sensitivity = match self.sensitivity.as_deref() {
            Some(s) => Sensitivity::parse(s)?,
            None => Sensitivity::Ordinary,
        };

        let source_title = self
            .source_title
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(source_desc);

        Ok(ValidatedFactInput {
            persona_id: persona_id.to_string(),
            category: category.to_string(),
            fact_key,
            attribute: attribute.to_string(),
            data_type,
            value: typed_value,
            normalized_value,
            unit: self
                .unit
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from),
            source_type,
            source_title: source_title.to_string(),
            source_description: source_desc.to_string(),
            reference_uri: self
                .reference_uri
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from),
            observed_at: self
                .observed_at
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from),
            valid_from: self
                .valid_from
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from),
            valid_until: self
                .valid_until
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from),
            notes: self
                .notes
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from),
            verification_status,
            sensitivity,
        })
    }
}

pub struct ValidatedFactInput {
    pub persona_id: String,
    pub category: String,
    pub fact_key: String,
    pub attribute: String,
    pub data_type: DataType,
    pub value: serde_json::Value,
    pub normalized_value: String,
    pub unit: Option<String>,
    pub source_type: SourceType,
    pub source_title: String,
    pub source_description: String,
    pub reference_uri: Option<String>,
    pub observed_at: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub notes: Option<String>,
    pub verification_status: VerificationStatus,
    pub sensitivity: Sensitivity,
}

/// Input pembaruan fakta
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFactInput {
    pub persona_id: String,
    pub fact_id: String,
    pub category: Option<String>,
    pub attribute: Option<String>,
    pub data_type: Option<String>,
    pub value: Option<String>,
    pub unit: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub notes: Option<String>,
    pub change_reason: Option<String>,
    pub verification_status: Option<String>,
    pub sensitivity: Option<String>,
}

/// Input penambahan sumber bukti baru ke fakta yang sudah ada
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddFactSourceInput {
    pub persona_id: String,
    pub fact_id: String,
    pub source_type: String,
    pub title: String,
    pub description: String,
    pub reference_uri: Option<String>,
    pub observed_at: Option<String>,
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
    pub notes: Option<String>,
}

/// Memeriksa apakah rentang validitas masih aktif terhadap waktu sekarang
pub fn check_validity_period(
    valid_from: Option<&str>,
    valid_until: Option<&str>,
    now_iso: &str,
) -> bool {
    let now_date = if now_iso.len() >= 10 {
        &now_iso[0..10]
    } else {
        now_iso
    };

    if let Some(from) = valid_from {
        let from_date = if from.len() >= 10 { &from[0..10] } else { from };
        if now_date < from_date {
            return false;
        }
    }

    if let Some(until) = valid_until {
        let until_date = if until.len() >= 10 {
            &until[0..10]
        } else {
            until
        };
        if now_date > until_date {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_value_normalization_string() {
        let (val, norm) = normalize_typed_value(DataType::String, "  Software Engineer  ").unwrap();
        assert_eq!(
            val,
            serde_json::Value::String("Software Engineer".to_string())
        );
        assert_eq!(norm, "software engineer");
    }

    #[test]
    fn test_typed_value_normalization_number() {
        let (val, norm) = normalize_typed_value(DataType::Number, "28").unwrap();
        assert_eq!(val, serde_json::json!(28));
        assert_eq!(norm, "28");

        let (val_curr, norm_curr) = normalize_typed_value(DataType::Number, "15.000.000").unwrap();
        assert_eq!(val_curr, serde_json::json!(15000000));
        assert_eq!(norm_curr, "15000000");

        let (val_dec, norm_dec) = normalize_typed_value(DataType::Number, "3.75").unwrap();
        assert_eq!(val_dec, serde_json::json!(3.75));
        assert_eq!(norm_dec, "3.75");
    }

    #[test]
    fn test_typed_value_normalization_boolean() {
        let (v_true, n_true) = normalize_typed_value(DataType::Boolean, "ya").unwrap();
        assert_eq!(v_true, serde_json::json!(true));
        assert_eq!(n_true, "true");

        let (v_false, n_false) = normalize_typed_value(DataType::Boolean, "tidak").unwrap();
        assert_eq!(v_false, serde_json::json!(false));
        assert_eq!(n_false, "false");
    }

    #[test]
    fn test_typed_value_normalization_date() {
        let (v_date, n_date) = normalize_typed_value(DataType::Date, "1996-08-17").unwrap();
        assert_eq!(v_date, serde_json::json!("1996-08-17"));
        assert_eq!(n_date, "1996-08-17");

        assert!(normalize_typed_value(DataType::Date, "invalid-date").is_err());
    }

    #[test]
    fn test_typed_value_normalization_json() {
        let raw = r#"{"brands": ["Toyota", "Honda"], "primary": "Honda"}"#;
        let (v_json, _) = normalize_typed_value(DataType::Json, raw).unwrap();
        assert_eq!(v_json["primary"], "Honda");
    }

    #[test]
    fn test_derive_fact_key() {
        assert_eq!(
            derive_fact_key("Penghasilan Bulanan"),
            "penghasilan_bulanan"
        );
        assert_eq!(derive_fact_key("Pekerjaan / Jabatan"), "pekerjaan_jabatan");
        assert_eq!(derive_fact_key("owns_car"), "owns_car");
    }

    #[test]
    fn test_validity_period_checker() {
        let now = "2026-11-23T00:00:00Z";
        assert!(check_validity_period(
            Some("2020-01-01"),
            Some("2030-01-01"),
            now
        ));
        assert!(!check_validity_period(
            Some("2020-01-01"),
            Some("2025-01-01"),
            now
        ));
        assert!(!check_validity_period(
            Some("2027-01-01"),
            Some("2030-01-01"),
            now
        ));
        assert!(check_validity_period(None, None, now));
    }
}
