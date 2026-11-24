use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Pengaturan Privasi & Diagnostik Aplikasi (SC-060)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PrivacySettings {
    /// Sensor otomatis seluruh log dari kredensial, kunci API, dan data pribadi (default: true)
    pub auto_redact_logs: bool,
    /// Wajibkan persetujuan eksplisit per-permintaan untuk fakta profil berkategori sensitif
    pub require_sensitive_consent_prompt: bool,
    /// Wajibkan persetujuan eksplisit per-permintaan untuk pengiriman gambar ke model visual eksternal
    pub require_vision_consent_prompt: bool,
    /// Kebijakan retensi tangkapan layar sementara: "immediate_delete" | "retain_1_hour" | "retain_24_hours" | "retain_manual_only"
    pub screenshot_retention_policy: String,
    /// Otomatis membersihkan folder temp_captures saat aplikasi ditutup
    pub auto_cleanup_temp_on_exit: bool,
    /// Level logging diagnostik: "INFO" | "WARN" | "ERROR" | "DEBUG"
    pub log_level: String,
    /// Timestamp pembaruan terakhir (ISO 8601 UTC)
    pub updated_at: String,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            auto_redact_logs: true,
            require_sensitive_consent_prompt: true,
            require_vision_consent_prompt: true,
            screenshot_retention_policy: "immediate_delete".to_string(),
            auto_cleanup_temp_on_exit: true,
            log_level: "INFO".to_string(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

/// Input pembaruan pengaturan privasi
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePrivacySettingsInput {
    pub auto_redact_logs: Option<bool>,
    pub require_sensitive_consent_prompt: Option<bool>,
    pub require_vision_consent_prompt: Option<bool>,
    pub screenshot_retention_policy: Option<String>,
    pub auto_cleanup_temp_on_exit: Option<bool>,
    pub log_level: Option<String>,
}

/// Rincian pengungkapan fakta profil untuk transmisi data keluar
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OutboundFactDisclosure {
    pub fact_id: String,
    pub category: String,
    pub attribute: String,
    pub value_preview: String,
    pub sensitivity: String, // "ordinary", "sensitive", "highly_sensitive"
    pub status: String, // "included", "excluded_no_consent", "excluded_archived", "excluded_unverified"
    pub is_sensitive: bool,
    pub exclusion_reason: Option<String>,
}

/// Transparansi Data Transmisi Keluar (What Data Leaves The Device)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OutboundDataDisclosure {
    pub provider_type: String,
    pub provider_name: String,
    pub destination_endpoint: String,
    pub model_id: String,
    pub question_text: String,
    pub instructions: Option<String>,
    pub options: Option<Vec<String>>,
    pub included_facts: Vec<OutboundFactDisclosure>,
    pub excluded_facts: Vec<OutboundFactDisclosure>,
    pub includes_sensitive_data: bool,
    pub sensitive_consent_granted: bool,
    pub includes_external_vision: bool,
    pub vision_consent_granted: bool,
    pub not_transmitted_guarantees: Vec<String>,
    pub generated_at: String,
}

/// Input inspeksi data keluar
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectOutboundDataInput {
    pub persona_id: String,
    pub question_text: String,
    pub instructions: Option<String>,
    pub options: Option<Vec<String>>,
    pub allow_sensitive: Option<bool>,
    pub allow_highly_sensitive: Option<bool>,
    pub allow_external_vision: Option<bool>,
    pub has_image_attachment: Option<bool>,
}

/// Item log diagnostik yang selalu teredaksi
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsLogEntry {
    pub id: String,
    pub timestamp: String,
    pub level: String,    // "INFO", "WARN", "ERROR", "DEBUG"
    pub category: String, // "AI_PROVIDER", "SCREEN_CAPTURE", "DATA_RETENTION", "SECURITY", "SYSTEM"
    pub message: String,
    pub metadata_json: Option<String>,
}

/// Input pencatatan log
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordLogInput {
    pub level: String,
    pub category: String,
    pub message: String,
    pub metadata_json: Option<String>,
}

/// Informasi diagnostik sistem aplikasi
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SystemDiagnosticsInfo {
    pub app_version: String,
    pub os_platform: String,
    pub db_schema_version: i64,
    pub sqlite_version: String,
    pub wal_size_bytes: i64,
    pub total_db_size_bytes: i64,
    pub credential_store_type: String,
    pub temp_captures_count: i64,
    pub temp_captures_size_bytes: i64,
    pub active_personas_count: i64,
    pub total_facts_count: i64,
    pub total_sessions_count: i64,
    pub provider_configs_count: i64,
    pub generated_at: String,
}

/// Status retensi berkas tangkapan layar dan cache
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RetentionStatus {
    pub temp_directory_path: String,
    pub temp_file_count: i64,
    pub temp_total_bytes: i64,
    pub oldest_file_timestamp: Option<String>,
    pub active_retention_policy: String,
    pub auto_cleanup_on_exit: bool,
}

/// Hasil eksekusi pembersihan file sementara
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RetentionCleanupResult {
    pub deleted_files_count: i64,
    pub freed_bytes: i64,
    pub cleaned_at: String,
    pub message: String,
}

/// Input pembersihan manual
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanupTempFilesInput {
    /// Usia maksimal dalam detik (jika None atau Some(0), bersihkan semua file)
    pub max_age_seconds: Option<i64>,
}

/// Mesin Sensor Redaksi Otomatis (Automatic Log and Secret Redactor)
pub struct LogRedactor;

impl LogRedactor {
    /// Sensor string teks dari seluruh API key, token, kredensial, dan data PII
    pub fn redact_text(input: &str) -> String {
        let mut text = input.to_string();

        // 1. Redact Google API keys: "AIza" followed by 35 alphanumeric or -_ chars
        text = Self::replace_pattern(&text, "AIza", 35, "[REDACTED_API_KEY]");

        // 2. Redact generic sk- keys
        text = Self::replace_prefixed_token(&text, "sk-", 20, "[REDACTED_API_KEY]");

        // 3. Redact Bearer tokens: "Bearer <token>"
        text = Self::replace_bearer_token(&text);

        // 4. Redact query param keys: "key=<long_token>"
        text = Self::replace_key_param(&text);

        // 5. Redact HTTP Headers: Authorization, x-goog-api-key
        text = Self::replace_header_value(&text, "authorization", "[REDACTED_AUTH_HEADER]");
        text = Self::replace_header_value(&text, "x-goog-api-key", "[REDACTED_API_KEY]");

        // 6. Redact password fields in strings: password: "..." or password=...
        text = Self::replace_password_field(&text);

        // 7. Redact Indonesian NIK (16 consecutive digits)
        text = Self::replace_16_digit_nik(&text);

        // 8. Redact Emails
        text = Self::replace_emails(&text);

        // 9. Redact Indonesian phone numbers (+62..., 62..., 08...)
        text = Self::replace_phone_numbers(&text);

        // 10. Redact Credit Card / Debit Card numbers (13-19 digits, possibly spaced/hyphenated)
        text = Self::replace_credit_cards(&text);

        text
    }

    /// Sensor struktur JSON secara rekursif
    pub fn redact_json(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::String(s) => serde_json::Value::String(Self::redact_text(s)),
            serde_json::Value::Array(arr) => {
                let redacted_arr = arr.iter().map(Self::redact_json).collect();
                serde_json::Value::Array(redacted_arr)
            }
            serde_json::Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    let k_lower = k.to_lowercase();
                    if k_lower.contains("secret")
                        || k_lower.contains("apikey")
                        || k_lower.contains("api_key")
                        || k_lower.contains("password")
                        || k_lower.contains("credential")
                        || k_lower.contains("token")
                        || k_lower == "authorization"
                    {
                        new_map.insert(
                            k.clone(),
                            serde_json::Value::String("[REDACTED_SECRET]".to_string()),
                        );
                    } else {
                        new_map.insert(k.clone(), Self::redact_json(v));
                    }
                }
                serde_json::Value::Object(new_map)
            }
            other => other.clone(),
        }
    }

    fn replace_pattern(text: &str, prefix: &str, length: usize, replacement: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut idx = 0;
        let bytes = text.as_bytes();
        let prefix_bytes = prefix.as_bytes();

        while idx < bytes.len() {
            if idx + prefix_bytes.len() + length <= bytes.len()
                && &bytes[idx..idx + prefix_bytes.len()] == prefix_bytes
            {
                // Check if followed by length valid token characters
                let token_slice =
                    &bytes[idx + prefix_bytes.len()..idx + prefix_bytes.len() + length];
                let is_valid = token_slice
                    .iter()
                    .all(|b| b.is_ascii_alphanumeric() || *b == b'-' || *b == b'_');
                if is_valid {
                    result.push_str(replacement);
                    idx += prefix_bytes.len() + length;
                    continue;
                }
            }
            result.push(bytes[idx] as char);
            idx += 1;
        }

        result
    }

    fn replace_prefixed_token(
        text: &str,
        prefix: &str,
        min_len: usize,
        replacement: &str,
    ) -> String {
        let mut result = String::with_capacity(text.len());
        let mut idx = 0;
        let bytes = text.as_bytes();
        let prefix_bytes = prefix.as_bytes();

        while idx < bytes.len() {
            if idx + prefix_bytes.len() <= bytes.len()
                && &bytes[idx..idx + prefix_bytes.len()] == prefix_bytes
            {
                // Find token end
                let mut end = idx + prefix_bytes.len();
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric()
                        || bytes[end] == b'-'
                        || bytes[end] == b'_')
                {
                    end += 1;
                }
                let token_len = end - (idx + prefix_bytes.len());
                if token_len >= min_len {
                    result.push_str(replacement);
                    idx = end;
                    continue;
                }
            }
            result.push(bytes[idx] as char);
            idx += 1;
        }

        result
    }

    fn replace_bearer_token(text: &str) -> String {
        let lower = text.to_lowercase();
        let mut result = String::with_capacity(text.len());
        let mut idx = 0;
        let bytes = text.as_bytes();

        while idx < bytes.len() {
            if idx + 7 <= bytes.len() && &lower[idx..idx + 7] == "bearer " {
                result.push_str("Bearer [REDACTED_TOKEN]");
                idx += 7;
                // Skip until whitespace, comma, or line end
                while idx < bytes.len()
                    && !bytes[idx].is_ascii_whitespace()
                    && bytes[idx] != b','
                    && bytes[idx] != b';'
                    && bytes[idx] != b'"'
                    && bytes[idx] != b'\''
                {
                    idx += 1;
                }
                continue;
            }
            result.push(bytes[idx] as char);
            idx += 1;
        }

        result
    }

    fn replace_key_param(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut idx = 0;
        let bytes = text.as_bytes();

        while idx < bytes.len() {
            if (idx == 0
                || bytes[idx - 1] == b'?'
                || bytes[idx - 1] == b'&'
                || bytes[idx - 1].is_ascii_whitespace())
                && idx + 4 <= bytes.len()
                && &bytes[idx..idx + 4] == b"key="
            {
                let start_val = idx + 4;
                let mut end_val = start_val;
                while end_val < bytes.len()
                    && (bytes[end_val].is_ascii_alphanumeric()
                        || bytes[end_val] == b'-'
                        || bytes[end_val] == b'_')
                {
                    end_val += 1;
                }
                if end_val - start_val >= 16 {
                    result.push_str("key=[REDACTED_KEY]");
                    idx = end_val;
                    continue;
                }
            }
            result.push(bytes[idx] as char);
            idx += 1;
        }

        result
    }

    fn replace_header_value(text: &str, header_name: &str, replacement: &str) -> String {
        let lower = text.to_lowercase();
        let target = format!("{}:", header_name.to_lowercase());
        let mut result = String::with_capacity(text.len());
        let mut idx = 0;
        let bytes = text.as_bytes();

        while idx < bytes.len() {
            if idx + target.len() <= bytes.len() && &lower[idx..idx + target.len()] == target {
                result.push_str(&text[idx..idx + target.len()]);
                result.push(' ');
                result.push_str(replacement);
                idx += target.len();
                // Consume value until newline or delimiter
                while idx < bytes.len()
                    && bytes[idx] != b'\n'
                    && bytes[idx] != b'\r'
                    && bytes[idx] != b','
                {
                    idx += 1;
                }
                continue;
            }
            result.push(bytes[idx] as char);
            idx += 1;
        }

        result
    }

    fn replace_password_field(text: &str) -> String {
        let lower = text.to_lowercase();
        let mut result = String::with_capacity(text.len());
        let mut idx = 0;
        let bytes = text.as_bytes();

        while idx < bytes.len() {
            if idx + 8 <= bytes.len() && &lower[idx..idx + 8] == "password" {
                let mut cur = idx + 8;
                // Skip whitespace, quotes, colons, equals
                while cur < bytes.len()
                    && (bytes[cur].is_ascii_whitespace()
                        || bytes[cur] == b':'
                        || bytes[cur] == b'='
                        || bytes[cur] == b'"'
                        || bytes[cur] == b'\'')
                {
                    cur += 1;
                }
                // Token starts here
                let val_start = cur;
                while cur < bytes.len()
                    && !bytes[cur].is_ascii_whitespace()
                    && bytes[cur] != b','
                    && bytes[cur] != b'"'
                    && bytes[cur] != b'\''
                    && bytes[cur] != b';'
                {
                    cur += 1;
                }
                if cur > val_start {
                    result.push_str(&text[idx..val_start]);
                    result.push_str("[REDACTED_PASSWORD]");
                    idx = cur;
                    continue;
                }
            }
            result.push(bytes[idx] as char);
            idx += 1;
        }

        result
    }

    fn replace_16_digit_nik(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let chars: Vec<char> = text.chars().collect();
        let mut idx = 0;

        while idx < chars.len() {
            if chars[idx].is_ascii_digit() {
                let start = idx;
                while idx < chars.len() && chars[idx].is_ascii_digit() {
                    idx += 1;
                }
                let len = idx - start;
                // Indonesian NIK is strictly 16 digits
                let is_isolated = (start == 0 || !chars[start - 1].is_ascii_alphanumeric())
                    && (idx >= chars.len() || !chars[idx].is_ascii_alphanumeric());
                if len == 16 && is_isolated {
                    result.push_str("[REDACTED_NIK_16_DIGITS]");
                } else {
                    for c in &chars[start..idx] {
                        result.push(*c);
                    }
                }
            } else {
                result.push(chars[idx]);
                idx += 1;
            }
        }

        result
    }

    fn replace_emails(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let chars: Vec<char> = text.chars().collect();
        let mut idx = 0;

        while idx < chars.len() {
            if chars[idx] == '@' && idx > 0 && idx + 1 < chars.len() {
                // Find start of local-part
                let mut start = idx;
                while start > 0
                    && (chars[start - 1].is_ascii_alphanumeric()
                        || chars[start - 1] == '.'
                        || chars[start - 1] == '_'
                        || chars[start - 1] == '+'
                        || chars[start - 1] == '-')
                {
                    start -= 1;
                }

                // Find end of domain
                let mut end = idx + 1;
                let mut dot_seen = false;
                while end < chars.len()
                    && (chars[end].is_ascii_alphanumeric()
                        || chars[end] == '.'
                        || chars[end] == '-')
                {
                    if chars[end] == '.' {
                        dot_seen = true;
                    }
                    end += 1;
                }

                // If valid email structure
                if start < idx && end > idx + 1 && dot_seen && chars[end - 1] != '.' {
                    // Truncate what was previously added from 'start'
                    let prefix_len_chars = idx - start;
                    for _ in 0..prefix_len_chars {
                        result.pop();
                    }
                    result.push_str("[REDACTED_EMAIL]");
                    idx = end;
                    continue;
                }
            }
            result.push(chars[idx]);
            idx += 1;
        }

        result
    }

    fn replace_phone_numbers(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let chars: Vec<char> = text.chars().collect();
        let mut idx = 0;

        while idx < chars.len() {
            let is_prefix = (idx + 3 <= chars.len()
                && chars[idx] == '+'
                && chars[idx + 1] == '6'
                && chars[idx + 2] == '2')
                || (idx + 2 <= chars.len() && chars[idx] == '6' && chars[idx + 1] == '2')
                || (idx + 2 <= chars.len() && chars[idx] == '0' && chars[idx + 1] == '8');

            let word_boundary_before = idx == 0 || !chars[idx - 1].is_ascii_alphanumeric();

            if is_prefix && word_boundary_before {
                let start = idx;
                let mut end = start;
                while end < chars.len()
                    && (chars[end].is_ascii_digit()
                        || chars[end] == '-'
                        || chars[end] == ' '
                        || chars[end] == '+')
                {
                    end += 1;
                }
                let digit_count = chars[start..end]
                    .iter()
                    .filter(|c| c.is_ascii_digit())
                    .count();
                if (10..=15).contains(&digit_count) {
                    result.push_str("[REDACTED_PHONE]");
                    idx = end;
                    continue;
                }
            }
            result.push(chars[idx]);
            idx += 1;
        }

        result
    }

    fn replace_credit_cards(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let chars: Vec<char> = text.chars().collect();
        let mut idx = 0;

        while idx < chars.len() {
            // Check for 4 blocks of 4 digits or 16 continuous digits (if not already redacted by NIK)
            let word_boundary_before = idx == 0 || !chars[idx - 1].is_ascii_alphanumeric();
            if word_boundary_before && chars[idx].is_ascii_digit() {
                let start = idx;
                let mut end = start;
                while end < chars.len()
                    && (chars[end].is_ascii_digit() || chars[end] == '-' || chars[end] == ' ')
                {
                    end += 1;
                }
                let digits: Vec<char> = chars[start..end]
                    .iter()
                    .copied()
                    .filter(|c| c.is_ascii_digit())
                    .collect();
                if digits.len() == 16
                    && (chars[start..end].contains(&'-') || chars[start..end].contains(&' '))
                {
                    result.push_str("[REDACTED_PAYMENT_CARD]");
                    idx = end;
                    continue;
                }
            }
            result.push(chars[idx]);
            idx += 1;
        }

        result
    }
}
