use std::sync::Arc;

use survey_copilot_lib::db::Database;
use survey_copilot_lib::domain::answer_engine::{
    ConfirmRecommendationInput, CopyAnswerInput, EditAnswerInput, EvidenceStatus,
    GenerateAnswerInput, ListAnswerHistoryInput, RawAnswerRecommendation, ReviewAction,
};
use survey_copilot_lib::domain::classifier::{ClassifyQuestionInput, QuestionCategory};
use survey_copilot_lib::domain::evidence::{
    ConflictType, EvidenceConfidence, EvidencePackage, EvidenceRecommendation,
    EvidenceRetrievalInput, SensitivityConsent,
};
use survey_copilot_lib::domain::fact::CreateFactInput;
use survey_copilot_lib::domain::persona::CreatePersonaInput;
use survey_copilot_lib::domain::survey::{
    AnswerFormat, ConfirmationStatus, CreateQuestionInput, CreateSessionInput, QuestionType,
};
use survey_copilot_lib::infrastructure::credentials::in_memory::InMemoryCredentialStore;
use survey_copilot_lib::infrastructure::http::MockHttpTransport;
use survey_copilot_lib::repository::{AnswerRepository, SurveyRepository};
use survey_copilot_lib::services::answer_engine::validator::validate_recommendation;
use survey_copilot_lib::services::{
    AnswerEngineService, EvidenceEngineService, FactService, PersonaService, ProviderPoolService,
    QuestionClassifierService, SurveyService,
};

fn create_test_db() -> Database {
    Database::initialize(std::path::Path::new(":memory:"))
        .expect("In-memory database should initialize")
}

