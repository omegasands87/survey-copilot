use std::path::Path;

use crate::domain::error::AppError;
use crate::domain::ocr::OcrRawResult;

/// Trait untuk abstraksi engine OCR (Windows Media OCR, Fixture/Test OCR, Tesseract)
pub trait OcrEngine: Send + Sync {
    /// Menjalankan ekstraksi OCR pada berkas gambar di path yang ditentukan
    fn recognize_file(&self, image_path: &Path) -> Result<OcrRawResult, AppError>;

    /// Memeriksa apakah engine OCR ini tersedia pada lingkungan sistem saat ini
    fn is_available(&self) -> bool;

    /// Mengembalikan nama identifikasi engine
    fn engine_name(&self) -> &'static str;
}
