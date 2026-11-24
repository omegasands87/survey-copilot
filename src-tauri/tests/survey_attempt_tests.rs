use rusqlite::Connection;
use survey_copilot_lib::domain::attempt::{
    AttemptFilter, CreateAttemptInput, LearningDataStatus, OutcomeConfidence, OutcomeLabel,
    RecordOutcomeEventInput,
};
use survey_copilot_lib::domain::opportunity::{
    CriterionInput, CriterionOperator, CriterionType, OpportunityProvenance,
    SaveOpportunityWithCriteriaInput,
};
use survey_copilot_lib::migrations::MigrationRunner;
use survey_copilot_lib::services::opportunity_service::OpportunityService;
use survey_copilot_lib::services::outcome_learning_service::{
    OutcomeLearningService, MIN_SAMPLE_THRESHOLD,
};

fn setup_test_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("in-memory db should open");
    let runner = MigrationRunner::with_app_migrations();
    runner.run(&mut conn).expect("migrations should succeed");
    conn
}

fn create_test_persona(conn: &Connection, id: &str, name: &str, is_active: bool) {
    conn.execute(
        r#"
        INSERT INTO personas (id, display_name, is_active, status, created_at, updated_at)
        VALUES (?1, ?2, ?3, 'active', '2026-11-20T12:00:00Z', '2026-11-20T12:00:00Z')
        "#,
        rusqlite::params![id, name, if is_active { 1 } else { 0 }],
    )
    .expect("persona insertion should succeed");
}

/// 1. Pencatatan seluruh 9 status percobaan survei yang didukung (SC-052)
#[test]
fn test_record_all_nine_outcome_statuses() {
    let mut conn = setup_test_db();
    let persona_id = "persona-andi";
    create_test_persona(&conn, persona_id, "Andi Pratama", true);

    let statuses = vec![
        OutcomeLabel::Qualified,
        OutcomeLabel::ScreenedOut,
        OutcomeLabel::QuotaFull,
        OutcomeLabel::Completed,
        OutcomeLabel::QualityRejected,
        OutcomeLabel::TechnicalFailure,
        OutcomeLabel::AbandonedByUser,
        OutcomeLabel::Pending,
        OutcomeLabel::Unknown,
    ];

    assert_eq!(statuses.len(), 9);

    for (idx, status) in statuses.into_iter().enumerate() {
        let input = CreateAttemptInput {
            persona_id: persona_id.to_string(),
            opportunity_id: None,
            opportunity_title: Some(format!("Survei Uji Status #{}", idx + 1)),
            source_platform: Some("YouGov".to_string()),
            category: Some("Teknologi".to_string()),
            target_url: Some(format!("https://survey.example.com/{}", idx + 1)),
            raw_text: Some(format!("Teks survei #{}", idx + 1)),
            session_id: None,
            started_at: Some(format!("2026-11-2{}T10:00:00Z", idx + 1)),
            initial_outcome: Some(status),
            evidence_source: Some("pengujian_otomatis".to_string()),
            evidence_note: Some(format!("Observasi status {:?}", status)),
            confidence: Some(OutcomeConfidence::Observed),
        };

        let detail = OutcomeLearningService::record_attempt(&mut conn, input)
            .expect("pencatatan percobaan survei harus berhasil");

        assert_eq!(detail.current_outcome, status);
        assert_eq!(detail.persona_id, persona_id);
        assert_eq!(detail.events.len(), 1);
        assert_eq!(detail.events[0].outcome_label, status);

        // Jika status terminal, ended_at harus terisi otomatis
        if status.is_terminal() {
            assert!(detail.ended_at.is_some());
        }
    }
}

