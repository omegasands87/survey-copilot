use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use rusqlite::Connection;
use survey_copilot_lib::db::Database;
use survey_copilot_lib::domain::capture::CaptureMode;
use survey_copilot_lib::domain::ocr::{
    BoundingBox, ConfirmOcrExtractionInput, ExtractOcrInput, OcrLine, OcrRawResult, OcrWord,
    UncertaintyReason,
};
use survey_copilot_lib::domain::survey::{AnswerFormat, Provenance, QuestionType};
use survey_copilot_lib::infrastructure::capture::encode_rgba_to_png;
use survey_copilot_lib::infrastructure::ocr::composite::CompositeOcrEngine;
use survey_copilot_lib::infrastructure::ocr::fixture::FixtureOcrEngine;
use survey_copilot_lib::infrastructure::ocr::OcrEngine;
use survey_copilot_lib::migrations::MigrationRunner;
use survey_copilot_lib::services::layout_reconstructor::LayoutReconstructorService;
use survey_copilot_lib::services::ocr_service::OcrService;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get_unique_temp_dir() -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("survey_copilot_ocr_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir
}

/// Sets up a file-backed SQLite database with all migrations applied.
/// Returns `(Database, temp_dir_path)` — caller must keep `temp_dir_path` alive (not dropped)
/// so the database file is not deleted prematurely.
fn setup_test_db() -> (Database, PathBuf) {
    let dir = get_unique_temp_dir();
    let db_path = dir.join("test_ocr.sqlite3");
    let runner = MigrationRunner::with_app_migrations();
    let db = Database::initialize_with_runner(&db_path, &runner).expect("db should initialize");
    (db, dir)
}

fn create_test_persona(conn: &Connection, name: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO personas (id, display_name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, name, now, now],
    )
    .expect("persona should insert");
    id
}

fn create_test_session(conn: &Connection, persona_id: &str, title: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO survey_sessions (id, persona_id, title, status, language, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'active', 'id', ?4, ?5)",
        rusqlite::params![id, persona_id, title, now, now],
    )
    .expect("session should insert");
    id
}

/// Creates a valid synthetic PNG using the project's encode_rgba_to_png helper.
fn create_synthetic_png(width: u32, height: u32) -> Vec<u8> {
    let pixels = vec![255u8; (width * height * 4) as usize];
    encode_rgba_to_png(width, height, &pixels).expect("png encode should succeed")
}

// ==========================================
// TEST 1: INDONESIAN SINGLE CHOICE
// ==========================================
#[test]
fn test_ocr_single_choice_indonesian() {
    let line1 =
        FixtureOcrEngine::build_line("1. Berapa usia Anda saat ini?", 20.0, 20.0, 20.0, 0.98);
    let line2 = FixtureOcrEngine::build_line("(Pilih satu jawaban)", 50.0, 16.0, 20.0, 0.95);
    let line3 = FixtureOcrEngine::build_line("A. 18 - 24 tahun", 80.0, 18.0, 30.0, 0.97);
    let line4 = FixtureOcrEngine::build_line("B. 25 - 34 tahun", 110.0, 18.0, 30.0, 0.96);
    let line5 = FixtureOcrEngine::build_line("C. 35 - 44 tahun", 140.0, 18.0, 30.0, 0.97);

    let raw = OcrRawResult {
        raw_text: "1. Berapa usia Anda saat ini?\n(Pilih satu jawaban)\nA. 18 - 24 tahun\nB. 25 - 34 tahun\nC. 35 - 44 tahun".to_string(),
        text_angle: 0.0,
        lines: vec![line1, line2, line3, line4, line5],
        engine: "fixture_ocr".to_string(),
        image_width: 800,
        image_height: 600,
    };

    let reconstructed = LayoutReconstructorService::reconstruct(
        &raw,
        Some("test-cap-1".to_string()),
        None,
        None,
        CaptureMode::Region,
        None,
        None,
    )
    .expect("reconstruction should succeed");

    assert_eq!(
        reconstructed.question_text_original,
        "Berapa usia Anda saat ini?"
    );
    assert_eq!(
        reconstructed.instructions,
        Some("(Pilih satu jawaban)".to_string())
    );
    assert_eq!(reconstructed.answer_options.len(), 3);
    assert_eq!(reconstructed.answer_options[0], "18 - 24 tahun");
    assert_eq!(reconstructed.answer_options[1], "25 - 34 tahun");
    assert_eq!(reconstructed.answer_options[2], "35 - 44 tahun");
    assert_eq!(reconstructed.question_type, QuestionType::SingleChoice);
    assert_eq!(reconstructed.answer_format, AnswerFormat::SingleChoice);
    assert_eq!(reconstructed.visible_units, Some("tahun".to_string()));
    assert!(reconstructed.uncertain_regions.is_empty());
}