#[test]
fn test_p3_e2e_full_workflow_from_session_to_review_and_history() {
    let db = create_test_db();
    let credential_store = InMemoryCredentialStore::new();
    let mock_transport = Arc::new(MockHttpTransport::new());
    let pool_service = ProviderPoolService::new();

    // 1. SETUP PERSONA
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Siti Rahmawati"))
        .expect("Persona should be created");

    // 2. SETUP FACTS (Ordinary Fact & Sensitive Fact)
    let ordinary_fact = FactService::create(
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
            source_title: Some("Ijazah S1".to_string()),
            source_description: "Pendidikan terakhir terkonfirmasi".to_string(),
            reference_uri: None,
            observed_at: Some("2026-01-01".to_string()),
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".to_string()),
            sensitivity: Some("ordinary".to_string()),
        },
    )
    .expect("Ordinary fact should be created");

    let sensitive_fact = FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "financial_economic".to_string(),
            fact_key: Some("monthly_income".to_string()),
            attribute: "monthly_income".to_string(),
            data_type: Some("number".to_string()),
            value: "8500000".to_string(),
            unit: Some("IDR".to_string()),
            source_type: Some("user_declaration".to_string()),
            source_title: Some("Slip Gaji".to_string()),
            source_description: "Penghasilan bulanan bersih".to_string(),
            reference_uri: None,
            observed_at: Some("2026-01-01".to_string()),
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".to_string()),
            sensitivity: Some("sensitive".to_string()),
        },
    )
    .expect("Sensitive fact should be created");

    // 3. SC-030: SURVEY SESSION CREATION (Survey Memory Foundation)
    let session = {
        let conn = db.connection.lock().unwrap();
        SurveyService::create_session(
            &conn,
            CreateSessionInput {
                persona_id: persona.id.clone(),
                title: "Survei Demografi dan Konsumsi 2026".to_string(),
                platform_label: Some("Platform Nusainsight".to_string()),
                source_url: Some("https://survey.nusainsight.id/s/demo-2026".to_string()),
                external_survey_id: Some("EXT-SURVEY-9921".to_string()),
                language: Some("id".to_string()),
                category: Some("Demografi & Ekonomi".to_string()),
                notes: Some("Sesi survei resmi".to_string()),
            },
        )
        .expect("Session should be created")
    };

    assert_eq!(session.title, "Survei Demografi dan Konsumsi 2026");
    assert_eq!(
        session.platform_label.as_deref(),
        Some("Platform Nusainsight")
    );
    assert_eq!(
        session.source_url.as_deref(),
        Some("https://survey.nusainsight.id/s/demo-2026")
    );
    assert_eq!(
        session.external_survey_id.as_deref(),
        Some("EXT-SURVEY-9921")
    );

    // 4. SC-030: QUESTION CREATION WITH ORDER SEQUENCING
    let q1 = {
        let conn = db.connection.lock().unwrap();
        SurveyService::create_question(
            &conn,
            CreateQuestionInput {
                persona_id: persona.id.clone(),
                session_id: session.id.clone(),
                capture_id: None,
                question_text_original: "Apakah tingkat pendidikan formal terakhir Anda?"
                    .to_string(),
                instructions: Some("Pilih satu jawaban yang paling sesuai.".to_string()),
                options: Some(vec![
                    "SMA / Sederajat".to_string(),
                    "Diploma (D3)".to_string(),
                    "Sarjana (S1)".to_string(),
                    "Pascasarjana (S2/S3)".to_string(),
                ]),
                question_type: Some(QuestionType::SingleChoice),
                answer_format: Some(AnswerFormat::SingleChoice),
                provenance: None,
                question_order: None, // Harusnya otomatis dihitung 1
                preceding_question_id: None,
                branch_logic: None,
            },
        )
        .expect("Question 1 should be created")
    };
    assert_eq!(q1.question_order, 1);

    let q2 = {
        let conn = db.connection.lock().unwrap();
        SurveyService::create_question(
            &conn,
            CreateQuestionInput {
                persona_id: persona.id.clone(),
                session_id: session.id.clone(),
                capture_id: None,
                question_text_original: "Berapa perkiraan pendapatan bulanan bersih Anda?"
                    .to_string(),
                instructions: Some("Isi dalam nominal Rupiah.".to_string()),
                options: None,
                question_type: Some(QuestionType::Numeric),
                answer_format: Some(AnswerFormat::Number),
                provenance: None,
                question_order: None, // Harusnya otomatis dihitung 2
                preceding_question_id: Some(q1.id.clone()),
                branch_logic: None,
            },
        )
        .expect("Question 2 should be created")
    };
    assert_eq!(q2.question_order, 2);
    assert_eq!(q2.preceding_question_id, Some(q1.id.clone()));

    // 5. SC-031: QUESTION CLASSIFIER & CONSTRAINT EXTRACTION
    let classification_q1 = QuestionClassifierService::classify(&ClassifyQuestionInput {
        persona_id: Some(persona.id.clone()),
        session_id: Some(session.id.clone()),
        question_id: Some(q1.id.clone()),
        question_text: q1.question_text_original.clone(),
        instructions: q1.instructions.clone(),
        options: Some(q1.options.clone()),
    });
    assert_eq!(
        classification_q1.primary_category,
        QuestionCategory::EmploymentEducation
    );
    assert_eq!(classification_q1.answer_format, AnswerFormat::SingleChoice);
    assert_eq!(classification_q1.question_type, QuestionType::SingleChoice);

    let classification_q2 = QuestionClassifierService::classify(&ClassifyQuestionInput {
        persona_id: Some(persona.id.clone()),
        session_id: Some(session.id.clone()),
        question_id: Some(q2.id.clone()),
        question_text: q2.question_text_original.clone(),
        instructions: q2.instructions.clone(),
        options: None,
    });
    assert_eq!(
        classification_q2.primary_category,
        QuestionCategory::FinancialEconomic
    );
    assert_eq!(classification_q2.answer_format, AnswerFormat::Number);

    // 6. SC-032: EVIDENCE RETRIEVAL & CONSISTENCY EVALUATION
    // A. Pengujian tanpa consent data sensitif: Fakta gaji (sensitive) harus ditahan
    let evidence_q2_no_consent = {
        let conn = db.connection.lock().unwrap();
        EvidenceEngineService::evaluate_evidence(
            &conn,
            &EvidenceRetrievalInput {
                persona_id: persona.id.clone(),
                session_id: Some(session.id.clone()),
                question_id: Some(q2.id.clone()),
                question_text: q2.question_text_original.clone(),
                instructions: q2.instructions.clone(),
                options: None,
                classification: Some(classification_q2.clone()),
                consent: Some(SensitivityConsent {
                    allow_sensitive: false,
                    allow_highly_sensitive: false,
                }),
                max_evidence_items: Some(5),
            },
        )
        .expect("Evidence evaluation without consent should succeed")
    };
    assert_eq!(evidence_q2_no_consent.retrieved_evidence.len(), 0);
    assert_eq!(
        evidence_q2_no_consent.recommendation,
        EvidenceRecommendation::InsufficientEvidence
    );
    assert!(evidence_q2_no_consent
        .missing_information
        .iter()
        .any(|msg| msg.contains("sensitif")));

    // B. Pengujian dengan consent data sensitif: Fakta gaji harus muncul
    let evidence_q2_with_consent = {
        let conn = db.connection.lock().unwrap();
        EvidenceEngineService::evaluate_evidence(
            &conn,
            &EvidenceRetrievalInput {
                persona_id: persona.id.clone(),
                session_id: Some(session.id.clone()),
                question_id: Some(q2.id.clone()),
                question_text: q2.question_text_original.clone(),
                instructions: q2.instructions.clone(),
                options: None,
                classification: Some(classification_q2.clone()),
                consent: Some(SensitivityConsent {
                    allow_sensitive: true,
                    allow_highly_sensitive: false,
                }),
                max_evidence_items: Some(5),
            },
        )
        .expect("Evidence evaluation with consent should succeed")
    };
    assert_eq!(evidence_q2_with_consent.retrieved_evidence.len(), 1);
    assert_eq!(
        evidence_q2_with_consent.retrieved_evidence[0].fact_id,
        sensitive_fact.id
    );
    assert_eq!(
        evidence_q2_with_consent.recommendation,
        EvidenceRecommendation::Supported
    );

    // 7. SC-033: STRUCTURED AI ANSWER ENGINE (Evidence-Grounded Recommendation)
    let answer_rec_q1 = AnswerEngineService::generate_recommendation(
        &db,
        &credential_store,
        mock_transport.clone(),
        &pool_service,
        GenerateAnswerInput {
            persona_id: persona.id.clone(),
            session_id: Some(session.id.clone()),
            question_id: Some(q1.id.clone()),
            question_text: q1.question_text_original.clone(),
            instructions: q1.instructions.clone(),
            options: Some(q1.options.clone()),
            consent: None,
            provider_config_id: None,
            prompt_version: None,
        },
        None,
    )
    .expect("Answer recommendation should be generated");

    assert_eq!(answer_rec_q1.status, EvidenceStatus::Supported);
    assert_eq!(answer_rec_q1.draft_answer, Some("Sarjana (S1)".to_string()));
    assert_eq!(answer_rec_q1.supporting_facts.len(), 1);
    assert_eq!(answer_rec_q1.supporting_facts[0].fact_id, ordinary_fact.id);
    assert!(answer_rec_q1.rejected_citations.is_empty());

    // 8. SC-034: REVIEW ACTIONS AND AUDIT HISTORY
    // A. Edit Action
    let edit_result = AnswerEngineService::edit_answer(
        &db,
        EditAnswerInput {
            persona_id: persona.id.clone(),
            question_id: q1.id.clone(),
            recommendation_id: Some(answer_rec_q1.id.clone()),
            actual_answer: "Sarjana (S1) - Teknik Informatika".to_string(),
            reason: Some("Menambahkan spesialisasi jurusan".to_string()),
        },
    )
    .expect("Edit answer action should succeed");

    assert_eq!(edit_result.action, ReviewAction::Edited);
    assert_eq!(edit_result.draft_answer, Some("Sarjana (S1)".to_string()));
    assert_eq!(
        edit_result.actual_answer,
        "Sarjana (S1) - Teknik Informatika"
    );
    assert_eq!(edit_result.review_status, "edited");

    // B. Copy Action
    let copy_result = AnswerEngineService::copy_answer(
        &db,
        CopyAnswerInput {
            persona_id: persona.id.clone(),
            question_id: q1.id.clone(),
            recommendation_id: Some(answer_rec_q1.id.clone()),
            copied_text: "Sarjana (S1) - Teknik Informatika".to_string(),
        },
    )
    .expect("Copy answer action should succeed");
    assert_eq!(copy_result.action, ReviewAction::Copied);

    // C. Confirm Action (CRITICAL BOUNDARY: Internal approval only; zero auto-submit; zero profile fact promotion)
    let facts_count_before = {
        let conn = db.connection.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM profile_facts WHERE persona_id = ?1",
            rusqlite::params![persona.id],
            |r| r.get::<_, i64>(0),
        )
        .unwrap()
    };

    let confirm_result = AnswerEngineService::confirm_answer(
        &db,
        ConfirmRecommendationInput {
            persona_id: persona.id.clone(),
            question_id: q1.id.clone(),
            recommendation_id: Some(answer_rec_q1.id.clone()),
            actual_answer: "Sarjana (S1) - Teknik Informatika".to_string(),
        },
    )
    .expect("Confirm answer action should succeed");

    assert_eq!(confirm_result.action, ReviewAction::Confirmed);
    assert_eq!(confirm_result.review_status, "confirmed");

    // Pastikan status pertanyaan terkonfirmasi di database
    let stored_q1 = {
        let conn = db.connection.lock().unwrap();
        SurveyRepository::get_question(&conn, &persona.id, &q1.id)
            .unwrap()
            .unwrap()
    };
    assert_eq!(stored_q1.confirmation_status, ConfirmationStatus::Confirmed);
    assert_eq!(
        stored_q1.user_answer_confirmed,
        Some("Sarjana (S1) - Teknik Informatika".to_string())
    );
    assert!(stored_q1.confirmed_at.is_some());

    // Pastikan tidak ada fakta profil baru yang terbuat (NO auto-promotion to profile facts)
    let facts_count_after = {
        let conn = db.connection.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM profile_facts WHERE persona_id = ?1",
            rusqlite::params![persona.id],
            |r| r.get::<_, i64>(0),
        )
        .unwrap()
    };
    assert_eq!(facts_count_before, facts_count_after);

    // 9. SC-034: AUDIT HISTORY TIMELINE & REVISIONS INSPECTION
    let history_items = AnswerEngineService::list_history(
        &db,
        ListAnswerHistoryInput {
            persona_id: persona.id.clone(),
            question_id: Some(q1.id.clone()),
            action: None,
            limit: None,
            offset: None,
        },
    )
    .expect("History list should be retrieved");

    // Harus memuat 3 aksi: confirmed (terbaru), copied, edited
    assert_eq!(history_items.len(), 3);
    assert_eq!(history_items[0].action, ReviewAction::Confirmed);
    assert_eq!(history_items[1].action, ReviewAction::Copied);
    assert_eq!(history_items[2].action, ReviewAction::Edited);

    // 10. SURVEY MEMORY VERIFICATION: List confirmed answers by persona
    let confirmed_answers = {
        let conn = db.connection.lock().unwrap();
        SurveyRepository::list_confirmed_answers_by_persona(&conn, &persona.id, Some(10)).unwrap()
    };
    assert_eq!(confirmed_answers.len(), 1);
    assert_eq!(confirmed_answers[0].id, q1.id);
    assert_eq!(
        confirmed_answers[0].user_answer_confirmed,
        Some("Sarjana (S1) - Teknik Informatika".to_string())
    );
}