/// 2. Perubahan status bertahap dan kronologi events (State Transitions)
#[test]
fn test_attempt_status_transitions_and_events() {
    let mut conn = setup_test_db();
    let persona_id = "persona-siti";
    create_test_persona(&conn, persona_id, "Siti Rahma", true);

    // Langkah 1: Mulai survei dengan status pending
    let initial_input = CreateAttemptInput {
        persona_id: persona_id.to_string(),
        opportunity_id: None,
        opportunity_title: Some("Survei Konsumsi Kopi".to_string()),
        source_platform: Some("Milieu".to_string()),
        category: Some("FMCG".to_string()),
        target_url: Some("https://milieu.example.com/coffee".to_string()),
        raw_text: Some("Apakah Anda peminum kopi harian?".to_string()),
        session_id: None,
        started_at: Some("2026-11-20T08:00:00Z".to_string()),
        initial_outcome: Some(OutcomeLabel::Pending),
        evidence_source: Some("pengguna_buka_tautan".to_string()),
        evidence_note: Some("Memulai survei di peramban".to_string()),
        confidence: Some(OutcomeConfidence::Observed),
    };

    let created = OutcomeLearningService::record_attempt(&mut conn, initial_input)
        .expect("percobaan awal harus tersimpan");
    assert_eq!(created.current_outcome, OutcomeLabel::Pending);
    assert!(created.ended_at.is_none());
    assert_eq!(created.events.len(), 1);

    // Langkah 2: Lolos skrining kriteria awal (transisi ke qualified)
    let event1_input = RecordOutcomeEventInput {
        attempt_id: created.id.clone(),
        persona_id: persona_id.to_string(),
        outcome_label: OutcomeLabel::Qualified,
        evidence_source: "screener_passed_screen".to_string(),
        evidence_note: Some("Lolos pertanyaan usia dan domisili".to_string()),
        confidence: Some(OutcomeConfidence::Observed),
        observed_at: Some("2026-11-20T08:05:00Z".to_string()),
    };

    let qualified = OutcomeLearningService::record_outcome_event(&mut conn, event1_input)
        .expect("transisi ke qualified harus berhasil");
    assert_eq!(qualified.current_outcome, OutcomeLabel::Qualified);
    assert_eq!(qualified.events.len(), 2);
    assert_eq!(qualified.events[1].outcome_label, OutcomeLabel::Qualified);

    // Langkah 3: Selesai penuh seluruh kuesioner (transisi terminal ke completed)
    let event2_input = RecordOutcomeEventInput {
        attempt_id: created.id.clone(),
        persona_id: persona_id.to_string(),
        outcome_label: OutcomeLabel::Completed,
        evidence_source: "thank_you_completion_page".to_string(),
        evidence_note: Some("Mencapai layar terima kasih".to_string()),
        confidence: Some(OutcomeConfidence::UserConfirmed),
        observed_at: Some("2026-11-20T08:20:00Z".to_string()),
    };

    let completed = OutcomeLearningService::record_outcome_event(&mut conn, event2_input)
        .expect("transisi ke completed harus berhasil");
    assert_eq!(completed.current_outcome, OutcomeLabel::Completed);
    assert_eq!(completed.events.len(), 3);
    assert_eq!(completed.events[2].outcome_label, OutcomeLabel::Completed);
    assert!(completed.ended_at.is_some());
    assert_eq!(completed.ended_at.unwrap(), "2026-11-20T08:20:00Z");
}

