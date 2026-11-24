use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use survey_copilot_lib::db::Database;
use survey_copilot_lib::domain::evidence::SensitivityConsent;
use survey_copilot_lib::domain::fact::CreateFactInput;
use survey_copilot_lib::domain::persona::CreatePersonaInput;
use survey_copilot_lib::domain::privacy::{
    CleanupTempFilesInput, InspectOutboundDataInput, LogRedactor, RecordLogInput,
    UpdatePrivacySettingsInput,
};
use survey_copilot_lib::services::{FactService, PersonaService, PrivacyService};

fn setup_memory_db() -> Database {
    Database::initialize(Path::new(":memory:")).expect("memory database should initialize")
}

#[test]
fn test_log_redactor_redacts_gemini_api_key() {
    let input = "Request failed: AIzaSyD1234567890abcdefghijklmnopqrstuv on https://generativelanguage.googleapis.com";
    let redacted = LogRedactor::redact_text(input);

    assert!(!redacted.contains("AIzaSyD1234567890abcdefghijklmnopqrstuv"));
    assert!(redacted.contains("[REDACTED_API_KEY]"));
}

#[test]
fn test_log_redactor_redacts_generic_sk_and_bearer_tokens() {
    let input = "Payload contains token: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9 and sk-1234567890abcdef1234567890 and header Authorization: secret_pass";
    let redacted = LogRedactor::redact_text(input);

    assert!(!redacted.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
    assert!(!redacted.contains("sk-1234567890abcdef1234567890"));
    assert!(!redacted.contains("secret_pass"));
    assert!(redacted.contains("Bearer [REDACTED_TOKEN]"));
    assert!(redacted.contains("[REDACTED_API_KEY]"));
    assert!(redacted.contains("[REDACTED_AUTH_HEADER]"));
}

#[test]
fn test_log_redactor_redacts_indonesian_nik_16_digits() {
    let input = "Data responden memiliki NIK 3171012304950001 dan tahun lahir 1995 dengan saldo 5000000";
    let redacted = LogRedactor::redact_text(input);

    assert!(!redacted.contains("3171012304950001"));
    assert!(redacted.contains("[REDACTED_NIK_16_DIGITS]"));
    // Angka 4 digit tahun lahir dan saldo tidak boleh terganggu
    assert!(redacted.contains("1995"));
    assert!(redacted.contains("5000000"));
}

#[test]
fn test_log_redactor_redacts_email_and_phone() {
    let input = "Kontak person: respondent.utama@gmail.co.id atau nomor telepon +6281234567890 atau 081987654321";
    let redacted = LogRedactor::redact_text(input);

    assert!(!redacted.contains("respondent.utama@gmail.co.id"));
    assert!(!redacted.contains("+6281234567890"));
    assert!(!redacted.contains("081987654321"));
    assert!(redacted.contains("[REDACTED_EMAIL]"));
    assert!(redacted.contains("[REDACTED_PHONE]"));
}

#[test]
fn test_log_redactor_redacts_credit_card_numbers() {
    let input = "Kartu kredit nasabah: 4111-2222-3333-4444 dalam pembayaran langganan";
    let redacted = LogRedactor::redact_text(input);

    assert!(!redacted.contains("4111-2222-3333-4444"));
    assert!(redacted.contains("[REDACTED_PAYMENT_CARD]"));
}

#[test]
fn test_log_redactor_redacts_json_secrets() {
    let json_val = serde_json::json!({
        "provider": "gemini",
        "apiKey": "AIzaSyTestKey1234567890123456789012345",
        "secret": "super_secret_token",
        "password": "my_password_123",
        "data": {
            "email": "user@test.com",
            "normalField": "public value"
        }
    });

    let redacted_json = LogRedactor::redact_json(&json_val);

    let json_str = serde_json::to_string(&redacted_json).unwrap();
    assert!(!json_str.contains("AIzaSyTestKey1234567890123456789012345"));
    assert!(!json_str.contains("super_secret_token"));
    assert!(!json_str.contains("my_password_123"));
    assert!(!json_str.contains("user@test.com"));
    assert!(json_str.contains("[REDACTED_SECRET]"));
    assert!(json_str.contains("[REDACTED_EMAIL]"));
    assert!(json_str.contains("public value"));
}

#[test]
fn test_privacy_settings_default_and_persistence() {
    let db = setup_memory_db();

    // 1. Ambil pengaturan default
    let default_settings = PrivacyService::get_privacy_settings(&db).expect("Failed to get privacy settings");
    assert!(default_settings.auto_redact_logs);
    assert!(default_settings.require_sensitive_consent_prompt);
    assert!(default_settings.require_vision_consent_prompt);
    assert_eq!(default_settings.screenshot_retention_policy, "immediate_delete");
    assert!(default_settings.auto_cleanup_temp_on_exit);
    assert_eq!(default_settings.log_level, "INFO");

    // 2. Perbarui pengaturan
    let updated = PrivacyService::update_privacy_settings(
        &db,
        UpdatePrivacySettingsInput {
            auto_redact_logs: Some(true),
            require_sensitive_consent_prompt: Some(false),
            require_vision_consent_prompt: Some(true),
            screenshot_retention_policy: Some("retain_24_hours".to_string()),
            auto_cleanup_temp_on_exit: Some(false),
            log_level: Some("DEBUG".to_string()),
        },
    )
    .expect("Failed to update privacy settings");

    assert!(!updated.require_sensitive_consent_prompt);
    assert_eq!(updated.screenshot_retention_policy, "retain_24_hours");
    assert!(!updated.auto_cleanup_temp_on_exit);
    assert_eq!(updated.log_level, "DEBUG");

    // 3. Pastikan tersimpan dan terbaca kembali dari basis data
    let reloaded = PrivacyService::get_privacy_settings(&db).expect("Failed to reload privacy settings");
    assert_eq!(reloaded.screenshot_retention_policy, "retain_24_hours");
    assert_eq!(reloaded.log_level, "DEBUG");
}

#[test]
fn test_outbound_data_inspection_transparency() {
    let db = setup_memory_db();

    // Buat persona
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Budi Santoso"))
        .expect("Failed to create persona");

    // Tambahkan fakta biasa
    let _fact1 = FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "demographics".into(),
            fact_key: Some("usia".into()),
            attribute: "Usia".into(),
            data_type: Some("number".into()),
            value: "30".into(),
            unit: Some("tahun".into()),
            source_type: Some("user_declaration".into()),
            source_title: Some("Deklarasi Profil".into()),
            source_description: "Usia responden".into(),
            reference_uri: None,
            observed_at: Some("2026-01-15".into()),
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".into()),
            sensitivity: Some("ordinary".into()),
        },
    )
    .expect("Failed to create fact 1");

    // Lakukan inspeksi data keluar
    let disclosure = PrivacyService::inspect_outbound_data(
        &db,
        InspectOutboundDataInput {
            persona_id: persona.id.clone(),
            question_text: "Berapa usia Anda saat ini?".to_string(),
            instructions: None,
            options: Some(vec!["20-29".to_string(), "30-39".to_string(), "40+".to_string()]),
            allow_sensitive: Some(false),
            allow_highly_sensitive: Some(false),
            allow_external_vision: Some(false),
            has_image_attachment: Some(false),
        },
    )
    .expect("Failed to inspect outbound data");

    assert_eq!(disclosure.question_text, "Berapa usia Anda saat ini?");
    assert!(!disclosure.destination_endpoint.is_empty());
    assert_eq!(disclosure.included_facts.len(), 1);
    assert_eq!(disclosure.included_facts[0].attribute, "Usia");
    assert_eq!(disclosure.included_facts[0].status, "included");
    assert!(!disclosure.not_transmitted_guarantees.is_empty());
}

