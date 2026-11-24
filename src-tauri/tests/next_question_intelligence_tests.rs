use rusqlite::Connection;
use survey_copilot_lib::domain::survey::{
    AnswerFormat, ConfirmAnswerInput, CreateQuestionInput, CreateSessionInput,
    PredictNextQuestionInput, PredictionStatus, Provenance, QuestionType,
    SurveyBranchTransition, TransitionType,
};
use survey_copilot_lib::migrations::MigrationRunner;
use survey_copilot_lib::repository::{PersonaRepository, SurveyRepository};
use survey_copilot_lib::services::{NextQuestionService, SurveyService};

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
        Some("Profil responden uji NQI"),
        is_active,
        &now,
    )
    .expect("create persona must succeed");
    id
}

#[test]
fn test_migration_0012_schema_and_tables() {
    let conn = setup_test_db();

    // Verify survey_branch_transitions table exists
    let count_transitions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='survey_branch_transitions'",
            [],
            |r| r.get(0),
        )
        .expect("query sqlite_master for survey_branch_transitions must succeed");
    assert_eq!(count_transitions, 1, "survey_branch_transitions table must exist");

    // Verify survey_question_predictions table exists
    let count_predictions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='survey_question_predictions'",
            [],
            |r| r.get(0),
        )
        .expect("query sqlite_master for survey_question_predictions must succeed");
    assert_eq!(count_predictions, 1, "survey_question_predictions table must exist");

    // Verify columns on survey_branch_transitions
    let mut stmt = conn
        .prepare("PRAGMA table_info(survey_branch_transitions)")
        .expect("pragma table_info on survey_branch_transitions must succeed");
    let transition_cols: Vec<String> = stmt
        .query_map([], |row| row.get(1))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    assert!(transition_cols.contains(&"from_question_id".to_string()));
    assert!(transition_cols.contains(&"from_question_fingerprint".to_string()));
    assert!(transition_cols.contains(&"from_answer_value".to_string()));
    assert!(transition_cols.contains(&"to_question_text".to_string()));
    assert!(transition_cols.contains(&"transition_type".to_string()));
    assert!(transition_cols.contains(&"branch_condition".to_string()));

    // Verify columns on survey_question_predictions
    let mut stmt = conn
        .prepare("PRAGMA table_info(survey_question_predictions)")
        .expect("pragma table_info on survey_question_predictions must succeed");
    let pred_cols: Vec<String> = stmt
        .query_map([], |row| row.get(1))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    assert!(pred_cols.contains(&"current_question_id".to_string()));
    assert!(pred_cols.contains(&"predicted_question_text".to_string()));
    assert!(pred_cols.contains(&"confidence_score".to_string()));
    assert!(pred_cols.contains(&"evidence_source".to_string()));
    assert!(pred_cols.contains(&"leading_condition".to_string()));
    assert!(pred_cols.contains(&"status".to_string()));
}

