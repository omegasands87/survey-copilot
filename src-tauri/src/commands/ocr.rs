use tauri::{AppHandle, Manager, State};

use crate::db::Database;
use crate::domain::error::AppError;
use crate::domain::ocr::{ConfirmOcrExtractionInput, ExtractOcrInput, ExtractedSurveyQuestion};
use crate::domain::survey::SurveyQuestion;
use crate::services::OcrService;
use crate::OcrEngineState;

#[tauri::command]
pub fn ocr_extract_capture(
    input: ExtractOcrInput,
    ocr_engine: State<'_, OcrEngineState>,
    database: State<'_, Database>,
    app_handle: AppHandle,
) -> Result<ExtractedSurveyQuestion, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::internal(format!("Gagal mengakses direktori data aplikasi: {e}")))?;
    let temp_dir = app_data_dir.join("temp_captures");

    OcrService::extract_from_capture_with_engine(ocr_engine.0.as_ref(), &temp_dir, &database, input)
}

#[tauri::command]
pub fn ocr_get_extraction(
    extraction_id: String,
    _ocr_engine: State<'_, OcrEngineState>,
    database: State<'_, Database>,
    _app_handle: AppHandle,
) -> Result<ExtractedSurveyQuestion, AppError> {
    OcrService::get_extraction_with_db(&database, &extraction_id)
}

#[tauri::command]
pub fn ocr_get_latest_for_capture(
    capture_id: String,
    _ocr_engine: State<'_, OcrEngineState>,
    database: State<'_, Database>,
    _app_handle: AppHandle,
) -> Result<Option<ExtractedSurveyQuestion>, AppError> {
    OcrService::get_latest_extraction_for_capture_with_db(&database, &capture_id)
}

#[tauri::command]
pub fn ocr_confirm_extraction(
    input: ConfirmOcrExtractionInput,
    _ocr_engine: State<'_, OcrEngineState>,
    database: State<'_, Database>,
    _app_handle: AppHandle,
) -> Result<SurveyQuestion, AppError> {
    OcrService::confirm_extraction_with_db(&database, input)
}
