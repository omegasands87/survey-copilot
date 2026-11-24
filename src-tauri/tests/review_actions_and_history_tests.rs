use std::sync::Arc;

use survey_copilot_lib::db::Database;
use survey_copilot_lib::domain::answer_engine::{
    ConfirmRecommendationInput, CopyAnswerInput, EditAnswerInput, EvidenceStatus,
    GenerateAnswerInput, ListAnswerHistoryInput, RegenerateAnswerInput, RejectAnswerInput,
    ReviewAction,
};
use survey_copilot_lib::domain::fact::CreateFactInput;
use survey_copilot_lib::domain::persona::CreatePersonaInput;
use survey_copilot_lib::domain::survey::{
    ConfirmationStatus, CreateQuestionInput, CreateSessionInput,
};
use survey_copilot_lib::infrastructure::credentials::in_memory::InMemoryCredentialStore;
use survey_copilot_lib::infrastructure::http::MockHttpTransport;
use survey_copilot_lib::services::{
    AnswerEngineService, FactService, PersonaService, ProviderPoolService, SurveyService,
};

fn create_test_db() -> Database {
    Database::initialize(std::path::Path::new(":memory:"))
        .expect("In-memory database should initialize")
}

#[test]
fn test_edit_answer_action_and_history() {
    let db = create_test_db();
    let credential_store = InMemoryCredentialStore::new();
    let mock_transport = Arc::new(MockHttpTransport::new());
    let pool_service = ProviderPoolService::new();

    // 1. Setup Persona
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Budi Santoso"))
        .expect("Persona created");

    // 2. Setup Fact
    FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "employment_education".to_string(),
            fact_key: Some("education_level".to_string()),
            attribute: "education_level".to_string(),
            data_type: Some("string".to_string()),
            value: "Sarjana (S1)".to_string(),
            unit: None,
            source_type: Some("user_declaration".to_string()),
            source_title: Some("Ijazah".to_string()),
            source_description: "Pendidikan terakhir".to_string(),
            reference_uri: None,
            observed_at: Some("2026-01-01".to_string()),
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".to_string()),
            sensitivity: Some("ordinary".to_string()),
        },
    )
    .expect("Fact created");

    // 3. Setup Session & Question
    let session = SurveyService::db_create_session(
        &db,
        CreateSessionInput {
            persona_id: persona.id.clone(),
            title: "Survei Pendidikan 2026".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: None,
            external_survey_id: None,
            language: Some("id".to_string()),
            category: None,
            notes: None,
        },
    )
    .expect("Session created");

    let question = SurveyService::db_create_question(
        &db,
        CreateQuestionInput {
            persona_id: persona.id.clone(),
            session_id: session.id,
            capture_id: None,
            question_text_original: "Apakah tingkat pendidikan terakhir Anda?".to_string(),
            question_order: Some(1),
            instructions: None,
            options: Some(vec![
                "SMA / SMK".to_string(),
                "Sarjana (S1)".to_string(),
                "Magister (S2)".to_string(),
            ]),
            question_type: None,
            answer_format: None,
            provenance: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .expect("Question created");

    // 4. Generate AI Recommendation (deterministic fallback matching)
    let rec = AnswerEngineService::generate_recommendation(
        &db,
        &credential_store,
        mock_transport.clone(),
        &pool_service,
        GenerateAnswerInput {
            persona_id: persona.id.clone(),
            session_id: Some(question.session_id.clone()),
            question_id: Some(question.id.clone()),
            question_text: question.question_text_original.clone(),
            instructions: None,
            options: Some(question.options.clone()),
            consent: None,
            provider_config_id: None,
            prompt_version: None,
        },
        None,
    )
    .expect("Recommendation generated");

    assert_eq!(rec.status, EvidenceStatus::Supported);
    assert_eq!(rec.draft_answer, Some("Sarjana (S1)".to_string()));

    // 5. User performs EDIT action
    let edit_input = EditAnswerInput {
        persona_id: persona.id.clone(),
        question_id: question.id.clone(),
        recommendation_id: Some(rec.id.clone()),
        actual_answer: "S1 Teknik Informatika".to_string(),
        reason: Some("Menambahkan jurusan secara lebih spesifik".to_string()),
    };

    let history_item =
        AnswerEngineService::edit_answer(&db, edit_input).expect("Edit answer should succeed");

    // Verifikasi pemisahan draft_answer vs actual_answer
    assert_eq!(history_item.action, ReviewAction::Edited);
    assert_eq!(history_item.draft_answer, Some("Sarjana (S1)".to_string()));
    assert_eq!(history_item.actual_answer, "S1 Teknik Informatika");
    assert_eq!(
        history_item.reason,
        Some("Menambahkan jurusan secara lebih spesifik".to_string())
    );

    // Verifikasi survey_questions terupdate
    let updated_q =
        SurveyService::db_get_question(&db, &persona.id, &question.id).expect("Question retrieved");
    assert_eq!(
        updated_q.user_answer_draft,
        Some("S1 Teknik Informatika".to_string())
    );

    // Verifikasi riwayat tersimpan di database
    let history_list = AnswerEngineService::list_history(
        &db,
        ListAnswerHistoryInput {
            persona_id: persona.id.clone(),
            question_id: Some(question.id.clone()),
            action: None,
            limit: None,
            offset: None,
        },
    )
    .expect("List history should succeed");

    assert_eq!(history_list.len(), 1);
    assert_eq!(history_list[0].id, history_item.id);
    assert_eq!(history_list[0].action, ReviewAction::Edited);
}

#[test]
fn test_confirm_answer_without_external_submission() {
    let db = create_test_db();
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Siti Rahma")).unwrap();

    let session = SurveyService::db_create_session(
        &db,
        CreateSessionInput {
            persona_id: persona.id.clone(),
            title: "Survei Keuangan".to_string(),
            platform_label: None,
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    let question = SurveyService::db_create_question(
        &db,
        CreateQuestionInput {
            persona_id: persona.id.clone(),
            session_id: session.id,
            capture_id: None,
            question_text_original: "Apakah Anda memiliki tabungan?".to_string(),
            question_order: Some(1),
            instructions: None,
            options: Some(vec!["Ya".to_string(), "Tidak".to_string()]),
            question_type: None,
            answer_format: None,
            provenance: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Konfirmasi Jawaban Aktual
    let confirm_input = ConfirmRecommendationInput {
        persona_id: persona.id.clone(),
        question_id: question.id.clone(),
        recommendation_id: None,
        actual_answer: "Ya".to_string(),
    };

    let item = AnswerEngineService::confirm_answer(&db, confirm_input)
        .expect("Confirm answer should succeed");

    assert_eq!(item.action, ReviewAction::Confirmed);
    assert_eq!(item.actual_answer, "Ya");
    assert_eq!(item.review_status, "confirmed");

    // Periksa status pertanyaan di database
    let q = SurveyService::db_get_question(&db, &persona.id, &question.id).unwrap();
    assert_eq!(q.confirmation_status, ConfirmationStatus::Confirmed);
    assert_eq!(q.user_answer_confirmed, Some("Ya".to_string()));
    assert!(q.confirmed_at.is_some());
}

#[test]
fn test_confirmed_answer_never_auto_promoted_to_profile_fact() {
    let db = create_test_db();
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Ahmad Dahlan")).unwrap();

    // Hitung jumlah fakta profil awal
    let facts_before = FactService::list(&db, &persona.id, false).unwrap();
    assert_eq!(facts_before.len(), 0);

    let session = SurveyService::db_create_session(
        &db,
        CreateSessionInput {
            persona_id: persona.id.clone(),
            title: "Survei Minat Baca".to_string(),
            platform_label: None,
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    let question = SurveyService::db_create_question(
        &db,
        CreateQuestionInput {
            persona_id: persona.id.clone(),
            session_id: session.id,
            capture_id: None,
            question_text_original: "Berapa buku yang Anda baca bulan ini?".to_string(),
            question_order: Some(1),
            instructions: None,
            options: None,
            question_type: None,
            answer_format: None,
            provenance: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Konfirmasi jawaban
    AnswerEngineService::confirm_answer(
        &db,
        ConfirmRecommendationInput {
            persona_id: persona.id.clone(),
            question_id: question.id.clone(),
            recommendation_id: None,
            actual_answer: "3 buku".to_string(),
        },
    )
    .unwrap();

    // CRITICAL REQUIREMENT VERIFICATION:
    // Konfirmasi jawaban survei TIDAK BOLEH otomatis menjadi fakta profil terkonfirmasi!
    let facts_after = FactService::list(&db, &persona.id, false).unwrap();
    assert_eq!(
        facts_after.len(),
        0,
        "Jawaban survei yang dikonfirmasi DILARANG KERAS otomatis dipromosikan menjadi fakta profil!"
    );
}

#[test]
fn test_copy_answer_action_audit() {
    let db = create_test_db();
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Dewi Lestari")).unwrap();

    let session = SurveyService::db_create_session(
        &db,
        CreateSessionInput {
            persona_id: persona.id.clone(),
            title: "Survei Hobi".to_string(),
            platform_label: None,
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    let question = SurveyService::db_create_question(
        &db,
        CreateQuestionInput {
            persona_id: persona.id.clone(),
            session_id: session.id,
            capture_id: None,
            question_text_original: "Apa hobi Anda?".to_string(),
            question_order: Some(1),
            instructions: None,
            options: None,
            question_type: None,
            answer_format: None,
            provenance: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    let copy_item = AnswerEngineService::copy_answer(
        &db,
        CopyAnswerInput {
            persona_id: persona.id.clone(),
            question_id: question.id.clone(),
            recommendation_id: None,
            copied_text: "Menulis fiksi".to_string(),
        },
    )
    .expect("Copy should succeed");

    assert_eq!(copy_item.action, ReviewAction::Copied);
    assert_eq!(copy_item.actual_answer, "Menulis fiksi");

    // Periksa riwayat
    let history = AnswerEngineService::list_history(
        &db,
        ListAnswerHistoryInput {
            persona_id: persona.id.clone(),
            question_id: Some(question.id),
            action: Some("copied".to_string()),
            limit: None,
            offset: None,
        },
    )
    .unwrap();

    assert_eq!(history.len(), 1);
    assert_eq!(history[0].action, ReviewAction::Copied);
}

#[test]
fn test_reject_answer_action_with_reason() {
    let db = create_test_db();
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Joko Widodo")).unwrap();

    let session = SurveyService::db_create_session(
        &db,
        CreateSessionInput {
            persona_id: persona.id.clone(),
            title: "Survei Kebijakan".to_string(),
            platform_label: None,
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    let question = SurveyService::db_create_question(
        &db,
        CreateQuestionInput {
            persona_id: persona.id.clone(),
            session_id: session.id,
            capture_id: None,
            question_text_original: "Apakah Anda setuju dengan kebijakan baru?".to_string(),
            question_order: Some(1),
            instructions: None,
            options: Some(vec!["Setuju".to_string(), "Tidak Setuju".to_string()]),
            question_type: None,
            answer_format: None,
            provenance: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    let reject_item = AnswerEngineService::reject_answer(
        &db,
        RejectAnswerInput {
            persona_id: persona.id.clone(),
            question_id: question.id.clone(),
            recommendation_id: None,
            reason: Some("Opsi tidak mencerminkan pandangan independen saya".to_string()),
        },
    )
    .expect("Reject should succeed");

    assert_eq!(reject_item.action, ReviewAction::Rejected);
    assert_eq!(reject_item.review_status, "rejected");
    assert_eq!(
        reject_item.reason,
        Some("Opsi tidak mencerminkan pandangan independen saya".to_string())
    );

    // Periksa status di question
    let q = SurveyService::db_get_question(&db, &persona.id, &question.id).unwrap();
    assert_eq!(q.confirmation_status, ConfirmationStatus::Rejected);
}

#[test]
fn test_regenerate_answer_preserves_previous_drafts_and_history() {
    let db = create_test_db();
    let credential_store = InMemoryCredentialStore::new();
    let mock_transport = Arc::new(MockHttpTransport::new());
    let pool_service = ProviderPoolService::new();

    let persona = PersonaService::create(&db, CreatePersonaInput::new("Rina Nose")).unwrap();

    // Buat fakta profil
    FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "demographic".to_string(),
            fact_key: Some("city".to_string()),
            attribute: "city".to_string(),
            data_type: Some("string".to_string()),
            value: "Bandung".to_string(),
            unit: None,
            source_type: Some("user_declaration".to_string()),
            source_title: Some("KTP".to_string()),
            source_description: "Kota domisili".to_string(),
            reference_uri: None,
            observed_at: Some("2026-01-01".to_string()),
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".to_string()),
            sensitivity: Some("ordinary".to_string()),
        },
    )
    .unwrap();

    let session = SurveyService::db_create_session(
        &db,
        CreateSessionInput {
            persona_id: persona.id.clone(),
            title: "Survei Domisili".to_string(),
            platform_label: None,
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    let question = SurveyService::db_create_question(
        &db,
        CreateQuestionInput {
            persona_id: persona.id.clone(),
            session_id: session.id,
            capture_id: None,
            question_text_original: "Di kota mana Anda berdomisili?".to_string(),
            question_order: Some(1),
            instructions: None,
            options: Some(vec![
                "Jakarta".to_string(),
                "Bandung".to_string(),
                "Surabaya".to_string(),
            ]),
            question_type: None,
            answer_format: None,
            provenance: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // 1. Rekomendasi Pertama
    let rec1 = AnswerEngineService::generate_recommendation(
        &db,
        &credential_store,
        mock_transport.clone(),
        &pool_service,
        GenerateAnswerInput {
            persona_id: persona.id.clone(),
            session_id: Some(question.session_id.clone()),
            question_id: Some(question.id.clone()),
            question_text: question.question_text_original.clone(),
            instructions: None,
            options: Some(question.options.clone()),
            consent: None,
            provider_config_id: None,
            prompt_version: None,
        },
        None,
    )
    .unwrap();

    assert_eq!(rec1.draft_answer, Some("Bandung".to_string()));

    // Pengguna mengedit draf
    AnswerEngineService::edit_answer(
        &db,
        EditAnswerInput {
            persona_id: persona.id.clone(),
            question_id: question.id.clone(),
            recommendation_id: Some(rec1.id.clone()),
            actual_answer: "Kota Bandung (Jawa Barat)".to_string(),
            reason: None,
        },
    )
    .unwrap();

    // 2. Pengguna meminta regenerasi rekomendasi AI
    let rec2 = AnswerEngineService::regenerate_answer(
        &db,
        &credential_store,
        mock_transport,
        &pool_service,
        RegenerateAnswerInput {
            persona_id: persona.id.clone(),
            session_id: Some(question.session_id),
            question_id: question.id.clone(),
            question_text: question.question_text_original,
            instructions: Some("Perhatikan wilayah".to_string()),
            options: Some(question.options),
            consent: None,
            provider_config_id: None,
            prompt_version: None,
            reason: Some("Mencoba opsi lebih presisi".to_string()),
        },
        None,
    )
    .expect("Regeneration should succeed");

    // REGENERATION CRITERIA VERIFICATION:
    // - Menghasilkan ID rekomendasi baru
    // - Rekomendasi sebelumnya tidak hilang
    assert_ne!(rec1.id, rec2.id);

    let revisions =
        AnswerEngineService::list_recommendation_revisions(&db, &persona.id, &question.id).unwrap();
    assert_eq!(
        revisions.len(),
        2,
        "Kedua versi rekomendasi wajib tersimpan!"
    );

    // Draf dan suntingan pengguna tetap dipertahankan
    let current_q = SurveyService::db_get_question(&db, &persona.id, &question.id).unwrap();
    assert_eq!(
        current_q.user_answer_draft,
        Some("Kota Bandung (Jawa Barat)".to_string()),
        "Suntingan pengguna wajib dipreservasi saat regenerasi AI terjadi!"
    );

    // Riwayat mencatat event regenerated
    let history = AnswerEngineService::list_history(
        &db,
        ListAnswerHistoryInput {
            persona_id: persona.id,
            question_id: Some(question.id),
            action: Some("regenerated".to_string()),
            limit: None,
            offset: None,
        },
    )
    .unwrap();

    assert_eq!(history.len(), 1);
    assert_eq!(history[0].action, ReviewAction::Regenerated);
    assert_eq!(
        history[0].reason,
        Some("Mencoba opsi lebih presisi".to_string())
    );
}

#[test]
fn test_strict_persona_isolation_on_history_and_revisions() {
    let db = create_test_db();

    // Setup 2 persona berbeda
    let persona_a = PersonaService::create(&db, CreatePersonaInput::new("Persona A")).unwrap();
    let persona_b = PersonaService::create(&db, CreatePersonaInput::new("Persona B")).unwrap();

    let session_a = SurveyService::db_create_session(
        &db,
        CreateSessionInput {
            persona_id: persona_a.id.clone(),
            title: "Sesi A".to_string(),
            platform_label: None,
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    let question_a = SurveyService::db_create_question(
        &db,
        CreateQuestionInput {
            persona_id: persona_a.id.clone(),
            session_id: session_a.id,
            capture_id: None,
            question_text_original: "Pertanyaan Rahasia A".to_string(),
            question_order: Some(1),
            instructions: None,
            options: None,
            question_type: None,
            answer_format: None,
            provenance: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Persona A mengedit dan mengonfirmasi jawaban
    AnswerEngineService::edit_answer(
        &db,
        EditAnswerInput {
            persona_id: persona_a.id.clone(),
            question_id: question_a.id.clone(),
            recommendation_id: None,
            actual_answer: "Jawaban Rahasia A".to_string(),
            reason: None,
        },
    )
    .unwrap();

    // UJI ISOLASI PERSONA B:
    // 1. Persona B meminta riwayat -> harus KOSONG (0 item)
    let history_b = AnswerEngineService::list_history(
        &db,
        ListAnswerHistoryInput {
            persona_id: persona_b.id.clone(),
            question_id: None,
            action: None,
            limit: None,
            offset: None,
        },
    )
    .unwrap();
    assert_eq!(history_b.len(), 0);

    // 2. Persona B meminta revisi pertanyaan milik Persona A -> harus KOSONG (0 item)
    let revisions_b =
        AnswerEngineService::list_recommendation_revisions(&db, &persona_b.id, &question_a.id)
            .unwrap();
    assert_eq!(revisions_b.len(), 0);

    // 3. Persona B mencoba mengedit pertanyaan milik Persona A -> Error NotFound!
    let edit_tamper = AnswerEngineService::edit_answer(
        &db,
        EditAnswerInput {
            persona_id: persona_b.id.clone(),
            question_id: question_a.id.clone(),
            recommendation_id: None,
            actual_answer: "Perubahan Ilegal oleh Persona B".to_string(),
            reason: None,
        },
    );
    assert!(edit_tamper.is_err());

    // 4. Persona B mencoba mengonfirmasi pertanyaan milik Persona A -> Error NotFound!
    let confirm_tamper = AnswerEngineService::confirm_answer(
        &db,
        ConfirmRecommendationInput {
            persona_id: persona_b.id,
            question_id: question_a.id,
            recommendation_id: None,
            actual_answer: "Konfirmasi Ilegal".to_string(),
        },
    );
    assert!(confirm_tamper.is_err());
}

#[test]
fn test_filter_answer_history_by_action() {
    let db = create_test_db();
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Koko Ardi")).unwrap();

    let session = SurveyService::db_create_session(
        &db,
        CreateSessionInput {
            persona_id: persona.id.clone(),
            title: "Sesi Filter".to_string(),
            platform_label: None,
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    let question = SurveyService::db_create_question(
        &db,
        CreateQuestionInput {
            persona_id: persona.id.clone(),
            session_id: session.id,
            capture_id: None,
            question_text_original: "Apakah Anda suka berolahraga?".to_string(),
            question_order: Some(1),
            instructions: None,
            options: None,
            question_type: None,
            answer_format: None,
            provenance: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Aksi 1: Edit
    AnswerEngineService::edit_answer(
        &db,
        EditAnswerInput {
            persona_id: persona.id.clone(),
            question_id: question.id.clone(),
            recommendation_id: None,
            actual_answer: "Sering olahraga".to_string(),
            reason: None,
        },
    )
    .unwrap();

    // Aksi 2: Copy
    AnswerEngineService::copy_answer(
        &db,
        CopyAnswerInput {
            persona_id: persona.id.clone(),
            question_id: question.id.clone(),
            recommendation_id: None,
            copied_text: "Sering olahraga".to_string(),
        },
    )
    .unwrap();

    // Aksi 3: Confirm
    AnswerEngineService::confirm_answer(
        &db,
        ConfirmRecommendationInput {
            persona_id: persona.id.clone(),
            question_id: question.id.clone(),
            recommendation_id: None,
            actual_answer: "Sering olahraga".to_string(),
        },
    )
    .unwrap();

    // Total history = 3
    let all_history = AnswerEngineService::list_history(
        &db,
        ListAnswerHistoryInput {
            persona_id: persona.id.clone(),
            question_id: Some(question.id.clone()),
            action: None,
            limit: None,
            offset: None,
        },
    )
    .unwrap();
    assert_eq!(all_history.len(), 3);

    // Filter confirmed only -> 1
    let confirmed_only = AnswerEngineService::list_history(
        &db,
        ListAnswerHistoryInput {
            persona_id: persona.id.clone(),
            question_id: Some(question.id.clone()),
            action: Some("confirmed".to_string()),
            limit: None,
            offset: None,
        },
    )
    .unwrap();
    assert_eq!(confirmed_only.len(), 1);
    assert_eq!(confirmed_only[0].action, ReviewAction::Confirmed);

    // Filter copied only -> 1
    let copied_only = AnswerEngineService::list_history(
        &db,
        ListAnswerHistoryInput {
            persona_id: persona.id,
            question_id: Some(question.id),
            action: Some("copied".to_string()),
            limit: None,
            offset: None,
        },
    )
    .unwrap();
    assert_eq!(copied_only.len(), 1);
    assert_eq!(copied_only[0].action, ReviewAction::Copied);
}
