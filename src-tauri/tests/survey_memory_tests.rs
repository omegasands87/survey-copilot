use rusqlite::Connection;
use survey_copilot_lib::domain::survey::{
    compute_question_fingerprint, compute_session_structural_fingerprint, extract_domain_from_url,
    AnswerFormat, ConfirmAnswerInput, ConfirmationStatus, CreateQuestionInput, CreateSessionInput,
    Provenance, QuestionType, RecognizeSurveyInput, SaveQuestionDraftInput,
    SurveyRecognitionStatus,
};
use survey_copilot_lib::migrations::MigrationRunner;
use survey_copilot_lib::repository::PersonaRepository;
use survey_copilot_lib::services::{SurveyRecognitionService, SurveyService};

fn setup_test_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("in-memory db must open");
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    let runner = MigrationRunner::with_app_migrations();
    runner.run(&mut conn).expect("migrations must succeed");
    conn
}

fn create_test_persona(conn: &Connection, name: &str, is_active: bool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    PersonaRepository::create(
        conn,
        &id,
        name,
        Some("Profil responden uji"),
        is_active,
        &now,
    )
    .expect("create persona must succeed");
    id
}

#[test]
fn test_migration_0011_schema_and_columns() {
    let conn = setup_test_db();

    // Verify columns on survey_sessions
    let mut stmt = conn
        .prepare("PRAGMA table_info(survey_sessions)")
        .expect("pragma table_info on survey_sessions must succeed");
    let session_cols: Vec<String> = stmt
        .query_map([], |row| row.get(1))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    assert!(session_cols.contains(&"source_domain".to_string()));
    assert!(session_cols.contains(&"version".to_string()));
    assert!(session_cols.contains(&"recognized_from_id".to_string()));

    // Verify columns on survey_questions
    let mut stmt = conn
        .prepare("PRAGMA table_info(survey_questions)")
        .expect("pragma table_info on survey_questions must succeed");
    let question_cols: Vec<String> = stmt
        .query_map([], |row| row.get(1))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    assert!(question_cols.contains(&"question_fingerprint".to_string()));

    // Verify survey_recognitions table exists
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='survey_recognitions'",
            [],
            |r| r.get(0),
        )
        .expect("query sqlite_master");
    assert_eq!(count, 1);
}

#[test]
fn test_survey_metadata_and_domain_extraction() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Dewi Sartika", true);

    // Test URL domain extraction helper
    assert_eq!(
        extract_domain_from_url("https://surveys.yougov.com/v2/start?id=123"),
        Some("surveys.yougov.com".to_string())
    );
    assert_eq!(
        extract_domain_from_url("http://toluna.com/survey/456"),
        Some("toluna.com".to_string())
    );
    assert_eq!(extract_domain_from_url(""), None);

    // Create session and verify domain is automatically extracted
    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Perbankan Digital 2026".to_string(),
            platform_label: Some("YouGov".to_string()),
            source_url: Some("https://surveys.yougov.com/entry/bank2026".to_string()),
            external_survey_id: Some("YG-BANK-01".to_string()),
            language: Some("id".to_string()),
            category: Some("Keuangan".to_string()),
            notes: Some("Sesi survei penggunaan aplikasi mobile banking".to_string()),
        },
    )
    .expect("create_session must succeed");

    assert_eq!(
        session.source_domain,
        Some("surveys.yougov.com".to_string())
    );
    assert_eq!(session.version, 1);
    assert_eq!(session.recognized_from_id, None);
}