// ==========================================
// TEST 2: INDONESIAN MULTIPLE CHOICE
// ==========================================
#[test]
fn test_ocr_multiple_choice_indonesian() {
    let line1 = FixtureOcrEngine::build_line(
        "2. Produk apa saja yang Anda beli bulan ini?",
        20.0,
        20.0,
        20.0,
        0.98,
    );
    let line2 = FixtureOcrEngine::build_line("(Pilih semua yang sesuai)", 50.0, 16.0, 20.0, 0.95);
    let line3 = FixtureOcrEngine::build_line("[ ] Kopi Instan", 80.0, 18.0, 30.0, 0.97);
    let line4 = FixtureOcrEngine::build_line("[ ] Teh Botol", 110.0, 18.0, 30.0, 0.96);
    let line5 = FixtureOcrEngine::build_line("[ ] Susu Formula", 140.0, 18.0, 30.0, 0.97);
    let line6 = FixtureOcrEngine::build_line("[ ] Minyak Goreng", 170.0, 18.0, 30.0, 0.96);

    let raw = OcrRawResult {
        raw_text: "2. Produk apa saja yang Anda beli bulan ini?\n(Pilih semua yang sesuai)\n[ ] Kopi Instan\n[ ] Teh Botol\n[ ] Susu Formula\n[ ] Minyak Goreng".to_string(),
        text_angle: 0.0,
        lines: vec![line1, line2, line3, line4, line5, line6],
        engine: "fixture_ocr".to_string(),
        image_width: 800,
        image_height: 600,
    };

    let reconstructed = LayoutReconstructorService::reconstruct(
        &raw,
        Some("test-cap-2".to_string()),
        None,
        None,
        CaptureMode::Region,
        None,
        None,
    )
    .expect("reconstruction should succeed");

    assert_eq!(
        reconstructed.question_text_original,
        "Produk apa saja yang Anda beli bulan ini?"
    );
    assert_eq!(
        reconstructed.instructions,
        Some("(Pilih semua yang sesuai)".to_string())
    );
    assert_eq!(reconstructed.answer_options.len(), 4);
    assert_eq!(reconstructed.answer_options[0], "Kopi Instan");
    assert_eq!(reconstructed.answer_options[1], "Teh Botol");
    assert_eq!(reconstructed.answer_options[2], "Susu Formula");
    assert_eq!(reconstructed.answer_options[3], "Minyak Goreng");
    assert_eq!(reconstructed.question_type, QuestionType::MultipleChoice);
    assert_eq!(reconstructed.answer_format, AnswerFormat::MultipleSelection);
    assert_eq!(reconstructed.visible_units, Some("bulan".to_string()));
}

// ==========================================
// TEST 3: INDONESIAN RATING SCALE
// ==========================================
#[test]
fn test_ocr_rating_scale_indonesian() {
    let line1 = FixtureOcrEngine::build_line(
        "3. Seberapa puas Anda dengan kecepatan pelayanan kami? *",
        20.0,
        20.0,
        20.0,
        0.99,
    );
    let line2 = FixtureOcrEngine::build_line(
        "1 (Sangat Tidak Puas) sampai 5 (Sangat Puas)",
        50.0,
        16.0,
        20.0,
        0.96,
    );
    let line3 = FixtureOcrEngine::build_line("1 2 3 4 5", 80.0, 18.0, 40.0, 0.99);

    let raw = OcrRawResult {
        raw_text: "3. Seberapa puas Anda dengan kecepatan pelayanan kami? *\n1 (Sangat Tidak Puas) sampai 5 (Sangat Puas)\n1 2 3 4 5".to_string(),
        text_angle: 0.0,
        lines: vec![line1, line2, line3],
        engine: "fixture_ocr".to_string(),
        image_width: 800,
        image_height: 600,
    };

    let reconstructed = LayoutReconstructorService::reconstruct(
        &raw,
        Some("test-cap-3".to_string()),
        None,
        None,
        CaptureMode::Region,
        None,
        None,
    )
    .expect("reconstruction should succeed");

    assert_eq!(
        reconstructed.question_text_original,
        "Seberapa puas Anda dengan kecepatan pelayanan kami?"
    );
    assert_eq!(reconstructed.question_type, QuestionType::Scale);
    assert_eq!(reconstructed.answer_format, AnswerFormat::RatingScale);
    assert_eq!(reconstructed.is_required, Some(true));

    let scale = reconstructed.scale.expect("scale must be detected");
    assert_eq!(scale.min_value, 1);
    assert_eq!(scale.max_value, 5);
    assert_eq!(scale.min_label, Some("Sangat Tidak Puas".to_string()));
    assert_eq!(scale.max_label, Some("Sangat Puas".to_string()));
    assert_eq!(scale.all_labels, vec!["1", "2", "3", "4", "5"]);
}

