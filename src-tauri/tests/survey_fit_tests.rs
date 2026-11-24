use rusqlite::Connection;
use survey_copilot_lib::domain::fit::{MatchState, OverallFitState};
use survey_copilot_lib::domain::opportunity::{
    CreateOpportunityInput, CriterionInput, CriterionOperator, CriterionType,
    OpportunityProvenance, SaveOpportunityWithCriteriaInput,
};
use survey_copilot_lib::migrations::MigrationRunner;
use survey_copilot_lib::services::fit_service::FitService;
use survey_copilot_lib::services::opportunity_service::OpportunityService;

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

fn create_test_fact(
    conn: &Connection,
    id: &str,
    persona_id: &str,
    category: &str,
    attribute: &str,
    value_json: &str,
    normalized_value: &str,
    verification_status: &str,
) {
    conn.execute(
        r#"
        INSERT INTO profile_facts (
            id, persona_id, category, attribute, value_json, normalized_value,
            verification_status, sensitivity, created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, 'ordinary', '2026-11-20T12:00:00Z', '2026-11-20T12:00:00Z'
        )
        "#,
        rusqlite::params![
            id,
            persona_id,
            category,
            attribute,
            value_json,
            normalized_value,
            verification_status
        ],
    )
    .expect("fact insertion should succeed");
}

/// AT-060: Kriteria cocok dengan fakta profil persona terkonfirmasi
#[test]
fn test_at_060_criterion_matches_confirmed_current_fact() {
    let mut conn = setup_test_db();
    let persona_id = "persona-budi";
    create_test_persona(&conn, persona_id, "Budi Santoso", true);

    // Fakta profil terkonfirmasi
    create_test_fact(
        &conn,
        "fact-age",
        persona_id,
        "demographics",
        "usia",
        "28",
        "28",
        "confirmed",
    );
    create_test_fact(
        &conn,
        "fact-gender",
        persona_id,
        "demographics",
        "jenis_kelamin",
        "\"pria\"",
        "pria",
        "confirmed",
    );
    create_test_fact(
        &conn,
        "fact-loc",
        persona_id,
        "demographics",
        "domisili",
        "\"Jakarta Selatan\"",
        "jakarta selatan",
        "confirmed",
    );

    // Peluang survei dengan kriteria eksplisit
    let opp_detail = OpportunityService::save_opportunity(
        &mut conn,
        SaveOpportunityWithCriteriaInput {
            opportunity: CreateOpportunityInput {
                title: Some("Survei Konsumen Kopi".to_string()),
                platform_label: Some("YouGov".to_string()),
                topic: Some("Konsumsi".to_string()),
                description: None,
                raw_text: "Survei Kopi untuk Pria usia 20-35 di Jabodetabek".to_string(),
                provenance: Some(OpportunityProvenance::ManualEntry),
                source_capture_id: None,
                source_ocr_id: None,
                captured_at: None,
            },
            criteria: vec![
                CriterionInput {
                    id: None,
                    criterion_text: "Usia 20 - 35 tahun".to_string(),
                    raw_text: Some("usia 20-35".to_string()),
                    criterion_type: CriterionType::Age,
                    normalized_attribute: Some("age".to_string()),
                    expected_value_json: Some(serde_json::json!({"min": 20, "max": 35})),
                    operator: CriterionOperator::Range,
                    criterion_order: 0,
                },
                CriterionInput {
                    id: None,
                    criterion_text: "Jenis Kelamin Pria".to_string(),
                    raw_text: Some("Pria".to_string()),
                    criterion_type: CriterionType::Gender,
                    normalized_attribute: Some("gender".to_string()),
                    expected_value_json: Some(serde_json::json!(["pria"])),
                    operator: CriterionOperator::In,
                    criterion_order: 1,
                },
                CriterionInput {
                    id: None,
                    criterion_text: "Domisili Jabodetabek".to_string(),
                    raw_text: Some("Jabodetabek".to_string()),
                    criterion_type: CriterionType::Location,
                    normalized_attribute: Some("location".to_string()),
                    expected_value_json: Some(serde_json::json!(["jabodetabek", "jakarta"])),
                    operator: CriterionOperator::In,
                    criterion_order: 2,
                },
            ],
        },
    )
    .expect("opportunity save should succeed");

    // Evaluasi fit deterministik
    let fit = FitService::evaluate_opportunity_fit(&mut conn, &opp_detail.opportunity.id, persona_id)
        .expect("fit evaluation should succeed");

    assert_eq!(fit.overall_fit, OverallFitState::Eligible);
    assert_eq!(fit.match_count, 3);
    assert_eq!(fit.mismatch_count, 0);
    assert_eq!(fit.unknown_count, 0);

    for res in &fit.criteria_results {
        assert_eq!(res.match_state, MatchState::Match);
        assert!(
            !res.supporting_fact_ids.is_empty(),
            "Kriteria '{}' harus memiliki referensi ID fakta pendukung (AT-060)",
            res.criterion_text
        );
    }
}

