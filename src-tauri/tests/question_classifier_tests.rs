use rusqlite::Connection;
use survey_copilot_lib::domain::classifier::{
    AmbiguityType, ClassificationConfidence, ClassifyQuestionInput, ConstraintProvenance,
    QuestionCategory, ScaleDirection, SelectionMode, TimeframeType,
};
use survey_copilot_lib::domain::survey::{
    AnswerFormat, CreateQuestionInput, CreateSessionInput, QuestionType,
};
use survey_copilot_lib::migrations::MigrationRunner;
use survey_copilot_lib::repository::PersonaRepository;
use survey_copilot_lib::services::question_classifier::QuestionClassifierService;
use survey_copilot_lib::services::SurveyService;

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
fn test_category_demographic() {
    let opts = vec![
        "< 18 tahun".to_string(),
        "18 - 24 tahun".to_string(),
        "25 - 34 tahun".to_string(),
    ];
    let res =
        QuestionClassifierService::classify_raw("Berapa usia Anda saat ini?", None, Some(&opts));
    assert_eq!(res.primary_category, QuestionCategory::Demographic);
    assert!(res.categories.contains(&QuestionCategory::Numeric));
    assert_eq!(res.constraints.selection_mode, SelectionMode::Single);

    let gender_opts = vec!["Laki-laki".to_string(), "Perempuan".to_string()];
    let res_gender = QuestionClassifierService::classify_raw(
        "Apakah jenis kelamin Anda?",
        None,
        Some(&gender_opts),
    );
    assert_eq!(res_gender.primary_category, QuestionCategory::Demographic);

    let res_en = QuestionClassifierService::classify_raw("What is your age in years?", None, None);
    assert_eq!(res_en.primary_category, QuestionCategory::Demographic);
}

#[test]
fn test_category_employment_education() {
    let emp_opts = vec![
        "Karyawan Swasta".to_string(),
        "PNS".to_string(),
        "Wirausaha".to_string(),
    ];
    let res_emp = QuestionClassifierService::classify_raw(
        "Apa pekerjaan atau profesi utama Anda saat ini?",
        None,
        Some(&emp_opts),
    );
    assert_eq!(
        res_emp.primary_category,
        QuestionCategory::EmploymentEducation
    );

    let edu_opts = vec![
        "SMA/SMK".to_string(),
        "Diploma".to_string(),
        "Sarjana (S1)".to_string(),
    ];
    let res_edu = QuestionClassifierService::classify_raw(
        "Apa tingkat pendidikan terakhir yang telah Anda selesaikan?",
        None,
        Some(&edu_opts),
    );
    assert_eq!(
        res_edu.primary_category,
        QuestionCategory::EmploymentEducation
    );
}

#[test]
fn test_category_financial_economic() {
    let res = QuestionClassifierService::classify_raw(
        "Berapa rata-rata pendapatan kotor per bulan keluarga Anda (dalam Rupiah)?",
        Some("Tuliskan dalam angka"),
        None,
    );
    assert_eq!(res.primary_category, QuestionCategory::FinancialEconomic);
    assert_eq!(res.question_type, QuestionType::Numeric);
    assert_eq!(res.answer_format, AnswerFormat::Number);
    let unit = res.constraints.units.expect("unit must be extracted");
    assert!(unit.unit_name.to_lowercase().contains("rupiah"));
}

#[test]
fn test_category_consumption_habits() {
    let opts = vec![
        "Setiap hari".to_string(),
        "3-4 kali seminggu".to_string(),
        "1-2 kali seminggu".to_string(),
    ];
    let res = QuestionClassifierService::classify_raw(
        "Seberapa sering Anda mengonsumsi kopi susu kemasan dalam 1 minggu terakhir?",
        None,
        Some(&opts),
    );
    assert_eq!(res.primary_category, QuestionCategory::ConsumptionHabits);
    assert!(res
        .categories
        .contains(&QuestionCategory::ExperienceBehavior));
    assert!(res.constraints.timeframe.is_some());
}

#[test]
fn test_category_asset_ownership() {
    let opts = vec!["Ya, memiliki".to_string(), "Tidak memiliki".to_string()];
    let res = QuestionClassifierService::classify_raw(
        "Apakah Anda atau keluarga Anda memiliki kendaraan mobil pribadi?",
        None,
        Some(&opts),
    );
    assert_eq!(res.primary_category, QuestionCategory::AssetOwnership);
    assert_eq!(res.answer_format, AnswerFormat::Boolean);
}