#[test]
fn test_structural_fingerprint_deterministic_and_sensitive_to_sequence() {
    let q1_options = vec!["BCA".to_string(), "Mandiri".to_string(), "BRI".to_string()];
    let fp1 = compute_question_fingerprint(
        "Bank apa yang paling sering Anda gunakan?",
        QuestionType::SingleChoice,
        &q1_options,
    );
    let fp1_repeat = compute_question_fingerprint(
        "Bank apa yang paling sering Anda gunakan?",
        QuestionType::SingleChoice,
        &q1_options,
    );
    assert_eq!(
        fp1, fp1_repeat,
        "Fingerprint pertanyaan harus deterministik"
    );

    // Perubahan opsi mengubah fingerprint
    let q1_options_modified = vec!["BCA".to_string(), "Mandiri".to_string(), "BNI".to_string()];
    let fp1_modified = compute_question_fingerprint(
        "Bank apa yang paling sering Anda gunakan?",
        QuestionType::SingleChoice,
        &q1_options_modified,
    );
    assert_ne!(
        fp1, fp1_modified,
        "Perubahan opsi harus mengubah fingerprint"
    );

    // Fingerprint sesi
    let questions_a = vec![
        (
            1,
            "Pertanyaan 1",
            QuestionType::SingleChoice,
            q1_options.as_slice(),
        ),
        (2, "Pertanyaan 2", QuestionType::Numeric, [].as_slice()),
    ];
    let session_fp_a = compute_session_structural_fingerprint(&questions_a);

    let questions_b = vec![
        (
            1,
            "Pertanyaan 1",
            QuestionType::SingleChoice,
            q1_options.as_slice(),
        ),
        (2, "Pertanyaan 2", QuestionType::Numeric, [].as_slice()),
    ];
    let session_fp_b = compute_session_structural_fingerprint(&questions_b);
    assert_eq!(
        session_fp_a, session_fp_b,
        "Fingerprint sesi harus identik untuk urutan pertanyaan yang sama"
    );

    // Urutan terbalik atau pertanyaan berbeda menghasilkan fingerprint berbeda
    let questions_reversed = vec![
        (1, "Pertanyaan 2", QuestionType::Numeric, [].as_slice()),
        (
            2,
            "Pertanyaan 1",
            QuestionType::SingleChoice,
            q1_options.as_slice(),
        ),
    ];
    let session_fp_rev = compute_session_structural_fingerprint(&questions_reversed);
    assert_ne!(
        session_fp_a, session_fp_rev,
        "Urutan pertanyaan yang berbeda harus menghasilkan fingerprint berbeda"
    );
}

#[test]
fn test_ordered_question_sequence_persistence() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Ahmad Dahlan", true);

    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Kepuasan Pelanggan".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: None,
            external_survey_id: None,
            language: Some("id".to_string()),
            category: None,
            notes: None,
        },
    )
    .unwrap();

    // Tambahkan 3 pertanyaan secara berurutan
    let q1 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            capture_id: None,
            question_text_original: "Berapa usia Anda saat ini?".to_string(),
            instructions: Some("Tuliskan dalam angka".to_string()),
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
    assert_eq!(q1.question_order, 1);
    assert!(q1.question_fingerprint.is_some());

    let q2 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            capture_id: None,
            question_text_original: "Apa status pekerjaan utama Anda?".to_string(),
            instructions: Some("Pilih satu jawaban".to_string()),
            options: Some(vec![
                "Karyawan Swasta".to_string(),
                "Wirausaha".to_string(),
                "Pelajar".to_string(),
            ]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: None,
            preceding_question_id: Some(q1.id.clone()),
            branch_logic: None,
        },
    )
    .unwrap();
    assert_eq!(q2.question_order, 2);

    let q3 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            capture_id: None,
            question_text_original: "Apakah Anda memiliki tabungan perbankan?".to_string(),
            instructions: None,
            options: Some(vec!["Ya".to_string(), "Tidak".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::Boolean),
            provenance: Some(Provenance::UserEntered),
            question_order: None,
            preceding_question_id: Some(q2.id.clone()),
            branch_logic: None,
        },
    )
    .unwrap();
    assert_eq!(q3.question_order, 3);

    // Ambil daftar seluruh pertanyaan dan verifikasi keterurutan sekuensial
    let questions = SurveyService::list_questions(&conn, &persona_id, &session.id).unwrap();
    assert_eq!(questions.len(), 3);
    assert_eq!(questions[0].question_order, 1);
    assert_eq!(questions[1].question_order, 2);
    assert_eq!(questions[2].question_order, 3);
    assert_eq!(
        questions[0].question_text_original,
        "Berapa usia Anda saat ini?"
    );
    assert_eq!(questions[1].options.len(), 3);
}

