use rusqlite::Connection;
use survey_copilot_lib::domain::survey::{
    AnswerFormat, ConfirmAnswerInput, ConfirmationStatus, CreateQuestionInput, CreateSessionInput,
    Provenance, QuestionType, SaveQuestionDraftInput, SessionStatus, UpdateSessionInput,
};
use survey_copilot_lib::migrations::MigrationRunner;
use survey_copilot_lib::repository::PersonaRepository;
use survey_copilot_lib::services::SurveyService;

fn setup_test_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("in-memory db must open");
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    let runner = MigrationRunner::with_app_migrations();
    runner.run(&mut conn).expect("migrations must succeed");
    conn
}

fn create_test_persona(conn: &Connection, name: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    PersonaRepository::create(conn, &id, name, Some("Profil responden uji"), true, &now)
        .expect("create persona must succeed");
    id
}

#[test]
fn test_migration_0005_compatibility_and_schema() {
    let conn = setup_test_db();

    // Verify columns exist on survey_sessions
    let mut stmt = conn
        .prepare("PRAGMA table_info(survey_sessions)")
        .expect("pragma table_info on survey_sessions must succeed");
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get(1))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    assert!(columns.contains(&"source_url".to_string()));
    assert!(columns.contains(&"external_survey_id".to_string()));
    assert!(columns.contains(&"language".to_string()));
    assert!(columns.contains(&"category".to_string()));
    assert!(columns.contains(&"fingerprint".to_string()));
    assert!(columns.contains(&"notes".to_string()));

    // Verify columns exist on survey_questions
    let mut stmt = conn
        .prepare("PRAGMA table_info(survey_questions)")
        .expect("pragma table_info on survey_questions must succeed");
    let q_columns: Vec<String> = stmt
        .query_map([], |row| row.get(1))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    assert!(q_columns.contains(&"persona_id".to_string()));
    assert!(q_columns.contains(&"question_type".to_string()));
    assert!(q_columns.contains(&"answer_format".to_string()));
    assert!(q_columns.contains(&"provenance".to_string()));
    assert!(q_columns.contains(&"observation_type".to_string()));
    assert!(q_columns.contains(&"confirmation_status".to_string()));
    assert!(q_columns.contains(&"user_answer_draft".to_string()));
    assert!(q_columns.contains(&"user_answer_confirmed".to_string()));
    assert!(q_columns.contains(&"confirmed_at".to_string()));
    assert!(q_columns.contains(&"preceding_question_id".to_string()));
    assert!(q_columns.contains(&"branch_logic".to_string()));
}