#[test]
fn test_category_brand_product_usage() {
    let opts = vec![
        "Samsung".to_string(),
        "Apple".to_string(),
        "Xiaomi".to_string(),
    ];
    let res = QuestionClassifierService::classify_raw(
        "Merek smartphone apa yang Anda gunakan saat ini?",
        None,
        Some(&opts),
    );
    assert_eq!(res.primary_category, QuestionCategory::BrandProductUsage);
}

#[test]
fn test_category_experience_behavior() {
    let res = QuestionClassifierService::classify_raw(
        "Ceritakan pengalaman Anda saat mengunjungi gerai kami minggu lalu.",
        Some("Tuliskan secara singkat"),
        None,
    );
    assert_eq!(res.primary_category, QuestionCategory::ExperienceBehavior);
    assert_eq!(res.question_type, QuestionType::FreeText);
    assert_eq!(res.answer_format, AnswerFormat::Text);
}

#[test]
fn test_category_preference_opinion() {
    let opts = vec![
        "Shopee".to_string(),
        "Tokopedia".to_string(),
        "Lazada".to_string(),
    ];
    let res = QuestionClassifierService::classify_raw(
        "Menurut pendapat Anda, manakah platform e-commerce yang paling Anda sukai?",
        None,
        Some(&opts),
    );
    assert_eq!(res.primary_category, QuestionCategory::PreferenceOpinion);
}

#[test]
fn test_category_satisfaction_rating_and_scale() {
    let res = QuestionClassifierService::classify_raw(
        "Seberapa puas Anda dengan pelayanan customer service kami? Berikan nilai dari 1 (Sangat Tidak Puas) hingga 5 (Sangat Puas).",
        None,
        None,
    );
    assert_eq!(res.primary_category, QuestionCategory::SatisfactionRating);
    assert_eq!(res.question_type, QuestionType::Scale);
    assert_eq!(res.answer_format, AnswerFormat::RatingScale);

    let scale = res.constraints.scale.expect("scale must be extracted");
    assert_eq!(scale.min_scale, 1);
    assert_eq!(scale.max_scale, 5);
    assert_eq!(scale.direction, ScaleDirection::LowToHigh);
    assert!(!scale.anchors.is_empty());
}

#[test]
fn test_category_multiple_choice_and_selection_limits() {
    let opts = vec![
        "WhatsApp".to_string(),
        "Telegram".to_string(),
        "Instagram".to_string(),
        "TikTok".to_string(),
        "Twitter/X".to_string(),
    ];
    let res = QuestionClassifierService::classify_raw(
        "Aplikasi apa saja yang Anda gunakan? Pilih minimal 2 dan maksimal 4 pilihan.",
        Some("Pilih semua yang berlaku"),
        Some(&opts),
    );
    assert_eq!(res.constraints.selection_mode, SelectionMode::Multiple);
    assert_eq!(res.constraints.min_selections, Some(2));
    assert_eq!(res.constraints.max_selections, Some(4));
    assert_eq!(res.question_type, QuestionType::MultipleChoice);
    assert_eq!(res.answer_format, AnswerFormat::MultipleSelection);
}

#[test]
fn test_category_matrix_grid() {
    let res = QuestionClassifierService::classify_raw(
        "Beri penilaian untuk setiap produk pada tabel matriks berikut:",
        Some("Pilih satu nilai untuk setiap baris"),
        None,
    );
    assert_eq!(res.primary_category, QuestionCategory::MatrixGrid);
    assert_eq!(res.question_type, QuestionType::Matrix);
    assert_eq!(res.answer_format, AnswerFormat::Matrix);
}

#[test]
fn test_category_numeric_and_date() {
    let res_num = QuestionClassifierService::classify_raw(
        "Berapa jumlah anggota keluarga yang tinggal serumah dengan Anda?",
        None,
        None,
    );
    assert_eq!(res_num.question_type, QuestionType::Numeric);
    assert_eq!(res_num.answer_format, AnswerFormat::Number);

    let res_date = QuestionClassifierService::classify_raw(
        "Kapan tanggal lahir Anda? (Format: DD/MM/YYYY)",
        None,
        None,
    );
    assert_eq!(res_date.primary_category, QuestionCategory::Date);
    assert_eq!(res_date.question_type, QuestionType::Date);
}

