use std::path::Path;
use survey_copilot_lib::db::Database;
use survey_copilot_lib::domain::fact::{AddFactSourceInput, CreateFactInput, UpdateFactInput};
use survey_copilot_lib::domain::persona::CreatePersonaInput;
use survey_copilot_lib::services::{FactService, PersonaService};

fn setup_memory_db() -> Database {
    Database::initialize(Path::new(":memory:")).expect("memory database should initialize")
}

#[test]
fn test_at_020_create_typed_facts_atomic_with_source() {
    let db = setup_memory_db();
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Budi Santoso")).expect("p");

    // Number fact (age)
    let fact_age = FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "Demografi".into(),
            fact_key: Some("age".into()),
            attribute: "Usia".into(),
            data_type: Some("number".into()),
            value: "29".into(),
            unit: Some("tahun".into()),
            source_type: Some("uploaded_document".into()),
            source_title: Some("KTP".into()),
            source_description: "KTP Republik Indonesia".into(),
            reference_uri: None,
            observed_at: Some("2024-05-10".into()),
            valid_from: None,
            valid_until: None,
            notes: Some("Terverifikasi via identitas resmi".into()),
            verification_status: Some("confirmed".into()),
            sensitivity: Some("ordinary".into()),
        },
    )
    .expect("create age fact");

    assert_eq!(fact_age.data_type, "number");
    assert_eq!(fact_age.value, serde_json::json!(29));
    assert_eq!(fact_age.normalized_value, "29");
    assert_eq!(fact_age.unit, Some("tahun".into()));

    // Verify detail includes source and history
    let detail = FactService::get_detail(&db, &persona.id, &fact_age.id).expect("detail");
    assert_eq!(detail.sources.len(), 1);
    assert_eq!(detail.sources[0].title, "KTP");
    assert_eq!(detail.sources[0].source_type, "uploaded_document");
    assert_eq!(detail.history.len(), 1);
    assert_eq!(detail.history[0].new_status, "confirmed");
}

#[test]
fn test_typed_fact_validation_number_bool_date_json() {
    let db = setup_memory_db();
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Siti")).expect("p");

    // Invalid number must fail validation
    let bad_num = FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "Umum".into(),
            fact_key: None,
            attribute: "Nilai Angka".into(),
            data_type: Some("number".into()),
            value: "bukan-angka".into(),
            unit: None,
            source_type: None,
            source_title: None,
            source_description: "Input".into(),
            reference_uri: None,
            observed_at: None,
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: None,
            sensitivity: None,
        },
    );
    assert!(bad_num.is_err());
    assert_eq!(bad_num.unwrap_err().code, "VALIDATION_ERROR");

    // Invalid boolean must fail validation
    let bad_bool = FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "Umum".into(),
            fact_key: None,
            attribute: "Kepemilikan".into(),
            data_type: Some("boolean".into()),
            value: "mungkin".into(),
            unit: None,
            source_type: None,
            source_title: None,
            source_description: "Input".into(),
            reference_uri: None,
            observed_at: None,
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: None,
            sensitivity: None,
        },
    );
    assert!(bad_bool.is_err());

    // Invalid date must fail validation
    let bad_date = FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "Umum".into(),
            fact_key: None,
            attribute: "Tanggal".into(),
            data_type: Some("date".into()),
            value: "31-02-2024".into(), // non-ISO format
            unit: None,
            source_type: None,
            source_title: None,
            source_description: "Input".into(),
            reference_uri: None,
            observed_at: None,
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: None,
            sensitivity: None,
        },
    );
    assert!(bad_date.is_err());

    // Invalid JSON must fail validation
    let bad_json = FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "Umum".into(),
            fact_key: None,
            attribute: "Data Struktur".into(),
            data_type: Some("json".into()),
            value: "{ bad json }".into(),
            unit: None,
            source_type: None,
            source_title: None,
            source_description: "Input".into(),
            reference_uri: None,
            observed_at: None,
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: None,
            sensitivity: None,
        },
    );
    assert!(bad_json.is_err());
}

#[test]
fn test_at_021_verification_lifecycle_unverified_disputed_outdated() {
    let db = setup_memory_db();
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Rian")).expect("p");

    // 1. New fact defaults to unverified
    let fact = FactService::create(
        &db,
        CreateFactInput::new(
            &persona.id,
            "Pekerjaan",
            "Profesi",
            "Staff IT",
            "Input pengguna",
        ),
    )
    .expect("create");
    assert_eq!(fact.verification_status, "unverified");

    // 2. Confirmed by explicit user action
    let confirmed =
        FactService::set_status(&db, &persona.id, &fact.id, "confirmed").expect("confirm");
    assert_eq!(confirmed.verification_status, "confirmed");

    // 3. Mark disputed
    let disputed =
        FactService::set_status(&db, &persona.id, &fact.id, "disputed").expect("disputed");
    assert_eq!(disputed.verification_status, "disputed");

    // Must be excluded from AI eligible evidence
    let eligible1 =
        FactService::get_eligible_facts_for_ai(&db, &persona.id, true).expect("eligible");
    assert_eq!(eligible1.len(), 0);

    // 4. Mark outdated
    let outdated =
        FactService::set_status(&db, &persona.id, &fact.id, "outdated").expect("outdated");
    assert_eq!(outdated.verification_status, "outdated");

    // Outdated fact is also excluded from AI eligible evidence
    let eligible2 =
        FactService::get_eligible_facts_for_ai(&db, &persona.id, true).expect("eligible");
    assert_eq!(eligible2.len(), 0);

    // Verify history contains all transitions
    let detail = FactService::get_detail(&db, &persona.id, &fact.id).expect("detail");
    assert_eq!(detail.history.len(), 4);
    assert_eq!(detail.history[0].new_status, "outdated");
    assert_eq!(detail.history[1].new_status, "disputed");
    assert_eq!(detail.history[2].new_status, "confirmed");
    assert_eq!(detail.history[3].new_status, "unverified");
}