#[test]
fn test_session_crud_lifecycle() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Budi Santoso");

    // 1. Create Session
    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Perilaku Konsumen 2026".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: Some("https://surveys.example.com/s123".to_string()),
            external_survey_id: Some("EXT-9988".to_string()),
            language: Some("id".to_string()),
            category: Some("Consumer Goods".to_string()),
            notes: Some("Sesi survei tentang produk minuman".to_string()),
        },
    )
    .expect("create_session must succeed");

    assert_eq!(session.title, "Survei Perilaku Konsumen 2026");
    assert_eq!(session.platform_label.as_deref(), Some("Toluna"));
    assert_eq!(session.status, SessionStatus::Active);
    assert_eq!(session.persona_id, persona_id);

    // 2. Get Session Detail
    let fetched = SurveyService::get_session(&conn, &persona_id, &session.id)
        .expect("get_session must succeed");
    assert_eq!(fetched.id, session.id);
    assert_eq!(fetched.title, session.title);

    // 3. Update Session
    let updated = SurveyService::update_session(
        &conn,
        UpdateSessionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            title: "Survei Perilaku Konsumen 2026 - Diperbarui".to_string(),
            platform_label: Some("Toluna Quick".to_string()),
            source_url: session.source_url.clone(),
            external_survey_id: session.external_survey_id.clone(),
            language: Some("id".to_string()),
            category: Some("F&B".to_string()),
            notes: Some("Catatan diperbarui".to_string()),
            status: None,
        },
    )
    .expect("update_session must succeed");
    assert_eq!(updated.title, "Survei Perilaku Konsumen 2026 - Diperbarui");
    assert_eq!(updated.category.as_deref(), Some("F&B"));

    // 4. Archive Session
    SurveyService::archive_session(&conn, &persona_id, &session.id)
        .expect("archive_session must succeed");
    let archived = SurveyService::get_session(&conn, &persona_id, &session.id)
        .expect("get_session on archived must succeed");
    assert_eq!(archived.status, SessionStatus::Archived);
    assert!(archived.archived_at.is_some());

    // 5. List Sessions (exclude archived vs include archived)
    let active_list = SurveyService::list_sessions(&conn, &persona_id, false)
        .expect("list_sessions without archived must succeed");
    assert_eq!(active_list.len(), 0);

    let all_list = SurveyService::list_sessions(&conn, &persona_id, true)
        .expect("list_sessions with archived must succeed");
    assert_eq!(all_list.len(), 1);
    assert_eq!(all_list[0].status, SessionStatus::Archived);

    // 6. Restore Session
    SurveyService::restore_session(&conn, &persona_id, &session.id)
        .expect("restore_session must succeed");
    let restored = SurveyService::get_session(&conn, &persona_id, &session.id)
        .expect("get_session after restore must succeed");
    assert_eq!(restored.status, SessionStatus::Active);
    assert!(restored.archived_at.is_none());

    // 7. Delete Session
    SurveyService::delete_session(&conn, &persona_id, &session.id)
        .expect("delete_session must succeed");
    let deleted_result = SurveyService::get_session(&conn, &persona_id, &session.id);
    assert!(deleted_result.is_err());
}