#[test]
fn test_strict_answer_separation_at_064() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Siti Nurhaliza", true);

    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Pola Belanja Online".to_string(),
            platform_label: Some("Milieu".to_string()),
            source_url: None,
            external_survey_id: None,
            language: Some("id".to_string()),
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
            capture_id: None,
            question_text_original: "Seberapa sering Anda berbelanja daring dalam sebulan?"
                .to_string(),
            instructions: None,
            options: Some(vec![
                "1-2 kali".to_string(),
                "3-5 kali".to_string(),
                ">5 kali".to_string(),
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

    // 1. Simpan draf jawaban sementara pengguna (auto-save)
    SurveyService::save_question_draft(
        &conn,
        SaveQuestionDraftInput {
            persona_id: persona_id.clone(),
            question_id: question.id.clone(),
            user_answer_draft: "3-5 kali (sementara)".to_string(),
        },
    )
    .unwrap();

    let q_after_draft = SurveyService::get_question(&conn, &persona_id, &question.id).unwrap();
    assert_eq!(
        q_after_draft.user_answer_draft,
        Some("3-5 kali (sementara)".to_string())
    );
    assert_eq!(
        q_after_draft.user_answer_confirmed, None,
        "Draf tidak boleh mengotori jawaban konfirmasi"
    );
    assert_eq!(q_after_draft.confirmation_status, ConfirmationStatus::Draft);

    // 2. Simpan rekomendasi AI ke answer_recommendations
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO answer_recommendations (
            id, question_id, status, draft_text, interpretation, rationale,
            missing_information_json, conflicts_json, assumptions_json, created_at
        ) VALUES (?1, ?2, 'ready', ?3, ?4, ?5, '[]', '[]', '[]', ?6)",
        rusqlite::params![
            "rec-123",
            question.id,
            "1-2 kali",
            "Frekuensi belanja online bulanan",
            "Berdasarkan profil responden yang jarang berbelanja online",
            now
        ],
    )
    .unwrap();

    // 3. Konfirmasi jawaban aktual nyata oleh responden (AT-064)
    SurveyService::confirm_question_answer(
        &conn,
        ConfirmAnswerInput {
            persona_id: persona_id.clone(),
            question_id: question.id.clone(),
            user_answer: "3-5 kali".to_string(),
        },
    )
    .unwrap();

    // 4. Ambil Survey Memory Session Detail dan verifikasi pemisahan mutlak
    let memory_detail =
        SurveyRecognitionService::get_memory_session_detail(&conn, &persona_id, &session.id)
            .expect("get_memory_session_detail must succeed");

    assert_eq!(memory_detail.questions.len(), 1);
    let mem_q = &memory_detail.questions[0];

    // AT-064: Jawaban aktual pengguna terkonfirmasi terpisah dari saran AI
    assert_eq!(mem_q.user_answer_confirmed, Some("3-5 kali".to_string()));
    assert!(mem_q.confirmed_at.is_some());
    assert_eq!(
        mem_q.user_answer_draft,
        Some("3-5 kali (sementara)".to_string())
    );
    assert_eq!(mem_q.ai_recommendation_draft, Some("1-2 kali".to_string()));
    assert_ne!(
        mem_q.user_answer_confirmed, mem_q.ai_recommendation_draft,
        "Jawaban aktual pengguna dan rekomendasi AI harus terpisah tegas"
    );
}