#[test]
fn test_at_023_sensitive_fact_requires_consent() {
    let db = setup_memory_db();
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Hartono")).expect("p");

    // Ordinary confirmed fact
    FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "Demografi".into(),
            fact_key: Some("city".into()),
            attribute: "Kota Domisili".into(),
            data_type: Some("string".into()),
            value: "Jakarta Selatan".into(),
            unit: None,
            source_type: None,
            source_title: None,
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
    .expect("create ordinary");

    // Sensitive confirmed fact (monthly income)
    FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "Finansial".into(),
            fact_key: Some("monthly_income".into()),
            attribute: "Penghasilan Bulanan".into(),
            data_type: Some("number".into()),
            value: "25000000".into(),
            unit: Some("IDR".into()),
            source_type: None,
            source_title: None,
            source_description: "Slip Gaji".into(),
            reference_uri: None,
            observed_at: None,
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".into()),
            sensitivity: Some("sensitive".into()),
        },
    )
    .expect("create sensitive");

    // Without consent -> only ordinary fact returned
    let without_consent =
        FactService::get_eligible_facts_for_ai(&db, &persona.id, false).expect("no consent");
    assert_eq!(without_consent.len(), 1);
    assert_eq!(without_consent[0].attribute, "Kota Domisili");

    // With explicit consent -> both returned
    let with_consent =
        FactService::get_eligible_facts_for_ai(&db, &persona.id, true).expect("with consent");
    assert_eq!(with_consent.len(), 2);
}

#[test]
fn test_conflict_detection_and_resolution() {
    let db = setup_memory_db();
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Doni")).expect("p");

    // Fact 1: Job = Akuntan
    let f1 = FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "Pekerjaan".into(),
            fact_key: Some("pekerjaan".into()),
            attribute: "Pekerjaan".into(),
            data_type: Some("string".into()),
            value: "Akuntan".into(),
            unit: None,
            source_type: None,
            source_title: None,
            source_description: "Input 2022".into(),
            reference_uri: None,
            observed_at: None,
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".into()),
            sensitivity: Some("ordinary".into()),
        },
    )
    .expect("f1");

    // Fact 2: Job = Konsultan Keuangan (contrary fact on same key)
    let f2 = FactService::create(
        &db,
        CreateFactInput {
            persona_id: persona.id.clone(),
            category: "Pekerjaan".into(),
            fact_key: Some("pekerjaan".into()),
            attribute: "Pekerjaan".into(),
            data_type: Some("string".into()),
            value: "Konsultan Keuangan".into(),
            unit: None,
            source_type: None,
            source_title: None,
            source_description: "Input 2024".into(),
            reference_uri: None,
            observed_at: None,
            valid_from: None,
            valid_until: None,
            notes: None,
            verification_status: Some("confirmed".into()),
            sensitivity: Some("ordinary".into()),
        },
    )
    .expect("f2");

    // System detects conflict
    let conflicts = FactService::detect_conflicts(&db, &persona.id).expect("conflicts");
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].fact_key, "pekerjaan");

    // Conflicting facts are blocked from AI recommendation
    let eligible = FactService::get_eligible_facts_for_ai(&db, &persona.id, false).expect("ai");
    assert_eq!(eligible.len(), 0);

    // User resolves conflict by marking old fact (f1) outdated
    FactService::set_status(&db, &persona.id, &f1.id, "outdated").expect("resolve");

    // Conflict resolved
    let conflicts_after = FactService::detect_conflicts(&db, &persona.id).expect("conflicts after");
    assert_eq!(conflicts_after.len(), 0);

    // f2 is now eligible for AI
    let eligible_after =
        FactService::get_eligible_facts_for_ai(&db, &persona.id, false).expect("ai after");
    assert_eq!(eligible_after.len(), 1);
    assert_eq!(eligible_after[0].id, f2.id);
    assert_eq!(
        eligible_after[0].value,
        serde_json::json!("Konsultan Keuangan")
    );
}