#[test]
fn test_strict_persona_isolation() {
    let conn = setup_test_db();
    let persona_a = create_test_persona(&conn, "Persona A (Siti)");
    let persona_b = create_test_persona(&conn, "Persona B (Agus)");

    // Create session for Persona A
    let session_a = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_a.clone(),
            title: "Sesi Rahasia Persona A".to_string(),
            platform_label: Some("Platform A".to_string()),
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .expect("create session_a must succeed");

    // Create session for Persona B
    let session_b = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_b.clone(),
            title: "Sesi Rahasia Persona B".to_string(),
            platform_label: Some("Platform B".to_string()),
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .expect("create session_b must succeed");

    // Add Question to Session A
    let q_a = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_a.clone(),
            session_id: session_a.id.clone(),
            question_text_original: "Berapa penghasilan bulanan Anda?".to_string(),
            instructions: Some("Pilih satu".to_string()),
            options: Some(vec!["< 5jt".to_string(), "5-10jt".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .expect("create question for persona A must succeed");

    // Isolation Test 1: Persona B cannot list Persona A's sessions
    let list_b = SurveyService::list_sessions(&conn, &persona_b, true).unwrap();
    assert_eq!(list_b.len(), 1);
    assert_eq!(list_b[0].id, session_b.id);
    assert_ne!(list_b[0].id, session_a.id);

    // Isolation Test 2: Persona B cannot get Persona A's session
    let cross_session_get = SurveyService::get_session(&conn, &persona_b, &session_a.id);
    assert!(cross_session_get.is_err());

    // Isolation Test 3: Persona B cannot update Persona A's session
    let cross_update = SurveyService::update_session(
        &conn,
        UpdateSessionInput {
            persona_id: persona_b.clone(),
            session_id: session_a.id.clone(),
            title: "Hijacked Title".to_string(),
            platform_label: None,
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
            status: None,
        },
    );
    assert!(cross_update.is_err());

    // Isolation Test 4: Persona B cannot list Persona A's questions
    let cross_question_list = SurveyService::list_questions(&conn, &persona_b, &session_a.id);
    assert!(cross_question_list.is_err());

    // Isolation Test 5: Persona B cannot access Persona A's question by ID
    let cross_question_get = SurveyService::get_question(&conn, &persona_b, &q_a.id);
    assert!(cross_question_get.is_err());

    // Isolation Test 6: Persona B cannot create a question inside Persona A's session
    let cross_create_question = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_b.clone(),
            session_id: session_a.id.clone(),
            question_text_original: "Pertanyaan Ilegal".to_string(),
            instructions: None,
            options: None,
            question_type: None,
            answer_format: None,
            provenance: None,
            question_order: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    );
    assert!(cross_create_question.is_err());

    // Isolation Test 7: Persona B cannot delete Persona A's session
    let cross_delete = SurveyService::delete_session(&conn, &persona_b, &session_a.id);
    assert!(cross_delete.is_err());
}

#[test]
fn test_question_persistence_and_ordering() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Dewi Lestari");

    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Minuman Ringan".to_string(),
            platform_label: Some("YouGov".to_string()),
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    // Add Q1 (order should automatically be 1)
    let q1 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            question_text_original: "Apakah Anda mengonsumsi teh kemasan dalam 7 hari terakhir?"
                .to_string(),
            instructions: Some("Pilih Ya atau Tidak".to_string()),
            options: Some(vec!["Ya".to_string(), "Tidak".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::Boolean),
            provenance: Some(Provenance::UserEntered),
            question_order: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();
    assert_eq!(q1.question_order, 1);
    assert_eq!(q1.options.len(), 2);
    assert_eq!(q1.options[0], "Ya");

    // Add Q2 (order should automatically be 2)
    let q2 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            question_text_original: "Merek teh mana saja yang pernah Anda beli?".to_string(),
            instructions: Some("Pilih semua yang sesuai".to_string()),
            options: Some(vec![
                "Teh Botol Sosro".to_string(),
                "Teh Pucuk Harum".to_string(),
                "Nu Green Tea".to_string(),
                "Lainnya".to_string(),
            ]),
            question_type: Some(QuestionType::MultipleChoice),
            answer_format: Some(AnswerFormat::MultipleSelection),
            provenance: Some(Provenance::UserEntered),
            question_order: None,
            preceding_question_id: Some(q1.id.clone()),
            branch_logic: Some("Jika Q1 = Ya".to_string()),
        },
    )
    .unwrap();
    assert_eq!(q2.question_order, 2);
    assert_eq!(q2.preceding_question_id.as_deref(), Some(q1.id.as_str()));
    assert_eq!(q2.branch_logic.as_deref(), Some("Jika Q1 = Ya"));

    // Add Q3
    let q3 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            question_text_original: "Berapa botol rata-rata per minggu?".to_string(),
            instructions: None,
            options: None,
            question_type: Some(QuestionType::Numeric),
            answer_format: Some(AnswerFormat::Number),
            provenance: Some(Provenance::UserEntered),
            question_order: None,
            preceding_question_id: Some(q2.id.clone()),
            branch_logic: None,
        },
    )
    .unwrap();
    assert_eq!(q3.question_order, 3);

    // List questions and verify strictly ordered sequence
    let q_list = SurveyService::list_questions(&conn, &persona_id, &session.id).unwrap();
    assert_eq!(q_list.len(), 3);
    assert_eq!(q_list[0].id, q1.id);
    assert_eq!(q_list[0].question_order, 1);
    assert_eq!(q_list[1].id, q2.id);
    assert_eq!(q_list[1].question_order, 2);
    assert_eq!(q_list[2].id, q3.id);
    assert_eq!(q_list[2].question_order, 3);
}