#[test]
fn test_survey_recognition_exact_match() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Bambang Pamungkas", true);

    // Sesi 1 (Sesi Historis yang pernah diselesaikan)
    let session_old = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Minat Olahraga 2026".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: Some("https://surveys.toluna.com/sports".to_string()),
            external_survey_id: Some("EXT-SP-01".to_string()),
            language: Some("id".to_string()),
            category: Some("Olahraga".to_string()),
            notes: None,
        },
    )
    .unwrap();

    let q1_old = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session_old.id.clone(),
            capture_id: None,
            question_text_original: "Seberapa sering Anda berolahraga setiap minggu?".to_string(),
            instructions: None,
            options: Some(vec![
                "1-2 kali".to_string(),
                "3-4 kali".to_string(),
                "Hampir setiap hari".to_string(),
            ]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Konfirmasi jawaban pada sesi lama
    SurveyService::confirm_question_answer(
        &conn,
        ConfirmAnswerInput {
            persona_id: persona_id.clone(),
            question_id: q1_old.id.clone(),
            user_answer: "3-4 kali".to_string(),
        },
    )
    .unwrap();

    // Sesi 2 (Survei baru yang sama persis muncul kembali)
    let session_new = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Minat Olahraga 2026".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: Some("https://surveys.toluna.com/sports".to_string()),
            external_survey_id: Some("EXT-SP-01".to_string()),
            language: Some("id".to_string()),
            category: Some("Olahraga".to_string()),
            notes: None,
        },
    )
    .unwrap();

    SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session_new.id.clone(),
            capture_id: None,
            question_text_original: "Seberapa sering Anda berolahraga setiap minggu?".to_string(),
            instructions: None,
            options: Some(vec![
                "1-2 kali".to_string(),
                "3-4 kali".to_string(),
                "Hampir setiap hari".to_string(),
            ]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Evaluasi pengenalan survei
    let eval = SurveyRecognitionService::evaluate_recognition(
        &conn,
        &RecognizeSurveyInput {
            persona_id: persona_id.clone(),
            session_id: session_new.id.clone(),
        },
    )
    .expect("evaluate_recognition must succeed");

    assert_eq!(eval.match_status, SurveyRecognitionStatus::ExactMatch);
    assert!(
        eval.confidence_score >= 0.95,
        "Exact match harus memiliki keyakinan >= 0.95"
    );
    assert_eq!(eval.matched_session_id, Some(session_old.id.clone()));
    assert_eq!(eval.matched_question_count, 1);
    assert_eq!(eval.recognized_answers.len(), 1);
    assert_eq!(eval.recognized_answers[0].confirmed_answer, "3-4 kali");
}