/// 3. Konsistensi data dan keterhubungan dengan kriteria peluang survei (SC-050/SC-052)
#[test]
fn test_attempt_data_consistency_and_criteria_association() {
    let mut conn = setup_test_db();
    let persona_id = "persona-budi";
    create_test_persona(&conn, persona_id, "Budi Santoso", true);

    // 1. Simpan peluang survei dengan 2 kriteria teramati
    let opp_input = SaveOpportunityWithCriteriaInput {
        opportunity: survey_copilot_lib::domain::opportunity::CreateOpportunityInput {
            title: Some("Survei Perbankan Digital".to_string()),
            platform_label: Some("Toluna".to_string()),
            topic: Some("Finansial".to_string()),
            description: None,
            raw_text: "Kriteria: Usia 21-45 tahun, Memiliki Rekening Bank".to_string(),
            provenance: Some(OpportunityProvenance::ManualEntry),
            source_capture_id: None,
            source_ocr_id: None,
            captured_at: None,
        },
        criteria: vec![
            CriterionInput {
                id: None,
                criterion_text: "Usia antara 21 sampai 45 tahun".to_string(),
                raw_text: Some("Usia 21-45 tahun".to_string()),
                criterion_type: CriterionType::Age,
                normalized_attribute: Some("usia".to_string()),
                expected_value_json: Some(serde_json::json!({"min": 21, "max": 45})),
                operator: CriterionOperator::Range,
                criterion_order: 0,
            },
            CriterionInput {
                id: None,
                criterion_text: "Memiliki rekening bank aktif".to_string(),
                raw_text: Some("Memiliki Rekening Bank".to_string()),
                criterion_type: CriterionType::ProductUsage,
                normalized_attribute: Some("rekening_bank".to_string()),
                expected_value_json: Some(serde_json::json!(true)),
                operator: CriterionOperator::Equals,
                criterion_order: 1,
            },
        ],
    };

    let opp_detail = OpportunityService::save_opportunity(&mut conn, opp_input)
        .expect("simpan peluang harus sukses");

    // 2. Catat percobaan yang merujuk pada peluang ini
    let attempt_input = CreateAttemptInput {
        persona_id: persona_id.to_string(),
        opportunity_id: Some(opp_detail.opportunity.id.clone()),
        opportunity_title: None,
        source_platform: None,
        category: None,
        target_url: None,
        raw_text: None,
        session_id: None,
        started_at: Some("2026-11-22T14:00:00Z".to_string()),
        initial_outcome: Some(OutcomeLabel::Completed),
        evidence_source: Some("konfirmasi_pengguna".to_string()),
        evidence_note: Some("Selesai mengisi 15 pertanyaan".to_string()),
        confidence: Some(OutcomeConfidence::UserConfirmed),
    };

    let attempt_detail = OutcomeLearningService::record_attempt(&mut conn, attempt_input)
        .expect("pencatatan percobaan harus sukses");

    // Verifikasi keterhubungan kriteria teramati
    assert_eq!(attempt_detail.opportunity_id, opp_detail.opportunity.id);
    assert_eq!(attempt_detail.opportunity_title, "Survei Perbankan Digital");
    assert_eq!(attempt_detail.source_platform, "Toluna");
    assert_eq!(attempt_detail.category, Some("Finansial".to_string()));
    assert_eq!(attempt_detail.criteria.len(), 2);
    assert_eq!(
        attempt_detail.criteria[0].criterion_type,
        CriterionType::Age
    );
    assert_eq!(
        attempt_detail.criteria[1].criterion_type,
        CriterionType::ProductUsage
    );
}