#[test]
fn test_draft_persistence_and_actual_answer_separation() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Rina Melati");

    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Kebutuhan Rumah Tangga".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    let question = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            question_text_original: "Sebutkan deterjen favorit keluarga Anda:".to_string(),
            instructions: None,
            options: Some(vec![
                "Rinso".to_string(),
                "Attack".to_string(),
                "Daia".to_string(),
            ]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // 1. User types a draft answer
    SurveyService::save_question_draft(
        &conn,
        SaveQuestionDraftInput {
            persona_id: persona_id.clone(),
            question_id: question.id.clone(),
            user_answer_draft: "Saya sedang mempertimbangkan Rinso karena wanginya tahan lama"
                .to_string(),
        },
    )
    .expect("save_question_draft must succeed");

    // Verify draft is stored in database
    let fetched_draft = SurveyService::get_question(&conn, &persona_id, &question.id).unwrap();
    assert_eq!(
        fetched_draft.user_answer_draft.as_deref(),
        Some("Saya sedang mempertimbangkan Rinso karena wanginya tahan lama")
    );
    assert_eq!(fetched_draft.confirmation_status, ConfirmationStatus::Draft);
    assert!(fetched_draft.user_answer_confirmed.is_none());

    // 2. User confirms their actual answer
    SurveyService::confirm_question_answer(
        &conn,
        ConfirmAnswerInput {
            persona_id: persona_id.clone(),
            question_id: question.id.clone(),
            user_answer: "Rinso".to_string(),
        },
    )
    .expect("confirm_question_answer must succeed");

    // Verify confirmed answer is stored separately from draft and status is Confirmed
    let fetched_confirmed = SurveyService::get_question(&conn, &persona_id, &question.id).unwrap();
    assert_eq!(
        fetched_confirmed.user_answer_confirmed.as_deref(),
        Some("Rinso")
    );
    assert_eq!(
        fetched_confirmed.user_answer_draft.as_deref(),
        Some("Saya sedang mempertimbangkan Rinso karena wanginya tahan lama")
    );
    assert_eq!(
        fetched_confirmed.confirmation_status,
        ConfirmationStatus::Confirmed
    );
    assert!(fetched_confirmed.confirmed_at.is_some());
}

#[test]
fn test_provider_failure_preserves_question_and_draft() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Hendra Wijaya");

    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Investasi Keuangan".to_string(),
            platform_label: Some("Lifepoints".to_string()),
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    let question = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            question_text_original: "Berapa persen alokasi tabungan untuk instrumen reksa dana?"
                .to_string(),
            instructions: Some("Isi dalam persen 0-100".to_string()),
            options: None,
            question_type: Some(QuestionType::Numeric),
            answer_format: Some(AnswerFormat::Number),
            provenance: Some(Provenance::UserEntered),
            question_order: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // User types draft
    SurveyService::save_question_draft(
        &conn,
        SaveQuestionDraftInput {
            persona_id: persona_id.clone(),
            question_id: question.id.clone(),
            user_answer_draft: "Sekitar 20%".to_string(),
        },
    )
    .unwrap();

    // Simulate an AI provider failure (e.g. RateLimited, QuotaExhausted, Timeout)
    // The application must preserve the user's question, options, instructions, and draft answer completely intact.
    let preserved_question = SurveyService::get_question(&conn, &persona_id, &question.id).unwrap();
    assert_eq!(
        preserved_question.question_text_original,
        "Berapa persen alokasi tabungan untuk instrumen reksa dana?"
    );
    assert_eq!(
        preserved_question.instructions.as_deref(),
        Some("Isi dalam persen 0-100")
    );
    assert_eq!(
        preserved_question.user_answer_draft.as_deref(),
        Some("Sekitar 20%")
    );
    assert_eq!(
        preserved_question.confirmation_status,
        ConfirmationStatus::Draft
    );
}

#[test]
fn test_session_cascade_deletion() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Fajar Nugraha");

    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Sesi Uji Hapus".to_string(),
            platform_label: None,
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    let q1 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            question_text_original: "Pertanyaan 1".to_string(),
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
    .unwrap();

    // Delete session
    SurveyService::delete_session(&conn, &persona_id, &session.id).unwrap();

    // Verify session deleted
    assert!(SurveyService::get_session(&conn, &persona_id, &session.id).is_err());

    // Verify child questions automatically cascaded (0 orphan questions)
    let q_check: Option<String> = conn
        .query_row(
            "SELECT id FROM survey_questions WHERE id = ?1",
            [&q1.id],
            |row| row.get(0),
        )
        .ok();
    assert!(
        q_check.is_none(),
        "Child questions must cascade delete when session is deleted"
    );
}