// ==========================================
// TEST 4: ENGLISH RATING SCALE
// ==========================================
#[test]
fn test_ocr_rating_scale_english() {
    let line1 = FixtureOcrEngine::build_line(
        "Please rate your overall experience with our customer support",
        20.0,
        20.0,
        20.0,
        0.98,
    );
    let line2 = FixtureOcrEngine::build_line(
        "From 1 (Very Dissatisfied) to 10 (Very Satisfied)",
        50.0,
        16.0,
        20.0,
        0.97,
    );
    let line3 = FixtureOcrEngine::build_line("1 2 3 4 5 6 7 8 9 10", 80.0, 18.0, 40.0, 0.99);

    let raw = OcrRawResult {
        raw_text: "Please rate your overall experience with our customer support\nFrom 1 (Very Dissatisfied) to 10 (Very Satisfied)\n1 2 3 4 5 6 7 8 9 10".to_string(),
        text_angle: 0.0,
        lines: vec![line1, line2, line3],
        engine: "fixture_ocr".to_string(),
        image_width: 800,
        image_height: 600,
    };

    let reconstructed = LayoutReconstructorService::reconstruct(
        &raw,
        Some("test-cap-4".to_string()),
        None,
        None,
        CaptureMode::Region,
        None,
        None,
    )
    .expect("reconstruction should succeed");

    assert_eq!(
        reconstructed.question_text_original,
        "Please rate your overall experience with our customer support"
    );
    assert_eq!(reconstructed.question_type, QuestionType::Scale);
    assert_eq!(reconstructed.answer_format, AnswerFormat::RatingScale);

    let scale = reconstructed.scale.expect("scale must be detected");
    assert_eq!(scale.min_value, 1);
    assert_eq!(scale.max_value, 10);
    assert_eq!(scale.min_label, Some("Very Dissatisfied".to_string()));
    assert_eq!(scale.max_label, Some("Very Satisfied".to_string()));
}

// ==========================================
// TEST 5: COMPLEX MATRIX / GRID TABLE LAYOUT
// ==========================================
#[test]
fn test_ocr_matrix_grid_complex_layout() {
    let line1 = FixtureOcrEngine::build_line(
        "4. Berikan penilaian Anda untuk setiap aspek berikut ini:",
        20.0,
        20.0,
        20.0,
        0.99,
    );
    let line2 = FixtureOcrEngine::build_line(
        "Sangat Buruk, Buruk, Cukup, Baik, Sangat Baik",
        60.0,
        18.0,
        200.0,
        0.96,
    );
    let line3 =
        FixtureOcrEngine::build_line("Kualitas produk yang diterima", 90.0, 18.0, 20.0, 0.97);
    let line4 = FixtureOcrEngine::build_line(
        "Kecepatan waktu pengiriman pesanan",
        120.0,
        18.0,
        20.0,
        0.98,
    );
    let line5 =
        FixtureOcrEngine::build_line("Kemudahan navigasi aplikasi", 150.0, 18.0, 20.0, 0.97);
    let line6 =
        FixtureOcrEngine::build_line("Responsivitas layanan pelanggan", 180.0, 18.0, 20.0, 0.98);

    let raw = OcrRawResult {
        raw_text: "4. Berikan penilaian Anda untuk setiap aspek berikut ini:\nSangat Buruk, Buruk, Cukup, Baik, Sangat Baik\nKualitas produk yang diterima\nKecepatan waktu pengiriman pesanan\nKemudahan navigasi aplikasi\nResponsivitas layanan pelanggan".to_string(),
        text_angle: 0.0,
        lines: vec![line1, line2, line3, line4, line5, line6],
        engine: "fixture_ocr".to_string(),
        image_width: 800,
        image_height: 600,
    };

    let reconstructed = LayoutReconstructorService::reconstruct(
        &raw,
        Some("test-cap-5".to_string()),
        None,
        None,
        CaptureMode::Region,
        None,
        None,
    )
    .expect("reconstruction should succeed");

    assert_eq!(
        reconstructed.question_text_original,
        "Berikan penilaian Anda untuk setiap aspek berikut ini:"
    );
    assert_eq!(reconstructed.question_type, QuestionType::Matrix);
    assert_eq!(reconstructed.answer_format, AnswerFormat::Matrix);

    let matrix = reconstructed
        .matrix
        .expect("matrix grid must be reconstructed");
    assert_eq!(matrix.columns.len(), 5);
    assert_eq!(matrix.columns[0], "Sangat Buruk");
    assert_eq!(matrix.columns[1], "Buruk");
    assert_eq!(matrix.columns[2], "Cukup");
    assert_eq!(matrix.columns[3], "Baik");
    assert_eq!(matrix.columns[4], "Sangat Baik");

    assert_eq!(matrix.rows.len(), 4);
    assert_eq!(matrix.rows[0], "Kualitas produk yang diterima");
    assert_eq!(matrix.rows[1], "Kecepatan waktu pengiriman pesanan");
    assert_eq!(matrix.rows[2], "Kemudahan navigasi aplikasi");
    assert_eq!(matrix.rows[3], "Responsivitas layanan pelanggan");

    // Pastikan kisi sel (cell_matrix) berdimensi 4 rows x 5 columns
    assert_eq!(matrix.cell_matrix.len(), 4);
    for row in &matrix.cell_matrix {
        assert_eq!(row.len(), 5);
    }
}