/// 4. Isolasi Persona yang Ketat (Strict Persona Isolation)
#[test]
fn test_strict_persona_isolation() {
    let mut conn = setup_test_db();
    let persona_a = "persona-a";
    let persona_b = "persona-b";
    create_test_persona(&conn, persona_a, "Persona A", true);
    create_test_persona(&conn, persona_b, "Persona B", false);

    // Persona A mencatat percobaan
    let input_a = CreateAttemptInput {
        persona_id: persona_a.to_string(),
        opportunity_id: None,
        opportunity_title: Some("Survei Rahasia Persona A".to_string()),
        source_platform: Some("YouGov".to_string()),
        category: Some("Pribadi".to_string()),
        target_url: None,
        raw_text: None,
        session_id: None,
        started_at: Some("2026-11-22T10:00:00Z".to_string()),
        initial_outcome: Some(OutcomeLabel::Completed),
        evidence_source: Some("user".to_string()),
        evidence_note: None,
        confidence: Some(OutcomeConfidence::Observed),
    };

    let detail_a = OutcomeLearningService::record_attempt(&mut conn, input_a)
        .expect("persona A harus bisa membuat percobaan");

    // 1. Persona B tidak boleh bisa mengakses detail percobaan Persona A
    let access_b = OutcomeLearningService::get_attempt_detail(&conn, &detail_a.id, Some(persona_b));
    assert!(
        access_b.is_err(),
        "Persona B tidak boleh dapat mengakses percobaan persona A"
    );

    // 2. Persona B tidak boleh bisa memperbarui event pada percobaan Persona A
    let update_b = RecordOutcomeEventInput {
        attempt_id: detail_a.id.clone(),
        persona_id: persona_b.to_string(),
        outcome_label: OutcomeLabel::ScreenedOut,
        evidence_source: "hacker".to_string(),
        evidence_note: None,
        confidence: None,
        observed_at: None,
    };
    let update_res = OutcomeLearningService::record_outcome_event(&mut conn, update_b);
    assert!(
        update_res.is_err(),
        "Persona B dilarang memperbarui percobaan Persona A"
    );

    // 3. Filter riwayat untuk Persona B tidak boleh menampilkan percobaan Persona A
    let list_b = OutcomeLearningService::list_attempts(
        &conn,
        AttemptFilter {
            persona_id: Some(persona_b.to_string()),
            ..Default::default()
        },
    )
    .expect("query list persona B harus sukses");
    assert_eq!(
        list_b.len(),
        0,
        "Percobaan Persona A tidak boleh bocor ke Persona B"
    );

    // 4. Pembelajaran historis Persona B tidak boleh memuat data Persona A
    let summary_b =
        OutcomeLearningService::evaluate_historical_learning(&conn, persona_b, None, None)
            .expect("ringkasan pembelajaran persona B harus sukses");
    assert_eq!(summary_b.sample_size, 0);
}

/// 5. Filter Riwayat Berdasarkan Kategori, Sumber Platform, dan Status
#[test]
fn test_attempt_filtering_by_category_platform_status() {
    let mut conn = setup_test_db();
    let persona_id = "persona-filter";
    create_test_persona(&conn, persona_id, "Persona Filter", true);

    // Catat 3 percobaan dengan variasi
    let attempts_data = vec![
        ("Survei Game", "Milieu", "Gaming", OutcomeLabel::Completed),
        (
            "Survei Belanja",
            "YouGov",
            "Retail",
            OutcomeLabel::ScreenedOut,
        ),
        (
            "Survei Otomotif",
            "YouGov",
            "Otomotif",
            OutcomeLabel::QuotaFull,
        ),
    ];

    for (title, platform, category, status) in attempts_data {
        OutcomeLearningService::record_attempt(
            &mut conn,
            CreateAttemptInput {
                persona_id: persona_id.to_string(),
                opportunity_id: None,
                opportunity_title: Some(title.to_string()),
                source_platform: Some(platform.to_string()),
                category: Some(category.to_string()),
                target_url: None,
                raw_text: None,
                session_id: None,
                started_at: Some("2026-11-20T10:00:00Z".to_string()),
                initial_outcome: Some(status),
                evidence_source: Some("test".to_string()),
                evidence_note: None,
                confidence: None,
            },
        )
        .expect("insert attempt");
    }

    // Filter by platform "YouGov"
    let yougov_list = OutcomeLearningService::list_attempts(
        &conn,
        AttemptFilter {
            persona_id: Some(persona_id.to_string()),
            source_platform: Some("YouGov".to_string()),
            ..Default::default()
        },
    )
    .expect("list yougov");
    assert_eq!(yougov_list.len(), 2);

    // Filter by category "Gaming"
    let gaming_list = OutcomeLearningService::list_attempts(
        &conn,
        AttemptFilter {
            persona_id: Some(persona_id.to_string()),
            category: Some("Gaming".to_string()),
            ..Default::default()
        },
    )
    .expect("list gaming");
    assert_eq!(gaming_list.len(), 1);
    assert_eq!(gaming_list[0].opportunity_title, "Survei Game");

    // Filter by status "ScreenedOut"
    let screened_list = OutcomeLearningService::list_attempts(
        &conn,
        AttemptFilter {
            persona_id: Some(persona_id.to_string()),
            status: Some(OutcomeLabel::ScreenedOut),
            ..Default::default()
        },
    )
    .expect("list screened");
    assert_eq!(screened_list.len(), 1);
    assert_eq!(screened_list[0].opportunity_title, "Survei Belanja");
}