#[test]
fn test_source_management_and_multi_provenance() {
    let db = setup_memory_db();
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Linda")).expect("p");

    let fact = FactService::create(
        &db,
        CreateFactInput::new(
            &persona.id,
            "Kesehatan",
            "Alergi",
            "Kacang",
            "Pernyataan pengguna",
        ),
    )
    .expect("fact");

    // Add second supporting source
    let source2 = FactService::add_source(
        &db,
        AddFactSourceInput {
            persona_id: persona.id.clone(),
            fact_id: fact.id.clone(),
            source_type: "uploaded_document".into(),
            title: "Surat Dokter".into(),
            description: "Hasil tes alergi laboratorium".into(),
            reference_uri: Some("docs/tes_lab.pdf".into()),
            observed_at: Some("2024-03-15".into()),
            valid_from: None,
            valid_until: None,
            notes: Some("Tes alergi IgE".into()),
        },
    )
    .expect("add source");

    assert_eq!(source2.title, "Surat Dokter");

    let detail = FactService::get_detail(&db, &persona.id, &fact.id).expect("detail");
    assert_eq!(detail.sources.len(), 2);
    assert_eq!(detail.sources[0].title, "Pernyataan pengguna");
    assert_eq!(detail.sources[1].title, "Surat Dokter");
}

#[test]
fn test_persona_isolation_strictly_enforced_for_facts_sources_and_history() {
    let db = setup_memory_db();
    let persona_a = PersonaService::create(&db, CreatePersonaInput::new("Orang A")).expect("pA");
    let persona_b = PersonaService::create(&db, CreatePersonaInput::new("Orang B")).expect("pB");

    let fact_a = FactService::create(
        &db,
        CreateFactInput::new(
            &persona_a.id,
            "Finansial",
            "Tabungan",
            "50.000.000",
            "Buku Rekening",
        ),
    )
    .expect("fact A");

    // 1. Persona B list -> 0 facts
    let list_b = FactService::list(&db, &persona_b.id, false).expect("list B");
    assert_eq!(list_b.len(), 0);

    // 2. Persona B get detail -> NOT_FOUND
    let get_b = FactService::get_detail(&db, &persona_b.id, &fact_a.id);
    assert!(get_b.is_err());
    assert_eq!(get_b.unwrap_err().code, "NOT_FOUND");

    // 3. Persona B update -> NOT_FOUND
    let update_b = FactService::update(
        &db,
        UpdateFactInput {
            persona_id: persona_b.id.clone(),
            fact_id: fact_a.id.clone(),
            category: None,
            attribute: None,
            data_type: None,
            value: Some("100".into()),
            unit: None,
            valid_from: None,
            valid_until: None,
            notes: None,
            change_reason: None,
            verification_status: None,
            sensitivity: None,
        },
    );
    assert!(update_b.is_err());
    assert_eq!(update_b.unwrap_err().code, "NOT_FOUND");

    // 4. Persona B set status -> NOT_FOUND
    let status_b = FactService::set_status(&db, &persona_b.id, &fact_a.id, "confirmed");
    assert!(status_b.is_err());
    assert_eq!(status_b.unwrap_err().code, "NOT_FOUND");

    // 5. Persona B add source -> NOT_FOUND or foreign key constraint
    let src_b = FactService::add_source(
        &db,
        AddFactSourceInput {
            persona_id: persona_b.id.clone(),
            fact_id: fact_a.id.clone(),
            source_type: "manual_entry".into(),
            title: "Palsu".into(),
            description: "Mencoba menyusup".into(),
            reference_uri: None,
            observed_at: None,
            valid_from: None,
            valid_until: None,
            notes: None,
        },
    );
    assert!(src_b.is_err());

    // 6. Persona B delete -> NOT_FOUND
    let delete_b = FactService::delete(&db, &persona_b.id, &fact_a.id);
    assert!(delete_b.is_err());
    assert_eq!(delete_b.unwrap_err().code, "NOT_FOUND");
}

#[test]
fn test_at_022_fact_delete_cascades_sources_and_history() {
    let db = setup_memory_db();
    let persona = PersonaService::create(&db, CreatePersonaInput::new("Tono")).expect("p");

    let fact = FactService::create(
        &db,
        CreateFactInput::new(
            &persona.id,
            "Umum",
            "Warna Favorit",
            "Biru",
            "Input pengguna",
        ),
    )
    .expect("create");

    // Verify detail has 1 source and 1 history
    let detail_before = FactService::get_detail(&db, &persona.id, &fact.id).expect("detail");
    assert_eq!(detail_before.sources.len(), 1);
    assert_eq!(detail_before.history.len(), 1);

    // Delete fact
    FactService::delete(&db, &persona.id, &fact.id).expect("delete");

    // Fact must no longer exist
    assert!(FactService::get_detail(&db, &persona.id, &fact.id).is_err());

    // In database, sources and history must be cleanly removed (0 orphan records)
    let conn = db.connection.lock().expect("lock");
    let src_count: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM fact_sources WHERE fact_id = ?1",
            [&fact.id],
            |r| r.get(0),
        )
        .expect("query");
    assert_eq!(src_count, 0);

    let hist_count: i64 = conn
        .query_row(
            "SELECT COUNT(1) FROM fact_history WHERE fact_id = ?1",
            [&fact.id],
            |r| r.get(0),
        )
        .expect("query");
    assert_eq!(hist_count, 0);
}