#[test]
fn test_survey_recognition_version_changed_differentiation() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Hendra Setiawan", true);

    // Sesi Versi 1 (3 pertanyaan)
    let session_v1 = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Transportasi Publik Jakarta".to_string(),
            platform_label: Some("Milieu".to_string()),
            source_url: Some("https://surveys.milieu.io/trans".to_string()),
            external_survey_id: Some("TRANS-01".to_string()),
            language: Some("id".to_string()),
            category: Some("Transportasi".to_string()),
            notes: None,
        },
    )
    .unwrap();

    let _q1 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session_v1.id.clone(),
            capture_id: None,
            question_text_original: "Moda transportasi apa yang paling sering Anda gunakan?"
                .to_string(),
            instructions: None,
            options: Some(vec![
                "KRL Commuter Line".to_string(),
                "MRT".to_string(),
                "TransJakarta".to_string(),
            ]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    let _q2 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session_v1.id.clone(),
            capture_id: None,
            question_text_original: "Berapa biaya transportasi harian Anda?".to_string(),
            instructions: None,
            options: None,
            question_type: Some(QuestionType::Numeric),
            answer_format: Some(AnswerFormat::Number),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(2),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Sesi Versi 2 (Ada 2 pertanyaan sama, ditambah 1 pertanyaan baru tentang LRT)
    let session_v2 = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Transportasi Publik Jakarta".to_string(),
            platform_label: Some("Milieu".to_string()),
            source_url: Some("https://surveys.milieu.io/trans".to_string()),
            external_survey_id: Some("TRANS-02".to_string()),
            language: Some("id".to_string()),
            category: Some("Transportasi".to_string()),
            notes: None,
        },
    )
    .unwrap();

    SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session_v2.id.clone(),
            capture_id: None,
            question_text_original: "Moda transportasi apa yang paling sering Anda gunakan?"
                .to_string(),
            instructions: None,
            options: Some(vec![
                "KRL Commuter Line".to_string(),
                "MRT".to_string(),
                "TransJakarta".to_string(),
            ]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session_v2.id.clone(),
            capture_id: None,
            question_text_original: "Berapa biaya transportasi harian Anda?".to_string(),
            instructions: None,
            options: None,
            question_type: Some(QuestionType::Numeric),
            answer_format: Some(AnswerFormat::Number),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(2),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Pertanyaan Baru pada Versi 2
    SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session_v2.id.clone(),
            capture_id: None,
            question_text_original: "Apakah Anda pernah mencoba LRT Jabodebek?".to_string(),
            instructions: None,
            options: Some(vec!["Pernah".to_string(), "Belum Pernah".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::Boolean),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(3),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Evaluasi pengenalan survei
    let eval = SurveyRecognitionService::evaluate_recognition(
        &conn,
        &RecognizeSurveyInput {
            persona_id: persona_id.clone(),
            session_id: session_v2.id.clone(),
        },
    )
    .unwrap();

    // Pastikan terdeteksi sebagai VersionChanged (bukan ExactMatch!)
    assert_eq!(
        eval.match_status,
        SurveyRecognitionStatus::VersionChanged,
        "Survei dengan pertanyaan bertambah harus dikenali sebagai VersionChanged"
    );
    assert_eq!(eval.matched_session_id, Some(session_v1.id.clone()));
    assert_eq!(eval.matched_question_count, 2);
    assert_eq!(eval.total_question_count, 3);
    assert!(
        !eval.differences.is_empty(),
        "Rincian perbedaan harus tercatat"
    );
}

#[test]
fn test_survey_recognition_title_alone_never_matches_definitely() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Ratna Sari", true);

    // Sesi 1
    let session_1 = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Konsumsi Kopi Indonesia 2026".to_string(),
            platform_label: Some("YouGov".to_string()),
            source_url: None,
            external_survey_id: None,
            language: Some("id".to_string()),
            category: None,
            notes: None,
        },
    )
    .unwrap();

    SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session_1.id.clone(),
            capture_id: None,
            question_text_original: "Berapa cangkir kopi yang Anda minum per hari?".to_string(),
            instructions: None,
            options: None,
            question_type: Some(QuestionType::Numeric),
            answer_format: Some(AnswerFormat::Number),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Sesi 2 memiliki judul persis sama, tapi pertanyaan tentang topik smartphone!
    let session_2 = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Konsumsi Kopi Indonesia 2026".to_string(),
            platform_label: Some("YouGov".to_string()),
            source_url: None,
            external_survey_id: None,
            language: Some("id".to_string()),
            category: None,
            notes: None,
        },
    )
    .unwrap();

    SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session_2.id.clone(),
            capture_id: None,
            question_text_original: "Merek smartphone apa yang sedang Anda gunakan saat ini?"
                .to_string(),
            instructions: None,
            options: Some(vec![
                "Samsung".to_string(),
                "Apple".to_string(),
                "Xiaomi".to_string(),
            ]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    let eval = SurveyRecognitionService::evaluate_recognition(
        &conn,
        &RecognizeSurveyInput {
            persona_id: persona_id.clone(),
            session_id: session_2.id.clone(),
        },
    )
    .unwrap();

    // Batasan Produk Non-Negotiable: Kemiripan judul semata tidak boleh menghasilkan ExactMatch atau LikelyMatch!
    assert_ne!(
        eval.match_status,
        SurveyRecognitionStatus::ExactMatch,
        "Kemiripan judul semata tidak boleh menghasilkan ExactMatch"
    );
    assert_ne!(
        eval.match_status,
        SurveyRecognitionStatus::LikelyMatch,
        "Kemiripan judul semata tidak boleh menghasilkan LikelyMatch"
    );
    assert!(
        eval.confidence_score <= 0.25,
        "Confidence score tanpa kecocokan struktur pertanyaan tidak boleh melebihi 0.25"
    );
    assert_eq!(eval.matched_question_count, 0);
}