#[test]
fn test_outbound_data_excludes_sensitive_facts_when_consent_denied() {
    let db = setup_memory_db();

    let persona = PersonaService::create(&db, CreatePersonaInput::new("Dewi Lestari"))
        .expect("Failed to create persona");

    // Tambahkan fakta sensitif (pendapatan)
    let _fact_sens = FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "financial".into(),
            fact_key: Some("pendapatan".into()),
            attribute: "Penghasilan Bulanan".into(),
            data_type: Some("number".into()),
            value: "12500000".into(),
            unit: Some("IDR".into()),
            source_type: Some("user_declaration".into()),
            source_title: Some("Deklarasi Profil".into()),
            source_description: "Penghasilan".into(),
            reference_uri: None,
            observed_at: Some("2026-01-15".into()),
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".into()),
            sensitivity: Some("sensitive".into()),
        },
    )
    .expect("Failed to create fact");

    // Inspeksi dengan allow_sensitive: false
    let disclosure = PrivacyService::inspect_outbound_data(
        &db,
        InspectOutboundDataInput {
            persona_id: persona.id.clone(),
            question_text: "Berapakah perkiraan penghasilan bulanan Anda?".to_string(),
            instructions: None,
            options: None,
            allow_sensitive: Some(false),
            allow_highly_sensitive: Some(false),
            allow_external_vision: Some(false),
            has_image_attachment: Some(false),
        },
    )
    .expect("Failed to inspect outbound data");

    assert!(disclosure.includes_sensitive_data);
    assert!(!disclosure.sensitive_consent_granted);
    // Fakta sensitif harus masuk excluded_facts dengan status excluded_no_consent
    assert_eq!(disclosure.included_facts.len(), 0);
    assert_eq!(disclosure.excluded_facts.len(), 1);
    assert_eq!(disclosure.excluded_facts[0].status, "excluded_no_consent");
    assert_eq!(disclosure.excluded_facts[0].value_preview, "[REDAKSI_SENSITIF]");
}