/// AT-061: Bukti kriteria tidak ada -> Kembalikan Unknown, JANGAN PERNAH dianggap Match
#[test]
fn test_at_061_criterion_evidence_absent_returns_unknown_never_match() {
    let mut conn = setup_test_db();
    let persona_id = "persona-siti";
    create_test_persona(&conn, persona_id, "Siti Rahma", true);

    // Hanya fakta usia, TIDAK ADA fakta kepemilikan hewan atau status merokok
    create_test_fact(
        &conn,
        "fact-age",
        persona_id,
        "demographics",
        "usia",
        "30",
        "30",
        "confirmed",
    );

    let opp_detail = OpportunityService::save_opportunity(
        &mut conn,
        SaveOpportunityWithCriteriaInput {
            opportunity: CreateOpportunityInput {
                title: Some("Survei Pemilik Hewan Peliharaan".to_string()),
                platform_label: Some("Toluna".to_string()),
                topic: Some("Hewan".to_string()),
                description: None,
                raw_text: "Pemilik kucing atau anjing".to_string(),
                provenance: Some(OpportunityProvenance::ManualEntry),
                source_capture_id: None,
                source_ocr_id: None,
                captured_at: None,
            },
            criteria: vec![CriterionInput {
                id: None,
                criterion_text: "Memiliki hewan peliharaan kucing".to_string(),
                raw_text: Some("Memiliki hewan peliharaan kucing".to_string()),
                criterion_type: CriterionType::ProductUsage,
                normalized_attribute: Some("pet".to_string()),
                expected_value_json: Some(serde_json::json!({"value": "kucing"})),
                operator: CriterionOperator::Contains,
                criterion_order: 0,
            }],
        },
    )
    .expect("opportunity save should succeed");

    let fit = FitService::evaluate_opportunity_fit(&mut conn, &opp_detail.opportunity.id, persona_id)
        .expect("fit evaluation should succeed");

    assert_eq!(fit.criteria_results.len(), 1);
    let crit_res = &fit.criteria_results[0];

    // Invariant: Status harus Unknown, dan Unknown TIDAK PERNAH dianggap Match (AT-061)
    assert_eq!(
        crit_res.match_state,
        MatchState::Unknown,
        "Kriteria tanpa bukti fakta harus berstatus Unknown"
    );
    assert_ne!(
        crit_res.match_state,
        MatchState::Match,
        "Unknown tidak boleh dianggap Match"
    );
    assert!(
        crit_res.supporting_fact_ids.is_empty(),
        "Kriteria unknown tidak memiliki supporting fact"
    );

    // Status keseluruhan harus Incomplete (bukan Eligible!)
    assert_eq!(
        fit.overall_fit,
        OverallFitState::Incomplete,
        "Keseluruhan fit harus Incomplete jika kriteria unknown"
    );
    assert_ne!(fit.overall_fit, OverallFitState::Eligible);
}

