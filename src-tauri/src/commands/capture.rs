use crate::db::Database;
use crate::domain::capture::{
    CapturePreview, CaptureRegionInput, CaptureWindowInput, CleanupResult, DiscardResult,
    WindowInfo,
};
use crate::domain::error::AppError;
use crate::services::CaptureService;
use crate::CaptureEngineState;
use std::fs;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub fn capture_list_windows(
    capture_engine: State<'_, CaptureEngineState>,
) -> Result<Vec<WindowInfo>, AppError> {
    CaptureService::list_windows_with_engine(capture_engine.0.as_ref())
}

#[tauri::command]
pub fn capture_region(
    input: CaptureRegionInput,
    capture_engine: State<'_, CaptureEngineState>,
    database: State<'_, Database>,
    app_handle: AppHandle,
) -> Result<CapturePreview, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::internal(format!("Gagal mengakses direktori data aplikasi: {e}")))?;
    let temp_dir = app_data_dir.join("temp_captures");
    CaptureService::capture_region_with_engine(
        capture_engine.0.as_ref(),
        &temp_dir,
        &database,
        input,
    )
}

#[tauri::command]
pub fn capture_window(
    input: CaptureWindowInput,
    capture_engine: State<'_, CaptureEngineState>,
    database: State<'_, Database>,
    app_handle: AppHandle,
) -> Result<CapturePreview, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::internal(format!("Gagal mengakses direktori data aplikasi: {e}")))?;
    let temp_dir = app_data_dir.join("temp_captures");
    CaptureService::capture_window_with_engine(
        capture_engine.0.as_ref(),
        &temp_dir,
        &database,
        input,
    )
}

#[tauri::command]
pub fn capture_discard(
    capture_id: String,
    database: State<'_, Database>,
    app_handle: AppHandle,
) -> Result<DiscardResult, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::internal(format!("Gagal mengakses direktori data aplikasi: {e}")))?;
    let temp_dir = app_data_dir.join("temp_captures");
    CaptureService::discard_capture_file(&temp_dir, &database, &capture_id)
}

#[tauri::command]
pub fn capture_cleanup(
    max_age_seconds: Option<u64>,
    database: State<'_, Database>,
    app_handle: AppHandle,
) -> Result<CleanupResult, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::internal(format!("Gagal mengakses direktori data aplikasi: {e}")))?;
    let temp_dir = app_data_dir.join("temp_captures");
    CaptureService::cleanup_expired_files(&temp_dir, &database, max_age_seconds.unwrap_or(86400))
}

#[tauri::command]
pub fn capture_get_preview(
    capture_id: String,
    app_handle: AppHandle,
) -> Result<CapturePreview, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::internal(format!("Gagal mengakses direktori data aplikasi: {e}")))?;
    let temp_dir = app_data_dir.join("temp_captures");
    let file_path = CaptureService::resolve_capture_file_path(&temp_dir, &capture_id)?;

    if !file_path.exists() {
        return Err(AppError::not_found(format!(
            "Berkas tangkapan layar dengan ID {capture_id} tidak ditemukan di disk."
        )));
    }

    let png_bytes = fs::read(&file_path).map_err(|e| {
        AppError::internal(format!("Gagal membaca berkas gambar tangkapan layar: {e}"))
    })?;

    let file_size_bytes = png_bytes.len() as u64;
    let preview_data_url = crate::infrastructure::capture::png_to_data_url(&png_bytes);
    let captured_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    Ok(CapturePreview {
        capture_id,
        session_id: None,
        source_mode: crate::domain::capture::CaptureMode::Region,
        captured_at,
        file_path: file_path.to_string_lossy().to_string(),
        file_name: file_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default(),
        file_size_bytes,
        width: 0,
        height: 0,
        target_label: None,
        preview_data_url: Some(preview_data_url),
    })
}