#[test]
fn test_outbound_data_includes_sensitive_facts_when_consent_granted() {
    let db = setup_memory_db();

    let persona = PersonaService::create(&db, CreatePersonaInput::new("Siti Rahma"))
        .expect("Failed to create persona");

    let _fact_sens = FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "financial".into(),
            fact_key: Some("pendapatan".into()),
            attribute: "Penghasilan Bulanan".into(),
            data_type: Some("number".into()),
            value: "8000000".into(),
            unit: Some("IDR".into()),
            source_type: Some("user_declaration".into()),
            source_title: Some("Deklarasi Profil".into()),
            source_description: "Gaji".into(),
            reference_uri: None,
            observed_at: Some("2026-01-15".into()),
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".into()),
            sensitivity: Some("sensitive".into()),
        },
    )
    .expect("Failed to create fact");

    // Inspeksi dengan allow_sensitive: true
    let disclosure = PrivacyService::inspect_outbound_data(
        &db,
        InspectOutboundDataInput {
            persona_id: persona.id.clone(),
            question_text: "Berapakah penghasilan bulanan Anda?".to_string(),
            instructions: None,
            options: None,
            allow_sensitive: Some(true),
            allow_highly_sensitive: Some(false),
            allow_external_vision: Some(false),
            has_image_attachment: Some(false),
        },
    )
    .expect("Failed to inspect outbound data");

    assert!(disclosure.sensitive_consent_granted);
    assert_eq!(disclosure.included_facts.len(), 1);
    assert_eq!(disclosure.included_facts[0].attribute, "Penghasilan Bulanan");
    assert_eq!(disclosure.included_facts[0].status, "included");
}

#[test]
fn test_per_request_consent_enforcement_for_vision() {
    // 1. Vision tanpa persetujuan harus ditolak
    let consent_no_vision = SensitivityConsent {
        allow_sensitive: true,
        allow_highly_sensitive: false,
        allow_external_vision: false,
    };
    let result = PrivacyService::enforce_per_request_consent(Some(&consent_no_vision), false, true);
    assert!(result.is_err(), "Must reject external vision without consent");

    // 2. Vision dengan persetujuan harus diterima
    let consent_with_vision = SensitivityConsent {
        allow_sensitive: true,
        allow_highly_sensitive: false,
        allow_external_vision: true,
    };
    let result_ok = PrivacyService::enforce_per_request_consent(Some(&consent_with_vision), false, true);
    assert!(result_ok.is_ok(), "Must accept external vision with consent");
}

