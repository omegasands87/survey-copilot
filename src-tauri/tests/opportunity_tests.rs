use rusqlite::Connection;
use survey_copilot_lib::domain::opportunity::{
    CreateOpportunityInput, CriterionInput, CriterionOperator, CriterionType,
    OpportunityProvenance, OpportunityStatus, SaveOpportunityWithCriteriaInput,
    UpdateCriteriaInput, UpdateOpportunityInput,
};
use survey_copilot_lib::migrations::MigrationRunner;
use survey_copilot_lib::repository::opportunity_repo::OpportunityRepository;
use survey_copilot_lib::services::criteria_extractor::CriteriaExtractorService;
use survey_copilot_lib::services::opportunity_service::OpportunityService;

fn setup_test_db() -> Connection {
    let mut conn = Connection::open_in_memory().expect("in-memory db should open");
    let runner = MigrationRunner::with_app_migrations();
    runner.run(&mut conn).expect("migrations should succeed");
    conn
}

#[test]
fn test_criteria_extraction_indonesian_survey_text() {
    let raw_text = r#"
        Survei Kepuasan Konsumen Minuman Kopi Kekinian
        Platform: YouGov
        Topik: Minuman & Konsumsi

        Target Responden:
        - Pria dan Wanita usia 20-35 tahun
        - Domisili Jabodetabek (Jakarta, Bogor, Depok, Tangerang, Bekasi)
        - Pekerjaan: Karyawan swasta atau wirausaha
        - Mengkonsumsi kopi kekinian minimal 2 kali seminggu
        - Bukan pekerja di bidang riset pasar atau periklanan
    "#;

    let extracted = CriteriaExtractorService::extract(raw_text, None);

    // 1. Traceable opportunity title & platform
    assert_eq!(
        extracted.title.as_deref(),
        Some("Survei Kepuasan Konsumen Minuman Kopi Kekinian")
    );
    assert_eq!(extracted.platform_label.as_deref(), Some("YouGov"));
    assert!(extracted.topic.is_some());

    // 2. Extracted criteria list
    assert!(
        extracted.criteria.len() >= 5,
        "Harus mengekstrak minimal 5 kriteria eksplisit, ditemukan {}",
        extracted.criteria.len()
    );

    // 3. Traceability: every criterion must have a non-empty raw_text matching original
    for c in &extracted.criteria {
        assert!(
            !c.raw_text.trim().is_empty(),
            "Raw text untuk kriteria '{}' tidak boleh kosong",
            c.criterion_text
        );
        assert!(
            raw_text.contains(&c.raw_text),
            "Raw text '{}' harus merupakan bagian dari teks sumber asli",
            c.raw_text
        );
    }

    // 4. Age criterion verification
    let age_crit = extracted
        .criteria
        .iter()
        .find(|c| c.criterion_type == CriterionType::Age)
        .expect("Kriteria usia harus terdeteksi");
    assert_eq!(age_crit.operator, CriterionOperator::Range);
    let age_val = age_crit.expected_value_json.as_ref().unwrap();
    assert_eq!(age_val.get("min").and_then(|v| v.as_i64()), Some(20));
    assert_eq!(age_val.get("max").and_then(|v| v.as_i64()), Some(35));

    // 5. Gender criterion verification
    let gender_crit = extracted
        .criteria
        .iter()
        .find(|c| c.criterion_type == CriterionType::Gender)
        .expect("Kriteria gender harus terdeteksi");
    assert_eq!(gender_crit.operator, CriterionOperator::In);

    // 6. Location criterion verification
    let loc_crit = extracted
        .criteria
        .iter()
        .find(|c| c.criterion_type == CriterionType::Location)
        .expect("Kriteria lokasi harus terdeteksi");
    let loc_val = loc_crit.expected_value_json.as_ref().unwrap();
    let locations = loc_val.get("locations").and_then(|v| v.as_array()).unwrap();
    assert!(locations.iter().any(|l| l.as_str() == Some("Jabodetabek")));

    // 7. Exclusion / NotEquals criterion verification
    let excl_crit = extracted
        .criteria
        .iter()
        .find(|c| c.operator == CriterionOperator::NotEquals)
        .expect("Kriteria eksklusi (bukan pekerja) harus terdeteksi dengan operator NotEquals");
    assert!(excl_crit.raw_text.contains("Bukan pekerja"));
}