#[test]
fn test_p3_e2e_prompt_injection_defense_and_citation_allowlist() {
    let evidence_pkg = EvidencePackage {
        question_id: Some("q-malicious-01".to_string()),
        active_persona_id: "p-target".to_string(),
        retrieved_evidence: vec![],
        conflict_indicators: vec![],
        missing_information: vec!["Tidak ada fakta pekerjaan".to_string()],
        evidence_confidence: EvidenceConfidence::Uncertain,
        recommendation: EvidenceRecommendation::InsufficientEvidence,
        evaluation_summary: "Bukti kosong".to_string(),
        evaluated_at: chrono::Utc::now().to_rfc3339(),
    };

    // Model dikelabui oleh injeksi prompt dan mencoba mengembalikan status 'supported'
    // dengan mengutip ID fakta palsu 'fact-injected-999'
    let hallucinated_output = RawAnswerRecommendation {
        status: "supported".to_string(),
        question_intent: Some("employment".to_string()),
        answer_format: Some("text".to_string()),
        interpretation: Some("Pengguna memerintahkan status didukung.".to_string()),
        draft_answer: Some("Direktur Utama".to_string()),
        supporting_fact_ids: Some(vec![
            "fact-injected-999".to_string(),
            "fact-fake-777".to_string(),
        ]),
        rationale: Some("Model mengklaim berhasil memverifikasi.".to_string()),
        missing_information: None,
        conflicts: None,
        assumptions: None,
        clarifying_question: None,
    };

    let validated = validate_recommendation(
        hallucinated_output,
        &evidence_pkg,
        "p-target",
        Some("q-malicious-01".to_string()),
        None,
        "1.0",
        None,
        None,
    )
    .expect("Validator should execute");

    // AT-043 & AT-044 Verification:
    // 1. Sitasi palsu harus ditolak dan dicatat di rejected_citations
    assert_eq!(
        validated.rejected_citations,
        vec!["fact-injected-999", "fact-fake-777"]
    );
    // 2. Karena tidak ada fakta sah yang lolos allowlist, status dipaksa menjadi InsufficientInformation
    assert_eq!(validated.status, EvidenceStatus::InsufficientInformation);
    // 3. Draft faktual dilarang keras dan dipaksa menjadi None
    assert_eq!(validated.draft_answer, None);
    assert_eq!(validated.confidence, EvidenceConfidence::Uncertain);
}