#[test]
fn test_diagnostic_logging_and_automatic_redaction() {
    let db = setup_memory_db();

    // Catat log dengan kunci rahasia mentah dan NIK
    let raw_msg = "Provider call with key AIzaSyD1234567890abcdefghijklmnopqrstuv and NIK 3171012304950001";
    let entry = PrivacyService::record_log(
        &db,
        RecordLogInput {
            level: "INFO".to_string(),
            category: "AI_PROVIDER".to_string(),
            message: raw_msg.to_string(),
            metadata_json: Some(serde_json::json!({"apiKey": "AIzaSySecret"}).to_string()),
        },
    )
    .expect("Failed to record diagnostic log");

    assert!(!entry.message.contains("AIzaSyD1234567890abcdefghijklmnopqrstuv"));
    assert!(!entry.message.contains("3171012304950001"));
    assert!(entry.message.contains("[REDACTED_API_KEY]"));
    assert!(entry.message.contains("[REDACTED_NIK_16_DIGITS]"));

    // Verifikasi log tersimpan di basis data dalam kondisi teredaksi
    let logs = PrivacyService::list_logs(&db, Some(10), None, None).expect("Failed to list logs");
    assert!(!logs.is_empty());
    assert!(!logs[0].message.contains("AIzaSy"));
    assert!(!logs[0].message.contains("3171012304950001"));

    // Uji pembersihan log
    let cleared = PrivacyService::clear_logs(&db).expect("Failed to clear logs");
    assert!(cleared >= 1);
    let logs_after = PrivacyService::list_logs(&db, Some(10), None, None).expect("Failed to list logs");
    assert!(logs_after.is_empty());
}

#[test]
fn test_retention_temp_captures_cleanup() {
    let db = setup_memory_db();
    let temp_dir = std::env::temp_dir().join(format!("sc_test_captures_{}", uuid::Uuid::new_v4()));
    let temp_captures_dir = temp_dir.join("temp_captures");
    fs::create_dir_all(&temp_captures_dir).expect("Failed to create temp_captures");

    // Buat beberapa file screenshot palsu di temp_captures
    let file1 = temp_captures_dir.join("capture1.png");
    let file2 = temp_captures_dir.join("capture2.png");
    File::create(&file1).unwrap().write_all(b"fake image data 1").unwrap();
    File::create(&file2).unwrap().write_all(b"fake image data 2").unwrap();

    // Verifikasi status retensi mencatat 2 file
    let status_before = PrivacyService::get_retention_status(&db, Some(&temp_dir))
        .expect("Failed to get retention status");
    assert_eq!(status_before.temp_file_count, 2);
    assert!(status_before.temp_total_bytes > 0);

    // Lakukan pembersihan seluruh file sementara (max_age_seconds: 0)
    let clean_res = PrivacyService::cleanup_temp_files(
        &db,
        Some(&temp_dir),
        CleanupTempFilesInput { max_age_seconds: Some(0) },
    )
    .expect("Failed to cleanup temp files");

    assert_eq!(clean_res.deleted_files_count, 2);
    assert!(clean_res.freed_bytes > 0);

    // Verifikasi file fisik telah terhapus
    let status_after = PrivacyService::get_retention_status(&db, Some(&temp_dir))
        .expect("Failed to get retention status");
    assert_eq!(status_after.temp_file_count, 0);
    assert_eq!(status_after.temp_total_bytes, 0);

    // Bersihkan direktori tes
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_system_diagnostics_retrieval() {
    let db = setup_memory_db();
    let temp_dir = std::env::temp_dir();

    let diag = PrivacyService::get_system_diagnostics(&db, Some(&temp_dir))
        .expect("Failed to get system diagnostics");

    assert_eq!(diag.app_version, "0.1.0");
    assert!(!diag.sqlite_version.is_empty());
    assert!(diag.db_schema_version >= 13);
    assert_eq!(diag.credential_store_type, "WindowsCredentialManager");
}