#[test]
fn test_sequential_question_sequence_modeling() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Budi Santoso", true);

    // Buat sesi survei
    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Minat Olahraga 2026".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: Some("https://survey.toluna.com/sport1".to_string()),
            external_survey_id: None,
            language: Some("id".to_string()),
            category: Some("lifestyle".to_string()),
            notes: None,
        },
    )
    .expect("create session must succeed");

    // Buat Pertanyaan 1, 2, 3 secara sekuensial
    let q1 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            capture_id: None,
            question_text_original: "Berapa usia Anda saat ini?".to_string(),
            instructions: None,
            options: Some(vec!["18-25".to_string(), "26-35".to_string(), "36+".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .expect("create q1 must succeed");

    let q2 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            capture_id: None,
            question_text_original: "Apakah Anda rutin berolahraga setiap minggu?".to_string(),
            instructions: None,
            options: Some(vec!["Ya".to_string(), "Tidak".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(2),
            preceding_question_id: Some(q1.id.clone()),
            branch_logic: None,
        },
    )
    .expect("create q2 must succeed");

    let q3 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            capture_id: None,
            question_text_original: "Jenis olahraga apa yang paling sering Anda lakukan?".to_string(),
            instructions: None,
            options: Some(vec!["Lari".to_string(), "Renang".to_string(), "Sepeda".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(3),
            preceding_question_id: Some(q2.id.clone()),
            branch_logic: None,
        },
    )
    .expect("create q3 must succeed");

    // Sinkronkan dan ambil transisi sesi
    let synced = NextQuestionService::sync_session_transitions(&conn, &persona_id, &session.id)
        .expect("sync transitions must succeed");
    assert!(synced >= 2, "Harus mencatat minimal 2 transisi sekuensial");

    let transitions = NextQuestionService::get_session_branch_transitions(&conn, &persona_id, &session.id)
        .expect("get transitions must succeed");

    assert_eq!(transitions.len(), 2);
    assert_eq!(transitions[0].from_question_id, q1.id);
    assert_eq!(transitions[0].to_question_id, Some(q2.id.clone()));
    assert_eq!(transitions[0].transition_type, TransitionType::Sequential);

    assert_eq!(transitions[1].from_question_id, q2.id);
    assert_eq!(transitions[1].to_question_id, Some(q3.id.clone()));
    assert_eq!(transitions[1].transition_type, TransitionType::Sequential);
}

#[test]
fn test_conditional_branch_transitions_and_accuracy_matching_at_065() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Dewi Lestari", true);

    // Sesi 1: Responden menjawab 'Ya' -> Lanjut ke Q2A
    let session1 = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Kepemilikan Kendaraan Sesi 1".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: Some("https://survey.toluna.com/car1".to_string()),
            external_survey_id: None,
            language: Some("id".to_string()),
            category: Some("automotive".to_string()),
            notes: None,
        },
    )
    .unwrap();

    let q1_s1 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session1.id.clone(),
            capture_id: None,
            question_text_original: "Apakah Anda memiliki kendaraan bermotor?".to_string(),
            instructions: None,
            options: Some(vec!["Ya".to_string(), "Tidak".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Responden konfirmasi jawaban 'Ya'
    SurveyService::confirm_question_answer(
        &conn,
        ConfirmAnswerInput {
            persona_id: persona_id.clone(),
            question_id: q1_s1.id.clone(),
            user_answer: "Ya".to_string(),
        },
    )
    .unwrap();

    let _q2a_s1 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session1.id.clone(),
            capture_id: None,
            question_text_original: "Berapa jumlah kendaraan yang Anda miliki?".to_string(),
            instructions: None,
            options: Some(vec!["1".to_string(), "2".to_string(), "3+".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(2),
            preceding_question_id: Some(q1_s1.id.clone()),
            branch_logic: Some("Jika memiliki kendaraan".to_string()),
        },
    )
    .unwrap();

    // Sesi 2: Responden menjawab 'Tidak' -> Lanjut ke Q2B
    let session2 = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Kepemilikan Kendaraan Sesi 2".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: Some("https://survey.toluna.com/car1".to_string()),
            external_survey_id: None,
            language: Some("id".to_string()),
            category: Some("automotive".to_string()),
            notes: None,
        },
    )
    .unwrap();

    let q1_s2 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session2.id.clone(),
            capture_id: None,
            question_text_original: "Apakah Anda memiliki kendaraan bermotor?".to_string(),
            instructions: None,
            options: Some(vec!["Ya".to_string(), "Tidak".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Responden konfirmasi jawaban 'Tidak'
    SurveyService::confirm_question_answer(
        &conn,
        ConfirmAnswerInput {
            persona_id: persona_id.clone(),
            question_id: q1_s2.id.clone(),
            user_answer: "Tidak".to_string(),
        },
    )
    .unwrap();

    let _q2b_s2 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session2.id.clone(),
            capture_id: None,
            question_text_original: "Apakah Anda berencana membeli kendaraan dalam 12 bulan ke depan?".to_string(),
            instructions: None,
            options: Some(vec!["Ya".to_string(), "Tidak".to_string(), "Mungkin".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(2),
            preceding_question_id: Some(q1_s2.id.clone()),
            branch_logic: Some("Jika tidak memiliki kendaraan".to_string()),
        },
    )
    .unwrap();

    // Sesi 3 (Aktif): Pengguna berada pada pertanyaan Q1 yang sama, dan memilih jawaban 'Ya'
    let session3 = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Kepemilikan Kendaraan Sesi 3".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: Some("https://survey.toluna.com/car1".to_string()),
            external_survey_id: None,
            language: Some("id".to_string()),
            category: Some("automotive".to_string()),
            notes: None,
        },
    )
    .unwrap();

    let q1_s3 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session3.id.clone(),
            capture_id: None,
            question_text_original: "Apakah Anda memiliki kendaraan bermotor?".to_string(),
            instructions: None,
            options: Some(vec!["Ya".to_string(), "Tidak".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Evaluasi prediksi dengan jawaban aktif = 'Ya'
    let nqi_result_ya = NextQuestionService::predict_next_questions(
        &conn,
        &PredictNextQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session3.id.clone(),
            current_question_id: q1_s3.id.clone(),
            current_answer: Some("Ya".to_string()),
        },
    )
    .expect("predict next questions must succeed");

    assert_eq!(nqi_result_ya.status, PredictionStatus::PredictedNotDetected);
    assert_eq!(nqi_result_ya.predictions.len(), 2);

    // Kandidat #1 harus Q2A ("Berapa jumlah kendaraan yang Anda miliki?") dengan keyakinan tinggi (>= 0.85)
    let top_pred = &nqi_result_ya.predictions[0];
    assert!(top_pred.candidate_question_text.contains("jumlah kendaraan"));
    assert!(top_pred.confidence >= 0.85, "Confidence untuk jawaban yang cocok harus tinggi");
    assert!(top_pred.is_predicted_not_detected, "AT-065: Wajib berlabel is_predicted_not_detected");
    assert_eq!(top_pred.evidence_source, "observed_branch");

    // Kandidat #2 harus cabang alternatif dengan keyakinan lebih rendah (< 0.50)
    let alt_pred = &nqi_result_ya.predictions[1];
    assert!(alt_pred.candidate_question_text.contains("berencana membeli kendaraan"));
    assert!(alt_pred.confidence < 0.50, "Confidence untuk cabang alternatif harus lebih rendah");
    assert!(alt_pred.is_predicted_not_detected);

    // Evaluasi prediksi jika pengguna memilih jawaban 'Tidak'
    let nqi_result_tidak = NextQuestionService::predict_next_questions(
        &conn,
        &PredictNextQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session3.id.clone(),
            current_question_id: q1_s3.id.clone(),
            current_answer: Some("Tidak".to_string()),
        },
    )
    .unwrap();

    assert_eq!(nqi_result_tidak.predictions[0].candidate_question_text, "Apakah Anda berencana membeli kendaraan dalam 12 bulan ke depan?");
    assert!(nqi_result_tidak.predictions[0].confidence >= 0.85);
}