/// 6. AT-063: Sparse History Is Labeled Insufficient Without Fabricated Probabilities
#[test]
fn test_at_063_sparse_history_labeled_insufficient() {
    let mut conn = setup_test_db();
    let persona_id = "persona-sparse";
    create_test_persona(&conn, persona_id, "Persona Data Terbatas", true);

    // Skenario A: 0 riwayat historis
    let empty_summary =
        OutcomeLearningService::evaluate_historical_learning(&conn, persona_id, None, None)
            .expect("evaluasi data kosong harus sukses");

    assert_eq!(empty_summary.status, LearningDataStatus::InsufficientData);
    assert_eq!(empty_summary.sample_size, 0);
    assert_eq!(empty_summary.minimum_required_sample, MIN_SAMPLE_THRESHOLD);
    assert!(
        empty_summary.completion_rate.is_none(),
        "Dilarang menampilkan completion_rate jika data sparse"
    );
    assert!(
        empty_summary.qualification_rate.is_none(),
        "Dilarang memfabrikasi persentase kelulusan jika data sparse"
    );
    assert!(empty_summary.holdout_consistency_rate.is_none());
    assert!(empty_summary
        .message
        .contains("Data historis belum mencukupi"));
    assert!(empty_summary
        .uncertainty_disclosure
        .contains("Ketidakpastian sangat tinggi"));

    // Skenario B: Hanya 3 percobaan (di bawah ambang batas MIN_SAMPLE_THRESHOLD = 10)
    for i in 1..=3 {
        OutcomeLearningService::record_attempt(
            &mut conn,
            CreateAttemptInput {
                persona_id: persona_id.to_string(),
                opportunity_id: None,
                opportunity_title: Some(format!("Survei Sparse #{}", i)),
                source_platform: Some("Milieu".to_string()),
                category: Some("Umum".to_string()),
                target_url: None,
                raw_text: None,
                session_id: None,
                started_at: Some(format!("2026-11-2{}T10:00:00Z", i)),
                initial_outcome: Some(OutcomeLabel::Completed),
                evidence_source: Some("test".to_string()),
                evidence_note: None,
                confidence: None,
            },
        )
        .expect("insert sparse attempt");
    }

    let sparse_summary =
        OutcomeLearningService::evaluate_historical_learning(&conn, persona_id, None, None)
            .expect("evaluasi 3 data harus sukses");

    assert_eq!(sparse_summary.status, LearningDataStatus::InsufficientData);
    assert_eq!(sparse_summary.sample_size, 3);
    assert_eq!(
        sparse_summary.completion_rate, None,
        "Dilarang menampilkan 100% dari 3 data kecil"
    );
    assert_eq!(sparse_summary.qualification_rate, None);
    assert!(sparse_summary
        .message
        .contains("tercatat 3 dari minimal 10"));
}