// ==========================================
// TEST 6: CLIPPED TEXT & LOW CONFIDENCE DETECTION (AT-052)
// ==========================================
#[test]
fn test_ocr_clipped_and_low_confidence_detection_at_052() {
    let img_w = 400.0;
    let img_h = 200.0;

    // Kata 'Karakteristik' menyentuh batas kanan (x=330, width=68 -> right=398 >= 395)
    let word_clipped = OcrWord {
        text: "Karakteristik".to_string(),
        bounding_box: BoundingBox::new(330.0, 20.0, 68.0, 18.0),
        confidence: 0.95,
    };

    // Kata 'buram' dengan keyakinan rendah (< 0.70)
    let word_low_conf = OcrWord {
        text: "buram".to_string(),
        bounding_box: BoundingBox::new(100.0, 60.0, 45.0, 18.0),
        confidence: 0.52,
    };

    let line1 = OcrLine {
        text: "Pertanyaan dengan teks Karakteristik".to_string(),
        bounding_box: BoundingBox::new(20.0, 20.0, 378.0, 18.0),
        words: vec![word_clipped],
        confidence: 0.95,
    };

    let line2 = OcrLine {
        text: "Opsi buram".to_string(),
        bounding_box: BoundingBox::new(20.0, 60.0, 125.0, 18.0),
        words: vec![word_low_conf],
        confidence: 0.52,
    };

    let raw = OcrRawResult {
        raw_text: "Pertanyaan dengan teks Karakteristik\nOpsi buram".to_string(),
        text_angle: 0.0,
        lines: vec![line1, line2],
        engine: "fixture_ocr".to_string(),
        image_width: img_w as u32,
        image_height: img_h as u32,
    };

    let reconstructed = LayoutReconstructorService::reconstruct(
        &raw,
        Some("test-cap-6".to_string()),
        None,
        None,
        CaptureMode::Region,
        None,
        None,
    )
    .expect("reconstruction should succeed");

    // AT-052: Verifikasi deteksi clipping dan low confidence
    assert!(
        reconstructed
            .uncertain_regions
            .iter()
            .any(|u| u.reason == UncertaintyReason::LikelyClipped),
        "Harus mendeteksi teks terpotong di tepi gambar"
    );

    assert!(
        reconstructed
            .uncertain_regions
            .iter()
            .any(|u| u.reason == UncertaintyReason::LowConfidence),
        "Harus mendeteksi keyakinan rendah (< 0.70)"
    );

    assert!(
        !reconstructed.warnings.is_empty(),
        "Harus menghasilkan peringatan kualitas (warnings)"
    );
    assert!(
        reconstructed.overall_confidence < 0.70,
        "Keyakinan agregat harus dipenalti"
    );
}