/// AT-062: Fakta kadaluarsa (Stale) atau diperdebatkan (Disputed/Conflict)
#[test]
fn test_at_062_criterion_fact_is_stale_or_disputed() {
    let mut conn = setup_test_db();
    let persona_stale = "persona-stale";
    let persona_disputed = "persona-disputed";
    create_test_persona(&conn, persona_stale, "Persona Kadaluarsa", true);
    create_test_persona(&conn, persona_disputed, "Persona Konflik", false);

    // 1. Fakta berstatus 'outdated' (Stale)
    create_test_fact(
        &conn,
        "fact-outdated-age",
        persona_stale,
        "demographics",
        "usia",
        "22",
        "22",
        "outdated",
    );

    // 2. Fakta berstatus 'disputed' (Conflict)
    create_test_fact(
        &conn,
        "fact-disputed-loc",
        persona_disputed,
        "demographics",
        "domisili",
        "\"Medan\"",
        "medan",
        "disputed",
    );

    let opp = OpportunityService::save_opportunity(
        &mut conn,
        SaveOpportunityWithCriteriaInput {
            opportunity: CreateOpportunityInput {
                title: Some("Survei Kebugaran".to_string()),
                platform_label: Some("Milieu".to_string()),
                topic: Some("Kesehatan".to_string()),
                description: None,
                raw_text: "Usia 20-30 domisili Medan".to_string(),
                provenance: Some(OpportunityProvenance::ManualEntry),
                source_capture_id: None,
                source_ocr_id: None,
                captured_at: None,
            },
            criteria: vec![
                CriterionInput {
                    id: None,
                    criterion_text: "Usia 20 - 30 tahun".to_string(),
                    raw_text: Some("Usia 20-30".to_string()),
                    criterion_type: CriterionType::Age,
                    normalized_attribute: Some("age".to_string()),
                    expected_value_json: Some(serde_json::json!({"min": 20, "max": 30})),
                    operator: CriterionOperator::Range,
                    criterion_order: 0,
                },
                CriterionInput {
                    id: None,
                    criterion_text: "Domisili Medan".to_string(),
                    raw_text: Some("Medan".to_string()),
                    criterion_type: CriterionType::Location,
                    normalized_attribute: Some("location".to_string()),
                    expected_value_json: Some(serde_json::json!(["medan"])),
                    operator: CriterionOperator::In,
                    criterion_order: 1,
                },
            ],
        },
    )
    .expect("opportunity save should succeed");

    // Evaluasi Persona Stale
    let fit_stale = FitService::evaluate_opportunity_fit(&mut conn, &opp.opportunity.id, persona_stale)
        .expect("fit evaluation should succeed");

    let age_res = fit_stale
        .criteria_results
        .iter()
        .find(|c| c.criterion_type == CriterionType::Age)
        .unwrap();
    assert_eq!(
        age_res.match_state,
        MatchState::Stale,
        "Fakta outdated harus menghasilkan status Stale (AT-062)"
    );
    assert_eq!(fit_stale.overall_fit, OverallFitState::Stale);

    // Evaluasi Persona Disputed
    let fit_disputed =
        FitService::evaluate_opportunity_fit(&mut conn, &opp.opportunity.id, persona_disputed)
            .expect("fit evaluation should succeed");

    let loc_res = fit_disputed
        .criteria_results
        .iter()
        .find(|c| c.criterion_type == CriterionType::Location)
        .unwrap();
    assert_eq!(
        loc_res.match_state,
        MatchState::Conflict,
        "Fakta disputed harus menghasilkan status Conflict (AT-062)"
    );
    assert_eq!(fit_disputed.overall_fit, OverallFitState::Conflict);
}