#[test]
fn test_strict_persona_isolation_at_066() {
    let conn = setup_test_db();

    // Persona A: Budi
    let persona_a = create_test_persona(&conn, "Budi Hartono", true);
    // Persona B: Siti
    let persona_b = create_test_persona(&conn, "Siti Aminah", false);

    // Sesi milik Persona A
    let session_a = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_a.clone(),
            title: "Survei Kebiasaan Merokok & Kesehatan".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: Some("https://surveys.toluna.com/health".to_string()),
            external_survey_id: Some("HLTH-01".to_string()),
            language: Some("id".to_string()),
            category: Some("Kesehatan".to_string()),
            notes: None,
        },
    )
    .unwrap();

    let q_a = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_a.clone(),
            session_id: session_a.id.clone(),
            capture_id: None,
            question_text_original: "Apakah Anda merokok?".to_string(),
            instructions: None,
            options: Some(vec![
                "Ya, setiap hari".to_string(),
                "Tidak pernah".to_string(),
            ]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::Boolean),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Persona A mengkonfirmasi jawaban pribadi sensitif
    SurveyService::confirm_question_answer(
        &conn,
        ConfirmAnswerInput {
            persona_id: persona_a.clone(),
            question_id: q_a.id.clone(),
            user_answer: "Ya, setiap hari".to_string(),
        },
    )
    .unwrap();

    // Sekarang Persona B membuat survei yang sama
    let session_b = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_b.clone(),
            title: "Survei Kebiasaan Merokok & Kesehatan".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: Some("https://surveys.toluna.com/health".to_string()),
            external_survey_id: Some("HLTH-01".to_string()),
            language: Some("id".to_string()),
            category: Some("Kesehatan".to_string()),
            notes: None,
        },
    )
    .unwrap();

    SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_b.clone(),
            session_id: session_b.id.clone(),
            capture_id: None,
            question_text_original: "Apakah Anda merokok?".to_string(),
            instructions: None,
            options: Some(vec![
                "Ya, setiap hari".to_string(),
                "Tidak pernah".to_string(),
            ]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::Boolean),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Jalankan pengenalan survei untuk Persona B
    let eval_b = SurveyRecognitionService::evaluate_recognition(
        &conn,
        &RecognizeSurveyInput {
            persona_id: persona_b.clone(),
            session_id: session_b.id.clone(),
        },
    )
    .unwrap();

    // AT-066: Persona B TIDAK BOLEH mencocokkan sesi milik Persona A
    assert_ne!(
        eval_b.matched_session_id,
        Some(session_a.id.clone()),
        "AT-066 Violation: Sesi Persona A bocor ke evaluasi pengenalan Persona B!"
    );
    assert_eq!(
        eval_b.match_status,
        SurveyRecognitionStatus::NoMatch,
        "Persona B belum memiliki riwayat survei sendiri, sehingga harus NoMatch"
    );
    assert!(
        eval_b.recognized_answers.is_empty(),
        "AT-066 Violation: Jawaban pribadi terkonfirmasi Persona A bocor ke Persona B!"
    );

    // Verifikasi bahwa Persona B tidak dapat memanggil get_memory_session_detail untuk sesi milik Persona A
    let memory_leak_attempt =
        SurveyRecognitionService::get_memory_session_detail(&conn, &persona_b, &session_a.id);
    assert!(
        memory_leak_attempt.is_err(),
        "AT-066 Violation: Persona B berhasil mengakses Survey Memory milik Persona A!"
    );
}