#[test]
fn test_criteria_extraction_age_variants() {
    // Min age test
    let text_min = "Kriteria: Usia minimal 21 tahun";
    let ext_min = CriteriaExtractorService::extract(text_min, None);
    assert_eq!(ext_min.criteria.len(), 1);
    assert_eq!(ext_min.criteria[0].criterion_type, CriterionType::Age);
    assert_eq!(
        ext_min.criteria[0].operator,
        CriterionOperator::GreaterThanOrEqual
    );
    let val_min = ext_min.criteria[0].expected_value_json.as_ref().unwrap();
    assert_eq!(val_min.get("min").and_then(|v| v.as_i64()), Some(21));

    // Max age test
    let text_max = "Kriteria: Umur maksimal 45 tahun";
    let ext_max = CriteriaExtractorService::extract(text_max, None);
    assert_eq!(ext_max.criteria.len(), 1);
    assert_eq!(ext_max.criteria[0].criterion_type, CriterionType::Age);
    assert_eq!(
        ext_max.criteria[0].operator,
        CriterionOperator::LessThanOrEqual
    );
    let val_max = ext_max.criteria[0].expected_value_json.as_ref().unwrap();
    assert_eq!(val_max.get("max").and_then(|v| v.as_i64()), Some(45));
}

#[test]
fn test_no_hallucinated_criteria_for_simple_text() {
    let simple_text = "Survei Toluna tentang Penggunaan Smartphone.";
    let ext = CriteriaExtractorService::extract(simple_text, None);

    assert_eq!(ext.platform_label.as_deref(), Some("Toluna"));
    // Tidak boleh mengarang kriteria usia/gender/lokasi jika teks tidak memuatnya
    assert!(
        !ext.criteria
            .iter()
            .any(|c| c.criterion_type == CriterionType::Age),
        "Tidak boleh ada kriteria usia terfabrikasi jika tidak ada di teks sumber"
    );
    assert!(
        !ext.criteria
            .iter()
            .any(|c| c.criterion_type == CriterionType::Gender),
        "Tidak boleh ada kriteria gender terfabrikasi jika tidak ada di teks sumber"
    );
}

#[test]
fn test_opportunity_service_persistence_lifecycle() {
    let mut conn = setup_test_db();

    // 1. Ekstrak dari teks mentah
    let raw = r#"
        Survei Produk Otomotif 2026
        Platform: Milieu
        Syarat:
        - Usia 25-50 tahun
        - Memiliki kendaraan mobil roda empat
        - Domisili Surabaya
    "#;

    let preview = OpportunityService::extract_from_text(raw, None).unwrap();
    assert_eq!(preview.criteria.len(), 3);

    // 2. Simpan peluang survei ke basis data lokal
    let save_input = SaveOpportunityWithCriteriaInput {
        opportunity: CreateOpportunityInput {
            title: preview.opportunity.title,
            platform_label: preview.opportunity.platform_label,
            topic: preview.opportunity.topic,
            description: preview.opportunity.description,
            raw_text: raw.to_string(),
            provenance: Some(OpportunityProvenance::ManualEntry),
            source_capture_id: None,
            source_ocr_id: None,
            captured_at: None,
        },
        criteria: preview
            .criteria
            .into_iter()
            .map(|c| CriterionInput {
                id: None,
                criterion_text: c.criterion_text,
                raw_text: c.raw_text,
                criterion_type: c.criterion_type,
                normalized_attribute: c.normalized_attribute,
                expected_value_json: c.expected_value_json,
                operator: c.operator,
                criterion_order: c.criterion_order,
            })
            .collect(),
    };

    let saved = OpportunityService::save_opportunity(&mut conn, save_input).unwrap();
    assert_eq!(saved.opportunity.status, OpportunityStatus::Confirmed);
    assert_eq!(saved.criteria.len(), 3);

    // 3. Ambil detail peluang tersimpan
    let fetched = OpportunityService::get_opportunity(&conn, &saved.opportunity.id)
        .unwrap()
        .expect("Peluang survei harus ditemukan");
    assert_eq!(fetched.opportunity.title, saved.opportunity.title);
    assert_eq!(
        fetched.opportunity.platform_label.as_deref(),
        Some("Milieu")
    );
    assert_eq!(fetched.criteria.len(), 3);

    // 4. Perbarui metadata peluang
    let updated = OpportunityService::update_opportunity(
        &conn,
        UpdateOpportunityInput {
            id: saved.opportunity.id.clone(),
            title: Some("Survei Otomotif Surabaya (Revisi)".to_string()),
            platform_label: None,
            topic: Some("Otomotif & Transportasi".to_string()),
            description: Some("Keterangan tambahan".to_string()),
            status: Some(OpportunityStatus::Evaluated),
        },
    )
    .unwrap();
    assert_eq!(
        updated.title.as_deref(),
        Some("Survei Otomotif Surabaya (Revisi)")
    );
    assert_eq!(updated.status, OpportunityStatus::Evaluated);

    // 5. Perbarui kriteria
    let mut updated_criteria_input: Vec<CriterionInput> = fetched
        .criteria
        .into_iter()
        .map(|c| CriterionInput {
            id: Some(c.id),
            criterion_text: c.criterion_text,
            raw_text: c.raw_text,
            criterion_type: c.criterion_type,
            normalized_attribute: c.normalized_attribute,
            expected_value_json: c.expected_value_json,
            operator: c.operator,
            criterion_order: c.criterion_order,
        })
        .collect();

    // Tambahkan 1 kriteria baru
    updated_criteria_input.push(CriterionInput {
        id: None,
        criterion_text: "Pendidikan minimal D3/S1".to_string(),
        raw_text: Some("Input manual tambahan".to_string()),
        criterion_type: CriterionType::Education,
        normalized_attribute: Some("education".to_string()),
        expected_value_json: Some(serde_json::json!({ "min_level": "D3" })),
        operator: CriterionOperator::GreaterThanOrEqual,
        criterion_order: 3,
    });

    let new_crit_list = OpportunityService::update_criteria(
        &mut conn,
        UpdateCriteriaInput {
            opportunity_id: saved.opportunity.id.clone(),
            criteria: updated_criteria_input,
        },
    )
    .unwrap();
    assert_eq!(new_crit_list.len(), 4);

    // 6. Daftar peluang survei
    let list = OpportunityService::list_opportunities(&conn, Some(10), Some(0)).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, saved.opportunity.id);

    // 7. Hapus peluang survei
    let deleted = OpportunityService::delete_opportunity(&conn, &saved.opportunity.id).unwrap();
    assert!(deleted);

    // Kriteria harus ikut terhapus karena foreign key ON DELETE CASCADE
    let post_delete_crit =
        OpportunityRepository::get_criteria_by_opportunity_id(&conn, &saved.opportunity.id)
            .unwrap();
    assert!(
        post_delete_crit.is_empty(),
        "Kriteria harus terhapus secara kaskade"
    );
}