#[test]
fn test_category_unknown_or_gibberish() {
    let res = QuestionClassifierService::classify_raw("qwrty zxcvb 98765 asdfghjkl", None, None);
    assert_eq!(res.primary_category, QuestionCategory::OtherUnknown);
    assert_eq!(res.confidence, ClassificationConfidence::Uncertain);
    assert!(res.ambiguity.is_ambiguous);
}

#[test]
fn test_constraint_timeframe_explicit_and_inferred() {
    let opts = vec!["Ya".to_string(), "Tidak".to_string()];
    let res_12m = QuestionClassifierService::classify_raw(
        "Dalam 12 bulan terakhir, apakah Anda pernah bepergian keluar negeri?",
        None,
        Some(&opts),
    );
    let tf1 = res_12m
        .constraints
        .timeframe
        .expect("timeframe 12m must exist");
    assert_eq!(tf1.provenance, ConstraintProvenance::Explicit);
    assert_eq!(tf1.count, Some(12));
    assert_eq!(tf1.timeframe_type, TimeframeType::PastNMonths);

    let res_30d = QuestionClassifierService::classify_raw(
        "Apakah Anda membeli detergen dalam 30 hari terakhir?",
        None,
        Some(&opts),
    );
    let tf2 = res_30d
        .constraints
        .timeframe
        .expect("timeframe 30d must exist");
    assert_eq!(tf2.count, Some(30));
    assert_eq!(tf2.timeframe_type, TimeframeType::PastNDays);
}

#[test]
fn test_constraint_text_length() {
    let res = QuestionClassifierService::classify_raw(
        "Berikan alasan Anda memilih produk ini.",
        Some("Tuliskan ulasan minimal 10 kata dan maksimal 250 karakter."),
        None,
    );
    let len = res
        .constraints
        .text_length
        .expect("text length must be extracted");
    assert_eq!(len.max_chars, Some(250));
    assert_eq!(len.provenance, ConstraintProvenance::Explicit);
}

#[test]
fn test_ambiguity_missing_timeframe_behavioral_at_041() {
    let opts = vec![
        "Sering".to_string(),
        "Kadang-kadang".to_string(),
        "Jarang".to_string(),
    ];
    let res = QuestionClassifierService::classify_raw(
        "Seberapa sering Anda membeli produk detergen cair?",
        None,
        Some(&opts),
    );
    assert!(res.ambiguity.is_ambiguous);
    assert!(res.ambiguity.needs_clarification);
    assert!(res
        .ambiguity
        .reasons
        .iter()
        .any(|r| r.ambiguity_type == AmbiguityType::MissingTimeframe));
    let prompt = res
        .ambiguity
        .clarification_prompt
        .expect("prompt must exist");
    assert!(prompt.to_lowercase().contains("periode waktu"));
}

#[test]
fn test_ambiguity_missing_unit_financial_at_041() {
    let res = QuestionClassifierService::classify_raw(
        "Berapa total gaji atau penghasilan bulanan Anda?",
        None,
        None,
    );
    assert!(res.ambiguity.is_ambiguous);
    assert!(res.ambiguity.needs_clarification);
    assert!(res
        .ambiguity
        .reasons
        .iter()
        .any(|r| r.ambiguity_type == AmbiguityType::MissingUnit));
    let prompt = res
        .ambiguity
        .clarification_prompt
        .expect("prompt must exist");
    assert!(
        prompt.to_lowercase().contains("satuan") || prompt.to_lowercase().contains("mata uang")
    );
}

#[test]
fn test_ambiguity_conflicting_instructions() {
    let opts = vec!["Kapal Api".to_string(), "Nescafe".to_string()];
    let res = QuestionClassifierService::classify_raw(
        "Pilih merk kopi yang Anda sukai. Pilih 1 jawaban saja namun pilih semua yang berlaku.",
        None,
        Some(&opts),
    );
    assert!(res.ambiguity.is_ambiguous);
    assert!(res
        .ambiguity
        .reasons
        .iter()
        .any(|r| r.ambiguity_type == AmbiguityType::ConflictingInstructions));

    let opts2 = vec!["Opsi A".to_string(), "Opsi B".to_string()];
    let res2 = QuestionClassifierService::classify_raw(
        "Pilih minimal 5 pilihan favorit Anda",
        None,
        Some(&opts2),
    );
    assert!(res2.ambiguity.is_ambiguous);
    assert!(res2
        .ambiguity
        .reasons
        .iter()
        .any(|r| r.ambiguity_type == AmbiguityType::ConflictingInstructions));
}