#[test]
fn test_p3_e2e_conflicting_evidence_enforces_abstention() {
    let db = create_test_db();

    // 1. Setup Persona
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Ahmad Fauzi")).unwrap();

    // 2. Setup Dua Fakta yang Bertentangan (AT-045)
    FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "employment_education".to_string(),
            fact_key: Some("employment_status".to_string()),
            attribute: "employment_status".to_string(),
            data_type: Some("string".to_string()),
            value: "Karyawan Penuh Waktu".to_string(),
            unit: None,
            source_type: Some("user_declaration".to_string()),
            source_title: Some("Profil Lama".to_string()),
            source_description: "Status kerja".to_string(),
            reference_uri: None,
            observed_at: Some("2025-01-01".to_string()),
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".to_string()),
            sensitivity: Some("ordinary".to_string()),
        },
    )
    .unwrap();

    FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "employment_education".to_string(),
            fact_key: Some("employment_status".to_string()),
            attribute: "employment_status".to_string(),
            data_type: Some("string".to_string()),
            value: "Wirausaha Mandiri".to_string(),
            unit: None,
            source_type: Some("user_declaration".to_string()),
            source_title: Some("Profil Baru".to_string()),
            source_description: "Status kerja".to_string(),
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

    // 3. Evaluasi Bukti Lokal
    let conn = db.connection.lock().unwrap();
    let evidence = EvidenceEngineService::evaluate_evidence(
        &conn,
        &EvidenceRetrievalInput {
            persona_id: persona.id.clone(),
            session_id: None,
            question_id: None,
            question_text: "Apakah status pekerjaan Anda saat ini?".to_string(),
            instructions: None,
            options: Some(vec!["Karyawan".to_string(), "Wirausaha".to_string()]),
            classification: None,
            consent: None,
            max_evidence_items: Some(5),
        },
    )
    .unwrap();

    // AT-045: Konflik kritis terdeteksi secara deterministik
    assert_eq!(
        evidence.recommendation,
        EvidenceRecommendation::NeedsClarification
    );
    assert!(evidence
        .conflict_indicators
        .iter()
        .any(|c| c.conflict_type == ConflictType::FactConflict));

    drop(conn);

    // 4. Generate Answer Engine
    let credential_store = InMemoryCredentialStore::new();
    let mock_transport = Arc::new(MockHttpTransport::new());
    let pool_service = ProviderPoolService::new();

    let rec = AnswerEngineService::generate_recommendation(
        &db,
        &credential_store,
        mock_transport,
        &pool_service,
        GenerateAnswerInput {
            persona_id: persona.id.clone(),
            session_id: None,
            question_id: None,
            question_text: "Apakah status pekerjaan Anda saat ini?".to_string(),
            instructions: None,
            options: Some(vec!["Karyawan".to_string(), "Wirausaha".to_string()]),
            consent: None,
            provider_config_id: None,
            prompt_version: None,
        },
        None,
    )
    .unwrap();

    assert_eq!(rec.status, EvidenceStatus::ConflictingEvidence);
    assert_eq!(rec.draft_answer, None);
    assert!(rec.clarifying_question.is_some());
    assert_eq!(rec.confidence, EvidenceConfidence::Uncertain);
}

