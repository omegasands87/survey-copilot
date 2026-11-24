use std::path::Path;
use std::sync::Arc;

use super::engine::OcrEngine;
use crate::domain::error::AppError;
use crate::domain::ocr::OcrRawResult;

/// Engine Komposit yang mencoba engine primer (Windows Media OCR) terlebih dahulu
/// dan dapat melakukan fallback ke engine sekunder jika dikonfigurasikan
pub struct CompositeOcrEngine {
    primary: Arc<dyn OcrEngine>,
    fallback: Option<Arc<dyn OcrEngine>>,
}

impl CompositeOcrEngine {
    pub fn new(primary: Arc<dyn OcrEngine>, fallback: Option<Arc<dyn OcrEngine>>) -> Self {
        Self { primary, fallback }
    }
}

impl OcrEngine for CompositeOcrEngine {
    fn engine_name(&self) -> &'static str {
        "composite_ocr"
    }

    fn is_available(&self) -> bool {
        self.primary.is_available()
            || self
                .fallback
                .as_ref()
                .map(|f| f.is_available())
                .unwrap_or(false)
    }

    fn recognize_file(&self, image_path: &Path) -> Result<OcrRawResult, AppError> {
        if self.primary.is_available() {
            match self.primary.recognize_file(image_path) {
                Ok(res) => return Ok(res),
                Err(err) => {
                    if let Some(ref fb) = self.fallback {
                        if fb.is_available() {
                            return fb.recognize_file(image_path);
                        }
                    }
                    return Err(err);
                }
            }
        }

        if let Some(ref fb) = self.fallback {
            if fb.is_available() {
                return fb.recognize_file(image_path);
            }
        }

        Err(AppError::service_unavailable(
            "Tidak ada engine OCR yang tersedia di sistem saat ini.",
        ))
    }
}