#[test]
fn test_ambiguity_mismatched_options() {
    let opts = vec!["Ya".to_string(), "Tidak".to_string()];
    let res =
        QuestionClassifierService::classify_raw("Berapa umur Anda sekarang?", None, Some(&opts));
    assert!(res.ambiguity.is_ambiguous);
    assert!(res
        .ambiguity
        .reasons
        .iter()
        .any(|r| r.ambiguity_type == AmbiguityType::MismatchedOptions));
}

#[test]
fn test_english_language_support() {
    let opts = vec![
        "1-2 times".to_string(),
        "3-5 times".to_string(),
        "> 5 times".to_string(),
    ];
    let res = QuestionClassifierService::classify_raw(
        "In the past 6 months, how many times did you purchase groceries online?",
        Some("Select all that apply"),
        Some(&opts),
    );
    assert!(
        res.categories
            .contains(&QuestionCategory::ExperienceBehavior)
            || res.primary_category == QuestionCategory::ConsumptionHabits
    );
    assert_eq!(res.constraints.selection_mode, SelectionMode::Multiple);
    let tf = res.constraints.timeframe.expect("timeframe must exist");
    assert_eq!(tf.count, Some(6));
}

#[test]
fn test_data_integrity_and_original_text_immutability() {
    let raw_text = "  bErapa  USIA   Anda   saat  ini???  ";
    let input = ClassifyQuestionInput {
        persona_id: None,
        session_id: None,
        question_id: None,
        question_text: raw_text.to_string(),
        instructions: None,
        options: None,
    };
    let res = QuestionClassifierService::classify(&input);

    // Raw original input text remains untouched
    assert_eq!(input.question_text, raw_text);
    // Normalized text is clean and separated
    assert_eq!(res.normalized_text, "berapa usia anda saat ini???");
    assert_eq!(res.classifier_version, "1.0.0");
}

#[test]
fn test_survey_service_classification_and_persona_isolation() {
    let conn = setup_test_db();
    let persona_1 = create_test_persona(&conn, "Budi Santoso", true);
    let persona_2 = create_test_persona(&conn, "Siti Rahma", false);

    // Create session for persona_1
    let session = SurveyService::create_session(
        &conn,
        CreateSessionInput {
            persona_id: persona_1.clone(),
            title: "Survei Perilaku Belanja 2026".to_string(),
            platform_label: Some("Milieu".to_string()),
            source_url: None,
            external_survey_id: None,
            language: None,
            category: None,
            notes: None,
        },
    )
    .expect("create session must succeed");

    // Create question with demographic & timeframe
    let question = SurveyService::create_question(
        &conn,
        CreateQuestionInput {
            persona_id: persona_1.clone(),
            session_id: session.id.clone(),
            capture_id: None,
            question_text_original: "Berapa usia Anda saat ini?".to_string(),
            instructions: Some("Pilih salah satu".to_string()),
            options: Some(vec![
                "< 20".to_string(),
                "20-29".to_string(),
                ">= 30".to_string(),
            ]),
            question_type: None,
            answer_format: None,
            provenance: None,
            question_order: None,
            preceding_question_id: None,
            branch_logic: None,
        },
    )
    .expect("create question must succeed");

    // Verify structure_json was automatically populated by QuestionClassifier
    let struct_val = question
        .structure_json
        .expect("structure_json must be populated");
    assert_eq!(struct_val["primaryCategory"], "demographic");
    assert_eq!(struct_val["classifierVersion"], "1.0.0");
    assert_eq!(question.question_type, QuestionType::SingleChoice);
    assert_eq!(question.answer_format, AnswerFormat::SingleChoice);

    // Persona isolation test: persona_2 cannot classify or access persona_1's question
    let cross_classify_err = SurveyService::classify_question(
        &conn,
        ClassifyQuestionInput {
            persona_id: Some(persona_2.clone()),
            session_id: Some(session.id.clone()),
            question_id: Some(question.id.clone()),
            question_text: "Pertanyaan uji".to_string(),
            instructions: None,
            options: None,
        },
    );
    assert!(cross_classify_err.is_err());
}
