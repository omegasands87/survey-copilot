use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::db::Database;
use crate::domain::capture::CaptureMode;
use crate::domain::error::AppError;
use crate::domain::ocr::{ConfirmOcrExtractionInput, ExtractOcrInput, ExtractedSurveyQuestion};
use crate::domain::survey::{ConfirmationStatus, CreateQuestionInput, Provenance, SurveyQuestion};
use crate::infrastructure::ocr::{CompositeOcrEngine, OcrEngine, WindowsMediaOcrEngine};
use crate::repository::OcrRepository;
use crate::services::capture_service::CaptureService;
use crate::services::layout_reconstructor::LayoutReconstructorService;
use crate::services::survey_service::SurveyService;

/// Layanan Orkestrasi OCR dan Rekonstruksi Tata Letak Pertanyaan (SC-041)
pub struct OcrService {
    engine: Arc<dyn OcrEngine>,
    temp_dir: PathBuf,
    database: Arc<Database>,
}

impl OcrService {
    pub fn new(engine: Arc<dyn OcrEngine>, temp_dir: PathBuf, database: Arc<Database>) -> Self {
        Self {
            engine,
            temp_dir,
            database,
        }
    }

    /// Membuat instance default OcrService tanpa synthetic fallback.
    /// FixtureOcrEngine hanya boleh dipasang secara eksplisit oleh test harness.
    pub fn with_default_engine(temp_dir: PathBuf, database: Arc<Database>) -> Self {
        let primary = Arc::new(WindowsMediaOcrEngine::new());
        let composite = Arc::new(CompositeOcrEngine::new(primary, None));
        Self::new(composite, temp_dir, database)
    }

    pub fn engine(&self) -> &dyn OcrEngine {
        self.engine.as_ref()
    }

    pub fn extract_from_capture(
        &self,
        input: ExtractOcrInput,
    ) -> Result<ExtractedSurveyQuestion, AppError> {
        Self::extract_from_capture_with_engine(
            self.engine.as_ref(),
            &self.temp_dir,
            &self.database,
            input,
        )
    }

    pub fn get_extraction(&self, extraction_id: &str) -> Result<ExtractedSurveyQuestion, AppError> {
        Self::get_extraction_with_db(&self.database, extraction_id)
    }

    pub fn get_latest_extraction_for_capture(
        &self,
        capture_id: &str,
    ) -> Result<Option<ExtractedSurveyQuestion>, AppError> {
        Self::get_latest_extraction_for_capture_with_db(&self.database, capture_id)
    }

    pub fn confirm_extraction(
        &self,
        input: ConfirmOcrExtractionInput,
    ) -> Result<SurveyQuestion, AppError> {
        Self::confirm_extraction_with_db(&self.database, input)
    }

    // ==========================================
    // STATIC PIPELINE METHODS FOR TAURI COMMANDS & TESTS
    // ==========================================

    pub fn extract_from_capture_with_engine(
        engine: &dyn OcrEngine,
        temp_dir: &Path,
        database: &Database,
        input: ExtractOcrInput,
    ) -> Result<ExtractedSurveyQuestion, AppError> {
        let capture_id = input.capture_id.trim();
        if capture_id.is_empty() {
            return Err(AppError::validation(
                "captureId",
                "ID tangkapan layar wajib disertakan untuk ekstraksi OCR.",
            ));
        }

        // Resolusi path berkas gambar tangkapan layar dengan pertahanan anti path-traversal
        let image_path = CaptureService::resolve_capture_file_path(temp_dir, capture_id)?;
        if !image_path.exists() {
            return Err(AppError::not_found(format!(
                "Berkas gambar tangkapan layar ({}) tidak ditemukan di disk atau telah dibersihkan.",
                image_path.display()
            )));
        }

        // Ambil metadata tangkapan layar dari database jika tersedia
        let (source_mode, captured_at, application_label) =
            Self::get_capture_metadata_from_db(database, capture_id);

        // Jalankan pemindaian OCR melalui engine yang terkonfigurasi
        let raw_result = engine.recognize_file(&image_path)?;

        // Rekonstruksi struktur layout, opsi, skala, matriks, unit, dan deteksi ketidakpastian
        let mut extracted = LayoutReconstructorService::reconstruct(
            &raw_result,
            Some(capture_id.to_string()),
            input.session_id.clone(),
            input.persona_id.clone(),
            source_mode,
            captured_at,
            application_label,
        )?;

        // Persistensi hasil ekstraksi ke SQLite untuk mempertahankan provenance capture-to-extraction (SC-041)
        if let Ok(conn) = database.connection.lock() {
            let _ = OcrRepository::save_extraction(&conn, &extracted);
        }

        // Set status konfirmasi awal sebagai draf
        extracted.confirmation_status = ConfirmationStatus::Draft;

        Ok(extracted)
    }