#[test]
fn test_opportunity_extraction_from_capture_and_ocr() {
    let conn = setup_test_db();

    // 1. Buat persona & sesi survei
    let persona_id = "test-persona-p5";
    conn.execute(
        "INSERT INTO personas (id, display_name, is_active, status, created_at, updated_at)
         VALUES (?1, 'Budi P5', 1, 'active', datetime('now'), datetime('now'))",
        rusqlite::params![persona_id],
    )
    .unwrap();

    let session_id = "test-session-p5";
    conn.execute(
        "INSERT INTO survey_sessions (id, persona_id, title, status, created_at, updated_at)
         VALUES (?1, ?2, 'Sesi Uji P5', 'active', datetime('now'), datetime('now'))",
        rusqlite::params![session_id, persona_id],
    )
    .unwrap();

    let capture_id = "test-capture-p5";
    conn.execute(
        "INSERT INTO screen_captures (id, session_id, capture_mode, captured_at, created_at)
         VALUES (?1, ?2, 'region', datetime('now'), datetime('now'))",
        rusqlite::params![capture_id, session_id],
    )
    .unwrap();

    let ocr_id = "test-ocr-p5";
    let ocr_text = "Riset Perilaku Belanja Online (Jakpat)\nKriteria:\n- Usia 18 - 40 tahun\n- Berdomisili di Bandung";
    conn.execute(
        "INSERT INTO ocr_extractions (
            id, capture_id, session_id, persona_id, ocr_engine, raw_ocr_text,
            question_text_original, options_json, text_blocks_json, uncertain_regions_json,
            warnings_json, extracted_at, created_at, updated_at
         ) VALUES (
            ?1, ?2, ?3, ?4, 'windows_media_ocr', ?5,
            ?5, '[]', '[]', '[]',
            '[]', datetime('now'), datetime('now'), datetime('now')
         )",
        rusqlite::params![ocr_id, capture_id, session_id, persona_id, ocr_text],
    )
    .unwrap();

    // 2. Ekstrak peluang dari tangkapan layar & OCR
    let extracted =
        OpportunityService::extract_from_capture(&conn, capture_id, Some(ocr_id)).unwrap();

    assert_eq!(
        extracted.opportunity.provenance,
        OpportunityProvenance::OcrDetected
    );
    assert_eq!(
        extracted.opportunity.source_capture_id.as_deref(),
        Some(capture_id)
    );
    assert_eq!(extracted.opportunity.source_ocr_id.as_deref(), Some(ocr_id));
    assert_eq!(
        extracted.opportunity.platform_label.as_deref(),
        Some("Jakpat")
    );
    assert_eq!(extracted.criteria.len(), 2);
}