/// 7. Validasi Chronological Holdout dengan Sampel Memadai (>= 10 Observasi)
#[test]
fn test_chronological_holdout_validation_sufficient_history() {
    let mut conn = setup_test_db();
    let persona_id = "persona-sufficient";
    create_test_persona(&conn, persona_id, "Persona Data Memadai", true);

    // Catat 12 observasi dalam urutan waktu kronologis (10 November s.d. 21 November)
    let outcomes = vec![
        OutcomeLabel::Completed,   // 1
        OutcomeLabel::Qualified,   // 2
        OutcomeLabel::ScreenedOut, // 3
        OutcomeLabel::QuotaFull,   // 4
        OutcomeLabel::Completed,   // 5
        OutcomeLabel::ScreenedOut, // 6
        OutcomeLabel::Completed,   // 7
        OutcomeLabel::Qualified,   // 8  <- Batas train 70% (8 item)
        OutcomeLabel::Completed,   // 9  <- Holdout 30% (4 item)
        OutcomeLabel::ScreenedOut, // 10
        OutcomeLabel::Completed,   // 11
        OutcomeLabel::ScreenedOut, // 12
    ];

    assert_eq!(outcomes.len(), 12);
    assert!(outcomes.len() >= MIN_SAMPLE_THRESHOLD);

    for (idx, status) in outcomes.into_iter().enumerate() {
        let day = idx + 10;
        OutcomeLearningService::record_attempt(
            &mut conn,
            CreateAttemptInput {
                persona_id: persona_id.to_string(),
                opportunity_id: None,
                opportunity_title: Some(format!("Survei Validasi Chronological #{}", idx + 1)),
                source_platform: Some("YouGov".to_string()),
                category: Some("Finansial".to_string()),
                target_url: None,
                raw_text: None,
                session_id: None,
                started_at: Some(format!("2026-11-{}T10:00:00Z", day)),
                initial_outcome: Some(status),
                evidence_source: Some("riwayat_kronologis".to_string()),
                evidence_note: None,
                confidence: Some(OutcomeConfidence::Observed),
            },
        )
        .expect("insert historical attempt");
    }

    let summary = OutcomeLearningService::evaluate_historical_learning(
        &conn,
        persona_id,
        Some("Finansial"),
        Some("YouGov"),
    )
    .expect("evaluasi pembelajaran cukup harus sukses");

    assert_eq!(summary.status, LearningDataStatus::SufficientData);
    assert_eq!(summary.sample_size, 12);
    assert!(summary.completion_rate.is_some());
    assert!(summary.qualification_rate.is_some());
    assert_eq!(summary.holdout_sample_size, 4); // 12 - (12 * 70 / 100) = 12 - 8 = 4
    assert!(summary.holdout_consistency_rate.is_some());

    // Verifikasi rentang observasi
    assert_eq!(
        summary.observation_window_start,
        Some("2026-11-10T10:00:00Z".to_string())
    );
    assert_eq!(
        summary.observation_window_end,
        Some("2026-11-21T10:00:00Z".to_string())
    );
}

/// 8. Batas Produk: Tidak Ada Pelacakan Saldo/Uang dan Tidak Ada Manipulasi Jawaban
#[test]
fn test_product_boundary_no_earnings_and_no_answer_manipulation() {
    let conn = setup_test_db();
    let persona_id = "persona-boundary";
    create_test_persona(&conn, persona_id, "Persona Batas Integritas", true);

    let summary =
        OutcomeLearningService::evaluate_historical_learning(&conn, persona_id, None, None)
            .expect("ringkasan pembelajaran");

    // Periksa disclaimer non-negosiasi
    let has_no_earnings = summary
        .disclaimers
        .iter()
        .any(|d| d.contains("Tidak ada fitur pelacakan saldo, reward, uang tunai, atau poin"));
    let has_no_manipulation = summary
        .disclaimers
        .iter()
        .any(|d| d.contains("Dilarang memanipulasi profil atau merekayasa jawaban"));
    let has_separation = summary
        .disclaimers
        .iter()
        .any(|d| d.contains("Observasi faktual terpisah secara tegas"));

    assert!(
        has_no_earnings,
        "Harus menegakkan batas tidak ada saldo/reward"
    );
    assert!(has_no_manipulation, "Harus melarang manipulasi jawaban");
    assert!(has_separation, "Harus memisahkan observasi dari estimasi");
}