#[test]
fn test_composite_ocr_without_fixture_fallback_never_returns_synthetic_text() {
    let unavailable_primary = FixtureOcrEngine::new();
    unavailable_primary.set_available(false);
    let engine = CompositeOcrEngine::new(Arc::new(unavailable_primary), None);

    assert!(!engine.is_available());
    let error = engine
        .recognize_file(std::path::Path::new("missing-capture.png"))
        .expect_err("unavailable production OCR must fail rather than fabricate text");
    assert!(error.message.contains("OCR") || error.message.contains("engine"));
}

#[test]
fn test_ocr_zero_confidence_is_reported_as_unavailable_not_low_confidence() {
    let line = OcrLine {
        text: "Pertanyaan hasil OCR".to_string(),
        bounding_box: BoundingBox::new(10.0, 10.0, 220.0, 20.0),
        words: vec![OcrWord {
            text: "Pertanyaan".to_string(),
            bounding_box: BoundingBox::new(10.0, 10.0, 100.0, 20.0),
            confidence: 0.0,
        }],
        confidence: 0.0,
    };
    let raw = OcrRawResult {
        raw_text: "Pertanyaan hasil OCR".to_string(),
        text_angle: 0.0,
        lines: vec![line],
        engine: "windows_media_ocr".to_string(),
        image_width: 800,
        image_height: 600,
    };
    let reconstructed = LayoutReconstructorService::reconstruct(
        &raw,
        None,
        None,
        None,
        CaptureMode::Region,
        None,
        None,
    )
    .expect("layout reconstruction should succeed");

    assert_eq!(reconstructed.overall_confidence, 0.0);
    assert!(!reconstructed
        .uncertain_regions
        .iter()
        .any(|region| region.reason == UncertaintyReason::LowConfidence));
    assert!(reconstructed
        .warnings
        .iter()
        .any(|warning| warning.contains("tidak menyediakan skor keyakinan")));
}

// ==========================================
// TEST 7: OCR SERVICE UNAVAILABLE FALLBACK (AT-053)
// ==========================================
#[test]
fn test_ocr_service_unavailable_at_053() {
    let (db, _db_dir) = setup_test_db();
    let capture_dir = get_unique_temp_dir();

    let fixture_engine = Arc::new(FixtureOcrEngine::new());
    // Simulasikan engine tidak tersedia
    fixture_engine.set_available(false);

    let ocr_service = OcrService::new(fixture_engine, capture_dir.clone(), Arc::new(db));

    // Buat berkas dummy capture
    let capture_id = uuid::Uuid::new_v4().to_string();
    let file_path = capture_dir.join(format!("capture_{capture_id}.png"));
    fs::write(&file_path, create_synthetic_png(100, 100)).unwrap();

    let input = ExtractOcrInput {
        capture_id,
        session_id: None,
        persona_id: None,
        engine: None,
    };

    let result = ocr_service.extract_from_capture(input);
    assert!(
        result.is_err(),
        "Harus mengembalikan error ketika OCR unavailable"
    );
    let err = result.unwrap_err();
    assert_eq!(err.code, "SERVICE_UNAVAILABLE");
    // AT-053: Pesan kesalahan mengarahkan pengguna bahwa input teks manual tetap dapat digunakan
    assert!(err.message.contains("Layanan OCR"));
}

// ==========================================
// TEST 8: SOURCE TEXT & SCREENSHOT IMMUTABILITY
// ==========================================
#[test]
fn test_ocr_source_text_and_screenshot_immutability() {
    let (db, _db_dir) = setup_test_db();
    let capture_dir = get_unique_temp_dir();

    let capture_id = uuid::Uuid::new_v4().to_string();
    let file_path = capture_dir.join(format!("capture_{capture_id}.png"));
    let png_bytes = create_synthetic_png(200, 100);
    fs::write(&file_path, &png_bytes).unwrap();

    let fixture_engine = Arc::new(FixtureOcrEngine::new());
    let ocr_service = OcrService::new(fixture_engine, capture_dir, Arc::new(db));

    let input = ExtractOcrInput {
        capture_id: capture_id.clone(),
        session_id: None,
        persona_id: None,
        engine: None,
    };

    let extracted = ocr_service.extract_from_capture(input).unwrap();
    let original_text_before = extracted.question_text_original.clone();
    let raw_text_before = extracted.raw_ocr_text.clone();

    // Verifikasi berkas fisik tangkapan layar sama sekali tidak dimodifikasi
    let current_bytes = fs::read(&file_path).unwrap();
    assert_eq!(
        current_bytes, png_bytes,
        "Gambar screenshot asli tidak boleh diubah"
    );

    // Pastikan teks asli tetap ada dan tidak diubah saat ekstraksi
    assert!(!original_text_before.is_empty());
    assert!(!raw_text_before.is_empty());
}