#[test]
fn test_p3_e2e_strict_cross_persona_isolation_across_all_layers() {
    let db = create_test_db();

    // Buat Persona A dan Persona B
    let persona_a = PersonaService::create(&db, CreatePersonaInput::new("Persona A")).unwrap();
    let persona_b = PersonaService::create(&db, CreatePersonaInput::new("Persona B")).unwrap();

    // Persona A memiliki sesi, pertanyaan, dan fakta
    let fact_a = FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona_a.id.clone(),
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

    let session_a = {
        let conn = db.connection.lock().unwrap();
        SurveyService::create_session(
            &conn,
            CreateSessionInput {
                persona_id: persona_a.id.clone(),
                title: "Sesi Persona A".to_string(),
                platform_label: Some("Platform A".to_string()),
                source_url: None,
                external_survey_id: None,
                language: None,
                category: None,
                notes: None,
            },
        )
        .unwrap()
    };

    let q_a = {
        let conn = db.connection.lock().unwrap();
        SurveyService::create_question(
            &conn,
            CreateQuestionInput {
                persona_id: persona_a.id.clone(),
                session_id: session_a.id.clone(),
                capture_id: None,
                question_text_original: "Di kota manakah Anda tinggal?".to_string(),
                instructions: None,
                options: None,
                question_type: None,
                answer_format: None,
                provenance: None,
                question_order: None,
                preceding_question_id: None,
                branch_logic: None,
            },
        )
        .unwrap()
    };

    let credential_store = InMemoryCredentialStore::new();
    let mock_transport = Arc::new(MockHttpTransport::new());
    let pool_service = ProviderPoolService::new();

    let rec_a = AnswerEngineService::generate_recommendation(
        &db,
        &credential_store,
        mock_transport,
        &pool_service,
        GenerateAnswerInput {
            persona_id: persona_a.id.clone(),
            session_id: Some(session_a.id.clone()),
            question_id: Some(q_a.id.clone()),
            question_text: q_a.question_text_original.clone(),
            instructions: None,
            options: None,
            consent: None,
            provider_config_id: None,
            prompt_version: None,
        },
        None,
    )
    .unwrap();

    // UJI ISOLASI: Persona B tidak boleh dapat mengakses data Persona A
    let conn = db.connection.lock().unwrap();

    // 1. Sesi Persona A tidak dapat diakses oleh Persona B
    let get_session_b = SurveyService::get_session(&conn, &persona_b.id, &session_a.id);
    assert!(get_session_b.is_err());

    // 2. Pertanyaan Persona A tidak dapat diakses oleh Persona B
    let get_q_b = SurveyService::get_question(&conn, &persona_b.id, &q_a.id);
    assert!(get_q_b.is_err());

    // 3. Rekomendasi Persona A tidak dapat diakses oleh Persona B
    let rec_b =
        AnswerRepository::get_recommendation_by_id(&conn, &persona_b.id, &rec_a.id).unwrap();
    assert!(rec_b.is_none());

    // 4. Riwayat Persona A tidak bocor ke Persona B
    let history_b =
        AnswerRepository::list_history(&conn, &persona_b.id, None, None, None, None).unwrap();
    assert!(history_b.is_empty());

    // 5. Fakta Persona A tidak dapat di-retrieval oleh Persona B
    let evidence_b = EvidenceEngineService::evaluate_evidence(
        &conn,
        &EvidenceRetrievalInput {
            persona_id: persona_b.id.clone(),
            session_id: None,
            question_id: None,
            question_text: "Di kota manakah Anda tinggal?".to_string(),
            instructions: None,
            options: None,
            classification: None,
            consent: None,
            max_evidence_items: Some(10),
        },
    )
    .unwrap();
    assert!(evidence_b.retrieved_evidence.is_empty());
    assert_ne!(
        evidence_b
            .retrieved_evidence
            .iter()
            .map(|f| &f.fact_id)
            .collect::<Vec<_>>(),
        vec![&fact_a.id]
    );

    drop(conn);

    // 6. Aksi review oleh Persona B pada pertanyaan Persona A ditolak
    let edit_b = AnswerEngineService::edit_answer(
        &db,
        EditAnswerInput {
            persona_id: persona_b.id.clone(),
            question_id: q_a.id.clone(),
            recommendation_id: Some(rec_a.id.clone()),
            actual_answer: "Jakarta".to_string(),
            reason: None,
        },
    );
    assert!(edit_b.is_err());
}