/// Pengujian kriteria eksklusi industri (NotEquals / Larangan riset pasar / periklanan)
#[test]
fn test_industry_exclusion_criteria() {
    let mut conn = setup_test_db();
    let p_allowed = "p-programmer";
    let p_excluded = "p-researcher";
    create_test_persona(&conn, p_allowed, "Programmer", true);
    create_test_persona(&conn, p_excluded, "Riset Pasar", false);

    create_test_fact(
        &conn,
        "f-prog",
        p_allowed,
        "employment",
        "pekerjaan",
        "\"Software Developer\"",
        "software developer",
        "confirmed",
    );
    create_test_fact(
        &conn,
        "f-res",
        p_excluded,
        "employment",
        "pekerjaan",
        "\"Market Research Agency\"",
        "market research agency",
        "confirmed",
    );

    let opp = OpportunityService::save_opportunity(
        &mut conn,
        SaveOpportunityWithCriteriaInput {
            opportunity: CreateOpportunityInput {
                title: Some("Survei FMCG".to_string()),
                platform_label: Some("Jakpat".to_string()),
                topic: Some("FMCG".to_string()),
                description: None,
                raw_text: "Tidak bekerja di riset pasar atau periklanan".to_string(),
                provenance: Some(OpportunityProvenance::ManualEntry),
                source_capture_id: None,
                source_ocr_id: None,
                captured_at: None,
            },
            criteria: vec![CriterionInput {
                id: None,
                criterion_text: "Bukan pekerja riset pasar atau periklanan".to_string(),
                raw_text: Some("Tidak bekerja di riset pasar atau periklanan".to_string()),
                criterion_type: CriterionType::Occupation,
                normalized_attribute: Some("occupation".to_string()),
                expected_value_json: Some(serde_json::json!([
                    "riset pasar",
                    "market research",
                    "periklanan",
                    "advertising"
                ])),
                operator: CriterionOperator::NotEquals,
                criterion_order: 0,
            }],
        },
    )
    .expect("opportunity save should succeed");

    // Persona programmer: harus Match (lolos eksklusi)
    let fit_allowed = FitService::evaluate_opportunity_fit(&mut conn, &opp.opportunity.id, p_allowed)
        .expect("fit evaluation should succeed");
    assert_eq!(fit_allowed.criteria_results[0].match_state, MatchState::Match);
    assert_eq!(fit_allowed.overall_fit, OverallFitState::Eligible);

    // Persona researcher: harus Mismatch (terkena eksklusi)
    let fit_excluded =
        FitService::evaluate_opportunity_fit(&mut conn, &opp.opportunity.id, p_excluded)
            .expect("fit evaluation should succeed");
    assert_eq!(
        fit_excluded.criteria_results[0].match_state,
        MatchState::Mismatch
    );
    assert_eq!(fit_excluded.overall_fit, OverallFitState::Ineligible);
}

