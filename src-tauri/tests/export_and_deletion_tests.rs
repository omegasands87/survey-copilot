use std::path::Path;
use survey_copilot_lib::db::Database;
use survey_copilot_lib::domain::fact::CreateFactInput;
use survey_copilot_lib::domain::credential::SaveCredentialInput;
use survey_copilot_lib::infrastructure::credentials::in_memory::InMemoryCredentialStore;
use survey_copilot_lib::infrastructure::credentials::CredentialStore;
use survey_copilot_lib::domain::persona::CreatePersonaInput;
use survey_copilot_lib::services::{
    CredentialService, DeletionService, ExportService, FactService, PersonaService,
};

fn setup_memory_db() -> Database {
    Database::initialize(Path::new(":memory:")).expect("memory database should initialize")
}

#[test]
fn test_at_070_export_all_local_data_excludes_secrets() {
    let db = setup_memory_db();

    // 1. Seed Personas
    let p1 = PersonaService::create(
        &db,
        CreatePersonaInput::with_description("Budi Santoso", Some("Responden Utama".into())),
    )
    .expect("create p1");

    let p2 = PersonaService::create(
        &db,
        CreatePersonaInput::with_description("Dewi Lestari", Some("Responden Kedua".into())),
    )
    .expect("create p2");

    // 2. Seed Facts (Ordinary, Sensitive, Highly Sensitive)
    FactService::create(
        &db,
        CreateFactInput {
            persona_id: p1.id.clone(),
            category: "Demografi".into(),
            fact_key: Some("age".into()),
            attribute: "Usia".into(),
            data_type: Some("number".into()),
            value: "28".into(),
            unit: Some("tahun".into()),
            source_type: Some("user_declaration".into()),
            source_title: Some("Deklarasi Profil".into()),
            source_description: "Pernyataan mandiri".into(),
            reference_uri: None,
            observed_at: Some("2024-01-01".into()),
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".into()),
            sensitivity: Some("ordinary".into()),
        },
    )
    .expect("fact age");

    FactService::create(
        &db,
        CreateFactInput {
            persona_id: p1.id.clone(),
            category: "Finansial".into(),
            fact_key: Some("monthly_income".into()),
            attribute: "Penghasilan Bulanan".into(),
            data_type: Some("number".into()),
            value: "15.000.000".into(),
            unit: Some("IDR".into()),
            source_type: Some("uploaded_document".into()),
            source_title: Some("Slip Gaji".into()),
            source_description: "Slip Gaji Kantor".into(),
            reference_uri: None,
            observed_at: Some("2024-02-01".into()),
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".into()),
            sensitivity: Some("sensitive".into()),
        },
    )
    .expect("fact income");

    FactService::create(
        &db,
        CreateFactInput {
            persona_id: p2.id.clone(),
            category: "Kesehatan".into(),
            fact_key: Some("blood_type".into()),
            attribute: "Golongan Darah".into(),
            data_type: Some("string".into()),
            value: "O+".into(),
            unit: None,
            source_type: Some("uploaded_document".into()),
            source_title: Some("Hasil Lab".into()),
            source_description: "Laboratorium Klinik".into(),
            reference_uri: None,
            observed_at: Some("2024-03-01".into()),
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".into()),
            sensitivity: Some("highly_sensitive".into()),
        },
    )
    .expect("fact medical");

    // 3. Seed Provider config with sensitive credential reference
    {
        let conn = db.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO provider_configs (id, display_name, provider_type, base_url, adapter_type, model_id, credential_ref, created_at, updated_at)
             VALUES ('prov-1', 'Gemini Pro', 'google', 'https://api.example.com', 'gemini', 'gemini-1.5-pro', 'win-cred-secret-gemini-key-999', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
            [],
        ).expect("insert provider config");

        // Seed preference
        conn.execute(
            "INSERT INTO persona_preferences (id, persona_id, preference_key, value_json, created_at, updated_at)
             VALUES ('pref-1', ?1, 'theme', '\"dark\"', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
            [&p1.id],
        ).expect("insert pref");

        // Seed survey session & question & recommendation & answer
        conn.execute(
            "INSERT INTO survey_sessions (id, persona_id, title, platform_label, status, created_at, updated_at)
             VALUES ('sess-1', ?1, 'Survei Belanja Online', 'Jakpat', 'active', '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
            [&p1.id],
        ).expect("insert session");

        conn.execute(
            "INSERT INTO survey_questions (id, session_id, question_text_original, extraction_status, question_order, created_at, updated_at)
             VALUES ('q-1', 'sess-1', 'Berapa usia Anda saat ini?', 'manual', 1, '2024-01-01T00:00:00Z', '2024-01-01T00:00:00Z')",
            [],
        ).expect("insert question");

        conn.execute(
            "INSERT INTO answer_recommendations (id, question_id, status, draft_text, interpretation, missing_information_json, conflicts_json, assumptions_json, created_at)
             VALUES ('rec-1', 'q-1', 'ready', '28 tahun', 'Usia responden saat ini', '[]', '[]', '[]', '2024-01-01T00:00:00Z')",
            [],
        ).expect("insert rec");

        conn.execute(
            "INSERT INTO answer_history (id, persona_id, question_id, recommendation_id, action, actual_answer, created_at)
             VALUES ('ans-1', ?1, 'q-1', 'rec-1', 'confirmed', '28 tahun', '2024-01-01T00:00:00Z')",
            [&p1.id],
        ).expect("insert answer");
    }

    // 4. Test Export Summary
    let summary = ExportService::get_summary(&db, None).expect("summary");
    assert_eq!(summary.export_scope, "full_application");
    assert_eq!(summary.total_personas, 2);
    assert_eq!(summary.total_facts, 3);
    assert_eq!(summary.total_sources, 3);
    assert_eq!(summary.total_preferences, 1);
    assert_eq!(summary.total_sessions, 1);
    assert_eq!(summary.total_questions, 1);
    assert_eq!(summary.total_recommendations, 1);
    assert_eq!(summary.total_answers, 1);
    assert_eq!(summary.sensitive_facts_count, 1);
    assert_eq!(summary.highly_sensitive_facts_count, 1);
    assert!(summary.has_sensitive_data);
    assert!(summary.warning_message.is_some());

    // 5. Test Full Export Payload
    let payload = ExportService::export_data(&db, None).expect("export payload");
    assert_eq!(payload.schema_version, "1.0.0");
    assert_eq!(payload.app_version, "0.1.0");
    assert_eq!(payload.export_scope, "full_application");
    assert_eq!(payload.data.personas.len(), 2);
    assert_eq!(payload.data.facts.len(), 3);
    assert_eq!(payload.data.fact_sources.len(), 3);
    assert_eq!(payload.data.persona_preferences.len(), 1);
    assert_eq!(payload.data.survey_sessions.len(), 1);
    assert_eq!(payload.data.survey_questions.len(), 1);
    assert_eq!(payload.data.answer_recommendations.len(), 1);
    assert_eq!(payload.data.answer_history.len(), 1);

    // 6. Test Export JSON String & Strict Secret Exclusion
    let json_str = ExportService::export_data_as_json_string(&db, None).expect("json string");
    assert!(json_str.contains("\"schemaVersion\": \"1.0.0\""));
    assert!(json_str.contains("Budi Santoso"));
    assert!(json_str.contains("Dewi Lestari"));
    assert!(json_str.contains("Survei Belanja Online"));

    // Absolute exclusion check: credentials and secret tokens MUST NOT exist anywhere
    assert!(!json_str.contains("win-cred-secret-gemini-key-999"));
    assert!(!json_str.contains("credential_ref"));
    assert!(!json_str.contains("credentialRef"));
    assert!(!json_str.contains("api_key"));
    assert!(!json_str.contains("apiKey"));
}

#[test]
fn test_per_persona_export_strict_cross_persona_isolation() {
    let db = setup_memory_db();

    let p_a = PersonaService::create(&db, CreatePersonaInput::new("Persona A")).expect("p_a");
    let p_b = PersonaService::create(&db, CreatePersonaInput::new("Persona B")).expect("p_b");

    // Fact for Persona A
    FactService::create(
        &db,
        CreateFactInput {
            persona_id: p_a.id.clone(),
            category: "Pekerjaan".into(),
            fact_key: Some("job_title".into()),
            attribute: "Pekerjaan".into(),
            data_type: Some("string".into()),
            value: "Software Engineer".into(),
            unit: None,
            source_type: Some("user_declaration".into()),
            source_title: Some("Deklarasi A".into()),
            source_description: "Input langsung A".into(),
            reference_uri: None,
            observed_at: None,
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".into()),
            sensitivity: Some("ordinary".into()),
        },
    )
    .expect("fact A");

    // Fact for Persona B
    FactService::create(
        &db,
        CreateFactInput {
            persona_id: p_b.id.clone(),
            category: "Pekerjaan".into(),
            fact_key: Some("job_title".into()),
            attribute: "Pekerjaan".into(),
            data_type: Some("string".into()),
            value: "Dokter Umum".into(),
            unit: None,
            source_type: Some("user_declaration".into()),
            source_title: Some("Deklarasi B".into()),
            source_description: "Input langsung B".into(),
            reference_uri: None,
            observed_at: None,
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".into()),
            sensitivity: Some("ordinary".into()),
        },
    )
    .expect("fact B");

    // Export Persona A
    let export_a = ExportService::export_data(&db, Some(&p_a.id)).expect("export A");
    assert_eq!(export_a.export_scope, "single_persona");
    assert_eq!(export_a.target_persona_id, Some(p_a.id.clone()));
    assert_eq!(export_a.target_persona_name, Some("Persona A".into()));

    assert_eq!(export_a.data.personas.len(), 1);
    assert_eq!(export_a.data.personas[0].id, p_a.id);
    assert_eq!(export_a.data.facts.len(), 1);
    assert_eq!(export_a.data.facts[0].persona_id, p_a.id);
    assert_eq!(
        export_a.data.facts[0].value,
        serde_json::json!("Software Engineer")
    );

    // Verify ZERO cross-persona data leakage
    for f in &export_a.data.facts {
        assert_ne!(f.persona_id, p_b.id);
        assert_ne!(f.value, serde_json::json!("Dokter Umum"));
    }
    for s in &export_a.data.fact_sources {
        assert_eq!(s.persona_id, p_a.id);
    }

    // Export Persona B
    let export_b = ExportService::export_data(&db, Some(&p_b.id)).expect("export B");
    assert_eq!(export_b.export_scope, "single_persona");
    assert_eq!(export_b.target_persona_id, Some(p_b.id.clone()));
    assert_eq!(export_b.data.personas.len(), 1);
    assert_eq!(export_b.data.facts.len(), 1);
    assert_eq!(
        export_b.data.facts[0].value,
        serde_json::json!("Dokter Umum")
    );
}

#[test]
fn test_empty_database_export_validity() {
    let db = setup_memory_db();

    let summary = ExportService::get_summary(&db, None).expect("empty summary");
    assert_eq!(summary.total_personas, 0);
    assert_eq!(summary.total_facts, 0);
    assert_eq!(summary.estimated_records_count, 0);
    assert!(!summary.has_sensitive_data);

    let json_str = ExportService::export_data_as_json_string(&db, None).expect("empty json");
    assert!(json_str.contains("\"schemaVersion\": \"1.0.0\""));
    assert!(json_str.contains("\"personas\": []"));
    assert!(json_str.contains("\"facts\": []"));
}

#[test]
fn test_at_071_full_application_wipe_and_zero_orphans() {
    let db = setup_memory_db();
    let store = InMemoryCredentialStore::new();

    // Seed data
    let p = PersonaService::create(&db, CreatePersonaInput::new("Responden Uji")).expect("p");
    FactService::create(
        &db,
        CreateFactInput {
            persona_id: p.id.clone(),
            category: "Demografi".into(),
            fact_key: Some("city".into()),
            attribute: "Kota".into(),
            data_type: Some("string".into()),
            value: "Surabaya".into(),
            unit: None,
            source_type: Some("manual_entry".into()),
            source_title: Some("Alamat".into()),
            source_description: "KTP".into(),
            reference_uri: None,
            observed_at: None,
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".into()),
            sensitivity: Some("ordinary".into()),
        },
    )
    .expect("fact");

    let credential = CredentialService::save_credential(
        &db,
        &store,
        SaveCredentialInput {
            id: None,
            provider_type: "gemini".to_string(),
            label: "Credential to wipe".to_string(),
            secret: "StageOneWipeSecretKey987654".to_string(),
        },
    )
    .expect("credential should be saved");

    // Insert session and capture
    {
        let conn = db.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO survey_sessions (id, persona_id, status, created_at, updated_at) VALUES ('s1', ?1, 'active', '2024-01-01', '2024-01-01')",
            [&p.id],
        ).unwrap();
        conn.execute(
            "INSERT INTO provider_configs (id, display_name, provider_type, adapter_type, model_id, credential_ref, created_at, updated_at)
             VALUES ('pr1', 'Test', 'google', 'gemini', 'm1', 'ref1', '2024-01-01', '2024-01-01')",
            [],
        ).unwrap();
    }

    // Verify pre-wipe counts
    let before = DeletionService::get_wipe_summary(&db).expect("before");
    assert!(before.personas > 0);
    assert!(before.facts > 0);
    assert!(before.provider_configs > 0);
    assert_eq!(before.credentials, 1);
    assert!(before.total > 0);

    // Execute Wipe with exact confirmation string
    let report = DeletionService::wipe_all_data(
        &db, &store, None, "HAPUS SEMUA DATA"
    ).expect("wipe");
    assert_eq!(report.deleted_personas_count, 1);
    assert_eq!(report.deleted_facts_count, 1);
    assert_eq!(report.deleted_provider_configs_count, 1);
    assert_eq!(report.deleted_credentials_count, 1);
    assert!(!store.exists(&credential.credential_ref).unwrap());
    assert!(report.wal_checkpoint_status.contains("Berhasil"));
    assert!(report.vacuum_status.contains("Berhasil"));
    assert_eq!(report.unaffected_external_items.len(), 3);

    // Verify all tables are completely wiped (0 orphans)
    let after = DeletionService::get_wipe_summary(&db).expect("after");
    assert_eq!(after.personas, 0);
    assert_eq!(after.facts, 0);
    assert_eq!(after.sources, 0);
    assert_eq!(after.history, 0);
    assert_eq!(after.sessions, 0);
    assert_eq!(after.questions, 0);
    assert_eq!(after.recommendations, 0);
    assert_eq!(after.answers, 0);
    assert_eq!(after.provider_configs, 0);
    assert_eq!(after.credentials, 0);
    assert_eq!(after.ocr_extractions, 0);
    assert_eq!(after.recognitions, 0);
    assert_eq!(after.branch_transitions, 0);
    assert_eq!(after.question_predictions, 0);
    assert_eq!(after.diagnostic_logs, 0);

    // Verify _migrations is preserved and provider_pool_settings is re-initialized
    {
        let conn = db.connection.lock().unwrap();
        let migration_count: i64 = conn
            .query_row("SELECT COUNT(1) FROM _migrations", [], |r| r.get(0))
            .unwrap();
        assert!(migration_count >= 2);

        let pool_count: i64 = conn
            .query_row("SELECT COUNT(1) FROM provider_pool_settings", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(pool_count, 1);
    }
}

#[test]
fn test_wipe_requires_exact_confirmation() {
    let db = setup_memory_db();
    let store = InMemoryCredentialStore::new();
    PersonaService::create(&db, CreatePersonaInput::new("Tetap Ada")).expect("p");

    let err1 = DeletionService::wipe_all_data(&db, &store, None, "hapus semua data").unwrap_err();
    assert_eq!(err1.field, Some("confirmationText".into()));

    let err2 = DeletionService::wipe_all_data(&db, &store, None, "DELETE").unwrap_err();
    assert_eq!(err2.field, Some("confirmationText".into()));

    let err3 = DeletionService::wipe_all_data(&db, &store, None, "").unwrap_err();
    assert_eq!(err3.field, Some("confirmationText".into()));

    // Verify data remains untouched
    let list = PersonaService::list(&db, true).unwrap();
    assert_eq!(list.len(), 1);
}

#[test]
fn test_persona_deletion_cleans_temporary_screenshot_files() {
    let db = setup_memory_db();
    let p = PersonaService::create(&db, CreatePersonaInput::new("Hapus File")).expect("p");

    // Create a real temporary file on disk
    let temp_dir = std::env::temp_dir();
    let fake_capture_file = temp_dir.join(format!("survey_cap_test_{}.png", uuid::Uuid::new_v4()));
    std::fs::write(&fake_capture_file, b"fake image bytes").expect("write temp file");
    assert!(fake_capture_file.exists());

    // Record session and capture pointing to this file
    {
        let conn = db.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO survey_sessions (id, persona_id, status, created_at, updated_at) VALUES ('sess-f1', ?1, 'active', '2024-01-01', '2024-01-01')",
            [&p.id],
        ).unwrap();
        conn.execute(
            "INSERT INTO screen_captures (id, session_id, capture_mode, captured_at, temporary_image_path, created_at)
             VALUES ('cap-f1', 'sess-f1', 'region', '2024-01-01', ?1, '2024-01-01')",
            [fake_capture_file.to_str().unwrap()],
        ).unwrap();
    }

    // Delete persona
    PersonaService::delete(&db, &p.id).expect("delete persona");

    // Verify the temporary file on disk has been cleaned up!
    assert!(!fake_capture_file.exists());
}

#[test]
fn test_retention_policy_info() {
    let policy = DeletionService::get_retention_policy_info();
    assert!(!policy.active_facts_policy.is_empty());
    assert!(!policy.archived_facts_policy.is_empty());
    assert!(!policy.screen_capture_policy.is_empty());
    assert!(!policy.ocr_data_policy.is_empty());
    assert!(!policy.survey_history_policy.is_empty());
    assert!(!policy.export_file_policy.is_empty());
    assert!(!policy.logs_and_diagnostics_policy.is_empty());
    assert!(!policy.deletion_scope_disclaimer.is_empty());
}
