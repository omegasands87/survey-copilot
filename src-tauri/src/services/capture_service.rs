use crate::db::Database;
use crate::domain::capture::{
    CaptureMode, CapturePreview, CaptureRegionInput, CaptureWindowInput, CleanupResult,
    DiscardResult, WindowInfo,
};
use crate::domain::error::AppError;
use crate::infrastructure::capture::{
    encode_rgba_to_png, png_to_data_url, RawCapturedImage, ScreenCaptureEngine,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

pub struct CaptureService {
    engine: Arc<dyn ScreenCaptureEngine>,
    temp_dir: PathBuf,
    database: Arc<Database>,
}

impl CaptureService {
    pub fn new(
        engine: Arc<dyn ScreenCaptureEngine>,
        temp_dir: PathBuf,
        database: Arc<Database>,
    ) -> Result<Self, AppError> {
        if !temp_dir.exists() {
            fs::create_dir_all(&temp_dir).map_err(|e| {
                AppError::internal(format!(
                    "Gagal membuat direktori penyimpanan sementara gambar tangkapan layar ({}): {e}",
                    temp_dir.display()
                ))
            })?;
        }

        Ok(Self {
            engine,
            temp_dir,
            database,
        })
    }

    pub fn temp_dir(&self) -> &Path {
        &self.temp_dir
    }

    pub fn list_windows(&self) -> Result<Vec<WindowInfo>, AppError> {
        Self::list_windows_with_engine(self.engine.as_ref())
    }

    pub fn capture_region(&self, input: CaptureRegionInput) -> Result<CapturePreview, AppError> {
        Self::capture_region_with_engine(
            self.engine.as_ref(),
            &self.temp_dir,
            &self.database,
            input,
        )
    }

    pub fn capture_window(&self, input: CaptureWindowInput) -> Result<CapturePreview, AppError> {
        Self::capture_window_with_engine(
            self.engine.as_ref(),
            &self.temp_dir,
            &self.database,
            input,
        )
    }

    pub fn discard_capture(&self, capture_id: &str) -> Result<DiscardResult, AppError> {
        Self::discard_capture_file(&self.temp_dir, &self.database, capture_id)
    }

    pub fn cleanup_expired_captures(
        &self,
        max_age_seconds: u64,
    ) -> Result<CleanupResult, AppError> {
        Self::cleanup_expired_files(&self.temp_dir, &self.database, max_age_seconds)
    }

    // ==========================================
    // STATIC PIPELINE METHODS FOR TAURI COMMANDS & TESTS
    // ==========================================

    pub fn list_windows_with_engine(
        engine: &dyn ScreenCaptureEngine,
    ) -> Result<Vec<WindowInfo>, AppError> {
        engine.list_windows()
    }

    pub fn capture_region_with_engine(
        engine: &dyn ScreenCaptureEngine,
        temp_dir: &Path,
        database: &Database,
        input: CaptureRegionInput,
    ) -> Result<CapturePreview, AppError> {
        let raw = engine.capture_region(input.rect)?;
        Self::process_raw_capture(
            raw,
            temp_dir,
            database,
            input.session_id,
            CaptureMode::Region,
        )
    }

    pub fn capture_window_with_engine(
        engine: &dyn ScreenCaptureEngine,
        temp_dir: &Path,
        database: &Database,
        input: CaptureWindowInput,
    ) -> Result<CapturePreview, AppError> {
        let raw = engine.capture_window(input.window_id)?;
        Self::process_raw_capture(
            raw,
            temp_dir,
            database,
            input.session_id,
            CaptureMode::Window,
        )
    }

    pub fn discard_capture_file(
        temp_dir: &Path,
        database: &Database,
        capture_id: &str,
    ) -> Result<DiscardResult, AppError> {
        let file_path = Self::resolve_capture_file_path(temp_dir, capture_id)?;
        let mut deleted = false;

        if file_path.exists() {
            fs::remove_file(&file_path).map_err(|e| {
                AppError::internal(format!(
                    "Gagal menghapus berkas gambar tangkapan layar sementara ({}): {e}",
                    file_path.display()
                ))
            })?;
            deleted = true;
        }

        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        if let Ok(conn) = database.connection.lock() {
            let _ = conn.execute(
                "UPDATE screen_captures SET image_deleted_at = ?, temporary_image_path = NULL WHERE id = ?",
                rusqlite::params![now, capture_id.trim()],
            );
        }

        Ok(DiscardResult {
            capture_id: capture_id.trim().to_string(),
            deleted,
            file_path: Some(file_path.to_string_lossy().to_string()),
            message: if deleted {
                "Berkas tangkapan layar sementara berhasil dibatalkan dan dihapus permanen dari disk."
                    .to_string()
            } else {
                "Berkas tangkapan layar tidak ditemukan atau telah dibersihkan sebelumnya."
                    .to_string()
            },
        })
    }

    pub fn cleanup_expired_files(
        temp_dir: &Path,
        database: &Database,
        max_age_seconds: u64,
    ) -> Result<CleanupResult, AppError> {
        if !temp_dir.exists() {
            return Ok(CleanupResult {
                deleted_count: 0,
                reclaimed_bytes: 0,
                message: "Direktori temporary tidak ditemukan.".to_string(),
            });
        }

        let entries = fs::read_dir(temp_dir).map_err(|e| {
            AppError::internal(format!(
                "Gagal membaca isi direktori temporary {}: {e}",
                temp_dir.display()
            ))
        })?;

        let mut deleted_count = 0u64;
        let mut reclaimed_bytes = 0u64;
        let now = SystemTime::now();

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(ext) = path.extension() {
                if ext != "png" {
                    continue;
                }
            } else {
                continue;
            }

            if let Ok(metadata) = entry.metadata() {
                let file_size = metadata.len();
                let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                if let Ok(elapsed) = now.duration_since(modified) {
                    if elapsed.as_secs() >= max_age_seconds && fs::remove_file(&path).is_ok() {
                        deleted_count += 1;
                        reclaimed_bytes += file_size;

                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if let Some(capture_id) = stem.strip_prefix("capture_") {
                                let now_str = chrono::Utc::now()
                                    .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
                                if let Ok(conn) = database.connection.lock() {
                                    let _ = conn.execute(
                                        "UPDATE screen_captures SET image_deleted_at = ?, temporary_image_path = NULL WHERE id = ?",
                                        rusqlite::params![now_str, capture_id],
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(CleanupResult {
            deleted_count,
            reclaimed_bytes,
            message: format!(
                "Pembersihan retensi selesai: {deleted_count} berkas dihapus, {reclaimed_bytes} bytes ruang disk dipulihkan."
            ),
        })
    }

    fn process_raw_capture(
        raw: RawCapturedImage,
        temp_dir: &Path,
        database: &Database,
        session_id: Option<String>,
        mode: CaptureMode,
    ) -> Result<CapturePreview, AppError> {
        if !temp_dir.exists() {
            fs::create_dir_all(temp_dir).map_err(|e| {
                AppError::internal(format!(
                    "Gagal membuat direktori temporary tangkapan layar {}: {e}",
                    temp_dir.display()
                ))
            })?;
        }

        let capture_id = uuid::Uuid::new_v4().to_string();
        let file_name = format!("capture_{capture_id}.png");
        let file_path = temp_dir.join(&file_name);

        let png_bytes = encode_rgba_to_png(raw.width, raw.height, &raw.rgba_pixels)?;
        let file_size_bytes = png_bytes.len() as u64;

        fs::write(&file_path, &png_bytes).map_err(|e| {
            AppError::internal(format!(
                "Gagal menyimpan berkas gambar sementara ke {}: {e}",
                file_path.display()
            ))
        })?;

        let captured_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let preview_data_url = png_to_data_url(&png_bytes);

        if let Some(ref sess_id) = session_id {
            if let Ok(conn) = database.connection.lock() {
                let exists: i64 = conn
                    .query_row(
                        "SELECT COUNT(1) FROM survey_sessions WHERE id = ?",
                        rusqlite::params![sess_id],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);

                if exists > 0 {
                    let _ = conn.execute(
                        "INSERT INTO screen_captures (
                            id, session_id, capture_mode, captured_at, application_label, temporary_image_path, image_deleted_at, created_at
                        ) VALUES (?, ?, ?, ?, ?, ?, NULL, ?)",
                        rusqlite::params![
                            capture_id,
                            sess_id,
                            mode.as_str(),
                            captured_at,
                            raw.target_label,
                            file_path.to_string_lossy(),
                            captured_at,
                        ],
                    );
                }
            }
        }

        Ok(CapturePreview {
            capture_id,
            session_id,
            source_mode: mode,
            captured_at,
            file_path: file_path.to_string_lossy().to_string(),
            file_name,
            file_size_bytes,
            width: raw.width,
            height: raw.height,
            target_label: raw.target_label,
            preview_data_url: Some(preview_data_url),
        })
    }

    pub fn resolve_capture_file_path(
        temp_dir: &Path,
        capture_id: &str,
    ) -> Result<PathBuf, AppError> {
        let clean_id = capture_id.trim();
        if uuid::Uuid::parse_str(clean_id).is_err() {
            return Err(AppError::validation(
                "captureId",
                "Identifier capture_id tidak valid (harus berupa UUID format standar).",
            ));
        }

        let file_name = format!("capture_{clean_id}.png");
        let path = temp_dir.join(file_name);

        if path.parent() != Some(temp_dir) {
            return Err(AppError::security(
                "Terdeteksi percobaan path traversal di luar direktori aman aplikasi.",
            ));
        }

        Ok(path)
    }
}