/// Pengujian pemilihan responden jujur lintas persona (compare_personas_fit)
#[test]
fn test_cross_persona_truthful_respondent_selection() {
    let mut conn = setup_test_db();
    let p1 = "p-muda";
    let p2 = "p-senior";
    create_test_persona(&conn, p1, "Rian (Mahasiswa 21 th)", true);
    create_test_persona(&conn, p2, "Hendra (Pensiunan 60 th)", false);

    create_test_fact(
        &conn,
        "f-age-p1",
        p1,
        "demographics",
        "usia",
        "21",
        "21",
        "confirmed",
    );
    create_test_fact(
        &conn,
        "f-age-p2",
        p2,
        "demographics",
        "usia",
        "60",
        "60",
        "confirmed",
    );

    let opp = OpportunityService::save_opportunity(
        &mut conn,
        SaveOpportunityWithCriteriaInput {
            opportunity: CreateOpportunityInput {
                title: Some("Survei Gen Z".to_string()),
                platform_label: Some("Milieu".to_string()),
                topic: Some("Lifestyle".to_string()),
                description: None,
                raw_text: "Usia 18-25 tahun".to_string(),
                provenance: Some(OpportunityProvenance::ManualEntry),
                source_capture_id: None,
                source_ocr_id: None,
                captured_at: None,
            },
            criteria: vec![CriterionInput {
                id: None,
                criterion_text: "Usia 18 - 25 tahun".to_string(),
                raw_text: Some("Usia 18-25".to_string()),
                criterion_type: CriterionType::Age,
                normalized_attribute: Some("age".to_string()),
                expected_value_json: Some(serde_json::json!({"min": 18, "max": 25})),
                operator: CriterionOperator::Range,
                criterion_order: 0,
            }],
        },
    )
    .expect("opportunity save should succeed");

    let comparison = FitService::compare_personas_fit(&mut conn, &opp.opportunity.id)
        .expect("comparison should succeed");

    assert_eq!(comparison.len(), 2);

    // Persona yang memenuhi syarat (Rian) diurutkan paling awal untuk truthful respondent selection
    assert_eq!(comparison[0].persona_id, p1);
    assert_eq!(comparison[0].overall_fit, OverallFitState::Eligible);
    assert_eq!(comparison[0].match_count, 1);

    // Persona yang tidak memenuhi syarat (Hendra) ditandai Ineligible dengan alasan jelas
    assert_eq!(comparison[1].persona_id, p2);
    assert_eq!(comparison[1].overall_fit, OverallFitState::Ineligible);
    assert_eq!(comparison[1].mismatch_count, 1);
    assert!(comparison[1].key_reasons[0].contains("luar rentang"));
}

/// Pengujian persistensi dan pengambilan kembali hasil fit dari basis data
#[test]
fn test_persistence_and_retrieval_of_survey_fit_results() {
    let mut conn = setup_test_db();
    let persona_id = "p-persist";
    create_test_persona(&conn, persona_id, "Dewi", true);
    create_test_fact(
        &conn,
        "f-dewi-gender",
        persona_id,
        "demographics",
        "gender",
        "\"wanita\"",
        "wanita",
        "confirmed",
    );

    let opp = OpportunityService::save_opportunity(
        &mut conn,
        SaveOpportunityWithCriteriaInput {
            opportunity: CreateOpportunityInput {
                title: Some("Survei Kosmetik Wanita".to_string()),
                platform_label: Some("Toluna".to_string()),
                topic: Some("Kecantikan".to_string()),
                description: None,
                raw_text: "Khusus wanita".to_string(),
                provenance: Some(OpportunityProvenance::ManualEntry),
                source_capture_id: None,
                source_ocr_id: None,
                captured_at: None,
            },
            criteria: vec![CriterionInput {
                id: None,
                criterion_text: "Responden Wanita".to_string(),
                raw_text: Some("Khusus wanita".to_string()),
                criterion_type: CriterionType::Gender,
                normalized_attribute: Some("gender".to_string()),
                expected_value_json: Some(serde_json::json!(["wanita"])),
                operator: CriterionOperator::In,
                criterion_order: 0,
            }],
        },
    )
    .expect("opportunity save should succeed");

    // Simpan hasil evaluasi ke tabel survey_fit_results
    let evaluated = FitService::evaluate_opportunity_fit(&mut conn, &opp.opportunity.id, persona_id)
        .expect("fit evaluation should succeed");
    assert_eq!(evaluated.overall_fit, OverallFitState::Eligible);

    // Ambil kembali dari database menggunakan get_opportunity_fit
    let retrieved = FitService::get_opportunity_fit(&conn, &opp.opportunity.id, persona_id)
        .expect("query should succeed")
        .expect("fit results should exist");

    assert_eq!(retrieved.opportunity_id, opp.opportunity.id);
    assert_eq!(retrieved.persona_id, persona_id);
    assert_eq!(retrieved.overall_fit, OverallFitState::Eligible);
    assert_eq!(retrieved.criteria_results.len(), 1);
    assert_eq!(retrieved.criteria_results[0].match_state, MatchState::Match);
    assert_eq!(
        retrieved.criteria_results[0].supporting_fact_ids,
        vec!["f-dewi-gender"]
    );
}