    pub fn get_extraction_with_db(
        database: &Database,
        extraction_id: &str,
    ) -> Result<ExtractedSurveyQuestion, AppError> {
        let conn = database.connection.lock().map_err(|_| {
            AppError::internal(
                "Gagal mengunci koneksi basis data untuk membaca hasil ekstraksi OCR.",
            )
        })?;

        OcrRepository::get_extraction_by_id(&conn, extraction_id.trim())?.ok_or_else(|| {
            AppError::not_found(format!(
                "Hasil ekstraksi OCR '{extraction_id}' tidak ditemukan."
            ))
        })
    }

    pub fn get_latest_extraction_for_capture_with_db(
        database: &Database,
        capture_id: &str,
    ) -> Result<Option<ExtractedSurveyQuestion>, AppError> {
        let conn = database.connection.lock().map_err(|_| {
            AppError::internal(
                "Gagal mengunci koneksi basis data untuk membaca hasil ekstraksi OCR.",
            )
        })?;

        Ok(OcrRepository::get_latest_extraction_by_capture(
            &conn,
            capture_id.trim(),
        )?)
    }

    pub fn confirm_extraction_with_db(
        database: &Database,
        input: ConfirmOcrExtractionInput,
    ) -> Result<SurveyQuestion, AppError> {
        let persona_id = input.persona_id.trim();
        let session_id = input.session_id.trim();
        let extraction_id = input.extraction_id.trim();

        if persona_id.is_empty() {
            return Err(AppError::validation("personaId", "Persona ID wajib diisi."));
        }
        if session_id.is_empty() {
            return Err(AppError::validation("sessionId", "Session ID wajib diisi."));
        }
        if input.question_text_corrected.trim().is_empty() {
            return Err(AppError::validation(
                "questionTextCorrected",
                "Teks pertanyaan hasil konfirmasi tidak boleh kosong.",
            ));
        }

        let options_json =
            serde_json::to_string(&input.options).unwrap_or_else(|_| "[]".to_string());

        let conn = database.connection.lock().map_err(|_| {
            AppError::internal(
                "Gagal mengunci koneksi basis data untuk mengonfirmasi hasil ekstraksi OCR.",
            )
        })?;

        // 1. Perbarui status ekstraksi OCR menjadi confirmed
        let _ = OcrRepository::update_confirmed_extraction(
            &conn,
            extraction_id,
            &input.question_text_corrected,
            input.instructions.as_deref(),
            &options_json,
            "confirmed",
        );

        // 2. Buat atau rekam pertanyaan survei ke sesi survei aktif melalui SurveyService
        let create_input = CreateQuestionInput {
            persona_id: persona_id.to_string(),
            session_id: session_id.to_string(),
            capture_id: input.capture_id,
            question_text_original: input.question_text_corrected.clone(),
            instructions: input.instructions,
            options: Some(input.options),
            question_type: input.question_type,
            answer_format: input.answer_format,
            provenance: Some(Provenance::OcrDetected),
            question_order: None,
            preceding_question_id: None,
            branch_logic: None,
        };

        let recorded_question = SurveyService::create_question(&conn, create_input)?;

        Ok(recorded_question)
    }

    // ==========================================
    // METADATA HELPER
    // ==========================================

    fn get_capture_metadata_from_db(
        database: &Database,
        capture_id: &str,
    ) -> (CaptureMode, Option<String>, Option<String>) {
        if let Ok(conn) = database.connection.lock() {
            let res: Result<(String, String, Option<String>), _> = conn.query_row(
                "SELECT capture_mode, captured_at, application_label FROM screen_captures WHERE id = ?1",
                rusqlite::params![capture_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            );

            if let Ok((mode_str, captured_at, app_label)) = res {
                let mode = match mode_str.as_str() {
                    "window" => CaptureMode::Window,
                    "clipboard_image" => CaptureMode::ClipboardImage,
                    "manual" => CaptureMode::Manual,
                    _ => CaptureMode::Region,
                };
                return (mode, Some(captured_at), app_label);
            }
        }

        (CaptureMode::Region, None, None)
    }
}