// ==========================================
// TEST 9: CONFIRM EXTRACTION & PROVENANCE INTEGRATION
// ==========================================
#[test]
fn test_ocr_confirm_extraction_records_question_with_provenance() {
    let (db, _db_dir) = setup_test_db();
    let capture_dir = get_unique_temp_dir();

    let persona_id = {
        let conn = db.connection.lock().unwrap();
        create_test_persona(&conn, "Ahmad Ramadhan")
    };

    let session_id = {
        let conn = db.connection.lock().unwrap();
        create_test_session(&conn, &persona_id, "Survei Kepuasan Konsumen 2026")
    };

    let capture_id = uuid::Uuid::new_v4().to_string();
    let file_path = capture_dir.join(format!("capture_{capture_id}.png"));
    fs::write(&file_path, create_synthetic_png(200, 100)).unwrap();

    // Catat tangkapan layar di SQLite
    {
        let conn = db.connection.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO screen_captures (id, session_id, capture_mode, captured_at, created_at)
             VALUES (?1, ?2, 'region', ?3, ?4)",
            rusqlite::params![capture_id, session_id, now, now],
        )
        .unwrap();
    }

    let fixture_engine = Arc::new(FixtureOcrEngine::new());
    let ocr_service = OcrService::new(fixture_engine, capture_dir, Arc::new(db));

    let extracted = ocr_service
        .extract_from_capture(ExtractOcrInput {
            capture_id: capture_id.clone(),
            session_id: Some(session_id.clone()),
            persona_id: Some(persona_id.clone()),
            engine: None,
        })
        .unwrap();

    // Pengguna mengoreksi teks di UI (PR-008)
    let confirm_input = ConfirmOcrExtractionInput {
        extraction_id: extracted.extraction_id,
        session_id: session_id.clone(),
        persona_id: persona_id.clone(),
        capture_id: Some(capture_id.clone()),
        question_text_corrected: "Berapa usia Anda saat ini? (Hasil Koreksi Pengguna)".to_string(),
        instructions: Some("(Pilih satu jawaban)".to_string()),
        options: vec!["18 - 24 tahun".to_string(), "25 - 34 tahun".to_string()],
        question_type: Some(QuestionType::SingleChoice),
        answer_format: Some(AnswerFormat::SingleChoice),
        scale_min: None,
        scale_max: None,
        scale_labels: None,
        matrix_rows: None,
        matrix_columns: None,
        visible_units: Some("tahun".to_string()),
    };

    let recorded = ocr_service.confirm_extraction(confirm_input).unwrap();

    // SC-030 / SC-041 Integration checks
    assert_eq!(recorded.session_id, session_id);
    assert_eq!(recorded.persona_id, persona_id);
    assert_eq!(recorded.capture_id, Some(capture_id));
    assert_eq!(recorded.provenance, Provenance::OcrDetected);
    assert_eq!(
        recorded.question_text_original,
        "Berapa usia Anda saat ini? (Hasil Koreksi Pengguna)"
    );
    assert_eq!(recorded.options.len(), 2);
}

// ==========================================
// TEST 10: ANTI PATH-TRAVERSAL SECURITY
// ==========================================
#[test]
fn test_ocr_anti_path_traversal_defense() {
    let (db, _db_dir) = setup_test_db();
    let capture_dir = get_unique_temp_dir();

    let fixture_engine = Arc::new(FixtureOcrEngine::new());
    let ocr_service = OcrService::new(fixture_engine, capture_dir, Arc::new(db));

    let malicious_ids = vec![
        "../secret.png",
        "..\\Windows\\System32\\cmd.exe",
        "../../etc/passwd",
        "not-a-valid-uuid",
    ];

    for malicious_id in malicious_ids {
        let input = ExtractOcrInput {
            capture_id: malicious_id.to_string(),
            session_id: None,
            persona_id: None,
            engine: None,
        };

        let result = ocr_service.extract_from_capture(input);
        assert!(
            result.is_err(),
            "Path traversal '{}' harus ditolak",
            malicious_id
        );
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALIDATION_ERROR");
    }
}