#[test]
fn test_unanswered_question_returns_all_candidates_with_conditions() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Citra Kirana", true);

    // Setup riwayat percabangan
    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Belanja Online".to_string(),
            platform_label: Some("Toluna".to_string()),
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
            capture_id: None,
            question_text_original: "Apakah Anda pernah berbelanja di e-commerce?".to_string(),
            instructions: None,
            options: Some(vec!["Pernah".to_string(), "Belum Pernah".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Catat transisi 1: jika 'pernah' -> 'Platform apa yang paling sering digunakan?'
    let trans1 = SurveyBranchTransition {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: session.id.clone(),
        persona_id: persona_id.clone(),
        from_question_id: q1.id.clone(),
        from_question_fingerprint: q1.question_fingerprint.clone(),
        from_question_order: 1,
        from_answer_value: Some("pernah".to_string()),
        to_question_id: None,
        to_question_fingerprint: None,
        to_question_text: "Platform e-commerce apa yang paling sering Anda gunakan?".to_string(),
        to_question_type: QuestionType::SingleChoice,
        to_options: vec!["Tokopedia".to_string(), "Shopee".to_string()],
        to_question_order: 2,
        branch_condition: Some("Jawaban: pernah".to_string()),
        transition_type: TransitionType::ConditionalBranch,
        observed_at: chrono::Utc::now().to_rfc3339(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    SurveyRepository::record_branch_transition(&conn, &trans1).unwrap();

    // Catat transisi 2: jika 'belum pernah' -> 'Apa alasan utama Anda belum berbelanja online?'
    let trans2 = SurveyBranchTransition {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: session.id.clone(),
        persona_id: persona_id.clone(),
        from_question_id: q1.id.clone(),
        from_question_fingerprint: q1.question_fingerprint.clone(),
        from_question_order: 1,
        from_answer_value: Some("belum pernah".to_string()),
        to_question_id: None,
        to_question_fingerprint: None,
        to_question_text: "Apa alasan utama Anda belum berbelanja secara online?".to_string(),
        to_question_type: QuestionType::SingleChoice,
        to_options: vec!["Khawatir penipuan".to_string(), "Lebih suka belanja langsung".to_string()],
        to_question_order: 2,
        branch_condition: Some("Jawaban: belum pernah".to_string()),
        transition_type: TransitionType::ConditionalBranch,
        observed_at: chrono::Utc::now().to_rfc3339(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    SurveyRepository::record_branch_transition(&conn, &trans2).unwrap();

    // Evaluasi tanpa jawaban aktif (current_answer = None)
    let nqi_result = NextQuestionService::predict_next_questions(
        &conn,
        &PredictNextQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            current_question_id: q1.id.clone(),
            current_answer: None,
        },
    )
    .unwrap();

    assert_eq!(nqi_result.status, PredictionStatus::PredictedNotDetected);
    assert_eq!(nqi_result.predictions.len(), 2);

    for pred in &nqi_result.predictions {
        assert!(pred.is_predicted_not_detected, "Harus berlabel tegas not yet detected");
        assert!(pred.leading_condition.is_some(), "Harus menyertakan kondisi percabangan");
        assert!(pred.confidence >= 0.60 && pred.confidence <= 0.85);
    }
}

#[test]
fn test_nqi_sparse_or_empty_history_returns_insufficient_evidence_at_065() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Eko Prabowo", true);

    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Baru Tanpa Riwayat".to_string(),
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
            capture_id: None,
            question_text_original: "Apakah Anda memiliki hewan ternak di pekarangan rumah?".to_string(),
            instructions: None,
            options: None,
            question_type: None,
            answer_format: None,
            provenance: None,
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Pertanyaan ini belum memiliki riwayat transisi sama sekali
    let nqi_result = NextQuestionService::predict_next_questions(
        &conn,
        &PredictNextQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            current_question_id: q1.id.clone(),
            current_answer: None,
        },
    )
    .expect("predict should complete safely");

    // AT-065 & Non-negotiable Rule 5
    assert_eq!(nqi_result.status, PredictionStatus::InsufficientEvidence);
    assert_eq!(nqi_result.predictions.len(), 0, "Dilarang mempublikasikan prediksi spekulatif");
    assert_eq!(nqi_result.observed_transitions_count, 0);
    assert!(nqi_result.explanation.contains("tidak membuat prediksi spekulatif"));
}

#[test]
fn test_nqi_question_detected_lifecycle_transition_matched_and_diverged() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Fajar Nugraha", true);

    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Transportasi Publik".to_string(),
            platform_label: Some("YouGov".to_string()),
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
            capture_id: None,
            question_text_original: "Moda transportasi apa yang Anda gunakan hari ini?".to_string(),
            instructions: None,
            options: Some(vec!["KRL / Kereta".to_string(), "Kendaraan Pribadi".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Catat transisi historis:
    // Cabang A: "Stasiun mana stasiun keberangkatan Anda?"
    // Cabang B: "Berapa liter bensin yang Anda habiskan?"
    let trans_a = SurveyBranchTransition {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: session.id.clone(),
        persona_id: persona_id.clone(),
        from_question_id: q1.id.clone(),
        from_question_fingerprint: q1.question_fingerprint.clone(),
        from_question_order: 1,
        from_answer_value: Some("krl / kereta".to_string()),
        to_question_id: None,
        to_question_fingerprint: None,
        to_question_text: "Stasiun mana stasiun keberangkatan Anda?".to_string(),
        to_question_type: QuestionType::SingleChoice,
        to_options: vec!["Manggarai".to_string(), "Tanah Abang".to_string()],
        to_question_order: 2,
        branch_condition: Some("Jawaban: KRL".to_string()),
        transition_type: TransitionType::ConditionalBranch,
        observed_at: chrono::Utc::now().to_rfc3339(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    SurveyRepository::record_branch_transition(&conn, &trans_a).unwrap();

    let trans_b = SurveyBranchTransition {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: session.id.clone(),
        persona_id: persona_id.clone(),
        from_question_id: q1.id.clone(),
        from_question_fingerprint: q1.question_fingerprint.clone(),
        from_question_order: 1,
        from_answer_value: Some("kendaraan pribadi".to_string()),
        to_question_id: None,
        to_question_fingerprint: None,
        to_question_text: "Berapa liter bensin yang Anda habiskan hari ini?".to_string(),
        to_question_type: QuestionType::Numeric,
        to_options: vec![],
        to_question_order: 2,
        branch_condition: Some("Jawaban: Kendaraan Pribadi".to_string()),
        transition_type: TransitionType::ConditionalBranch,
        observed_at: chrono::Utc::now().to_rfc3339(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    SurveyRepository::record_branch_transition(&conn, &trans_b).unwrap();

    // Jalankan prediksi saat pengguna memilih 'KRL / Kereta'
    let _pred_result = NextQuestionService::predict_next_questions(
        &conn,
        &PredictNextQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            current_question_id: q1.id.clone(),
            current_answer: Some("KRL / Kereta".to_string()),
        },
    )
    .unwrap();

    // Periksa status pending pada audit log prediksi
    let pending_before = SurveyRepository::find_pending_predictions_for_question(&conn, &persona_id, &q1.id).unwrap();
    assert_eq!(pending_before.len(), 2);
    assert_eq!(pending_before[0].status, PredictionStatus::PredictedNotDetected);

    // Pengguna sekarang memajukan survei dan OCR mendeteksi pertanyaan aktual:
    // "Stasiun mana stasiun keberangkatan Anda?"
    let q2_actual = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            capture_id: None,
            question_text_original: "Stasiun mana stasiun keberangkatan Anda?".to_string(),
            instructions: None,
            options: Some(vec!["Manggarai".to_string(), "Tanah Abang".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::OcrDetected),
            question_order: Some(2),
            preceding_question_id: Some(q1.id.clone()),
            branch_logic: None,
        },
    )
    .unwrap();

    // Verifikasi bahwa status prediksi terbarui:
    // - Prediksi yang cocok menjadi 'detected_matched' dan tertaut ke q2_actual.id
    // - Prediksi yang diverged menjadi 'detected_diverged'
    let all_preds = NextQuestionService::get_session_predictions(&conn, &persona_id, &session.id).unwrap();
    assert_eq!(all_preds.len(), 2);

    let matched = all_preds.iter().find(|p| p.predicted_question_text.contains("Stasiun mana")).unwrap();
    assert_eq!(matched.status, PredictionStatus::DetectedMatched);
    assert_eq!(matched.matched_question_id, Some(q2_actual.id.clone()));

    let diverged = all_preds.iter().find(|p| p.predicted_question_text.contains("liter bensin")).unwrap();
    assert_eq!(diverged.status, PredictionStatus::DetectedDiverged);
    assert_eq!(diverged.matched_question_id, None);
}

#[test]
fn test_skip_jump_transition_modeling() {
    let conn = setup_test_db();
    let persona_id = create_test_persona(&conn, "Gunawan Wibowo", true);

    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_id.clone(),
            title: "Survei Penyaringan Kriteria".to_string(),
            platform_label: None,
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    // Pertanyaan 1 (Order 1)
    let q1 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            capture_id: None,
            question_text_original: "Apakah Anda berdomisili di wilayah Jabodetabek?".to_string(),
            instructions: None,
            options: Some(vec!["Ya".to_string(), "Tidak".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // Pengguna menjawab 'Tidak', survei langsung lompat ke Pertanyaan 5 (Order 5)
    SurveyService::confirm_question_answer(
        &conn,
        ConfirmAnswerInput {
            persona_id: persona_id.clone(),
            question_id: q1.id.clone(),
            user_answer: "Tidak".to_string(),
        },
    )
    .unwrap();

    let q5 = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_id.clone(),
            session_id: session.id.clone(),
            capture_id: None,
            question_text_original: "Terima kasih atas partisipasi Anda, survei telah selesai.".to_string(),
            instructions: None,
            options: None,
            question_type: Some(QuestionType::Other),
            answer_format: Some(AnswerFormat::Unknown),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(5),
            preceding_question_id: Some(q1.id.clone()),
            branch_logic: Some("Lompat karena domisili luar Jabodetabek".to_string()),
        },
    )
    .unwrap();

    let transitions = NextQuestionService::get_session_branch_transitions(&conn, &persona_id, &session.id).unwrap();
    assert_eq!(transitions.len(), 1);

    let jump_trans = &transitions[0];
    assert_eq!(jump_trans.from_question_id, q1.id);
    assert_eq!(jump_trans.to_question_id, Some(q5.id));
    assert_eq!(jump_trans.transition_type, TransitionType::SkipJump);
    assert!(jump_trans.branch_condition.as_ref().unwrap().contains("Lompat 3 pertanyaan"));
}

#[test]
fn test_strict_persona_isolation_at_066() {
    let conn = setup_test_db();
    let persona_a = create_test_persona(&conn, "Persona A (Rahasia)", true);
    let persona_b = create_test_persona(&conn, "Persona B (Baru)", false);

    // Persona A memiliki sesi survei dengan data transisi dan jawaban pribadi
    let session_a = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_a.clone(),
            title: "Survei Finansial Pribadi A".to_string(),
            platform_label: Some("Toluna".to_string()),
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    let q1_a = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_a.clone(),
            session_id: session_a.id.clone(),
            capture_id: None,
            question_text_original: "Berapa penghasilan bulanan Anda?".to_string(),
            instructions: None,
            options: Some(vec!["< 10 Juta".to_string(), ">= 10 Juta".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    SurveyService::confirm_question_answer(
        &conn,
        ConfirmAnswerInput {
            persona_id: persona_a.clone(),
            question_id: q1_a.id.clone(),
            user_answer: ">= 10 Juta (Jawaban Pribadi A)".to_string(),
        },
    )
    .unwrap();

    let _q2_a = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_a.clone(),
            session_id: session_a.id.clone(),
            capture_id: None,
            question_text_original: "Instrumen investasi apa yang Anda gunakan?".to_string(),
            instructions: None,
            options: Some(vec!["Saham".to_string(), "Reksadana".to_string()]),
            question_type: Some(QuestionType::SingleChoice),
            answer_format: Some(AnswerFormat::SingleChoice),
            provenance: Some(Provenance::UserEntered),
            question_order: Some(2),
            preceding_question_id: Some(q1_a.id.clone()),
            branch_logic: None,
        },
    )
    .unwrap();

    // Verifikasi Persona B tidak bisa mengakses data transisi Persona A
    let trans_b = NextQuestionService::get_session_branch_transitions(&conn, &persona_b, &session_a.id).unwrap();
    assert_eq!(trans_b.len(), 0, "Transisi sesi Persona A tidak boleh dibaca oleh Persona B");

    // Persona B membuat sesi sendiri
    let session_b = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_b.clone(),
            title: "Survei Berbeda Persona B".to_string(),
            platform_label: None,
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .unwrap();

    let q1_b = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_b.clone(),
            session_id: session_b.id.clone(),
            capture_id: None,
            question_text_original: "Apakah Anda menyukai film komedi?".to_string(),
            instructions: None,
            options: None,
            question_type: None,
            answer_format: None,
            provenance: None,
            question_order: Some(1),
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .unwrap();

    // NQI untuk Persona B tidak boleh membocorkan jawaban atau transisi finansial Persona A
    let nqi_b = NextQuestionService::predict_next_questions(
        &conn,
        &PredictNextQuestionInput {
            persona_id: persona_b.clone(),
            session_id: session_b.id.clone(),
            current_question_id: q1_b.id.clone(),
            current_answer: None,
        },
    )
    .unwrap();

    assert_eq!(nqi_b.status, PredictionStatus::InsufficientEvidence);
    assert_eq!(nqi_b.predictions.len(), 0);
    assert_eq!(nqi_b.persona_id, persona_b);
}
