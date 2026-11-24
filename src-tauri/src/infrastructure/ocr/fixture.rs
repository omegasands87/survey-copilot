use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::engine::OcrEngine;
use crate::domain::error::AppError;
use crate::domain::ocr::{BoundingBox, OcrLine, OcrRawResult, OcrWord};

/// Fixture OCR Engine untuk pengujian unit dan integrasi deterministik
#[derive(Clone)]
pub struct FixtureOcrEngine {
    fixtures: Arc<Mutex<HashMap<String, OcrRawResult>>>,
    default_fixture: Arc<Mutex<Option<OcrRawResult>>>,
    available: Arc<Mutex<bool>>,
    engine_name: &'static str,
}

impl FixtureOcrEngine {
    pub fn new() -> Self {
        Self {
            fixtures: Arc::new(Mutex::new(HashMap::new())),
            default_fixture: Arc::new(Mutex::new(None)),
            available: Arc::new(Mutex::new(true)),
            engine_name: "fixture_ocr",
        }
    }

    pub fn with_engine_name(mut self, name: &'static str) -> Self {
        self.engine_name = name;
        self
    }

    pub fn set_available(&self, is_available: bool) {
        let mut avail = self.available.lock().unwrap();
        *avail = is_available;
    }

    pub fn set_default_fixture(&self, result: OcrRawResult) {
        let mut def = self.default_fixture.lock().unwrap();
        *def = Some(result);
    }

    pub fn register_fixture(&self, key: impl Into<String>, result: OcrRawResult) {
        let mut fixtures = self.fixtures.lock().unwrap();
        fixtures.insert(key.into(), result);
    }

    /// Helper untuk membuat baris OCR dengan kata-kata dan koordinat otomatis
    pub fn build_line(
        text: &str,
        y: f64,
        height: f64,
        start_x: f64,
        word_confidence: f32,
    ) -> OcrLine {
        let words_str: Vec<&str> = text.split_whitespace().collect();
        let mut words = Vec::new();
        let mut current_x = start_x;

        for w in words_str {
            let w_len = (w.len() as f64) * 8.0 + 4.0;
            words.push(OcrWord {
                text: w.to_string(),
                bounding_box: BoundingBox::new(current_x, y, w_len, height),
                confidence: word_confidence,
            });
            current_x += w_len + 6.0;
        }

        let total_w = current_x - start_x;
        OcrLine {
            text: text.to_string(),
            bounding_box: BoundingBox::new(start_x, y, total_w, height),
            words,
            confidence: word_confidence,
        }
    }
}

impl Default for FixtureOcrEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl OcrEngine for FixtureOcrEngine {
    fn engine_name(&self) -> &'static str {
        self.engine_name
    }

    fn is_available(&self) -> bool {
        *self.available.lock().unwrap()
    }

    fn recognize_file(&self, image_path: &Path) -> Result<OcrRawResult, AppError> {
        if !self.is_available() {
            return Err(AppError::service_unavailable(
                "Layanan OCR sedang tidak tersedia (simulasi AT-053).",
            ));
        }

        let file_name = image_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let fixtures = self.fixtures.lock().unwrap();
        if let Some(res) = fixtures.get(&file_name) {
            return Ok(res.clone());
        }

        // Cek jika path mengandung key tertentu
        let path_str = image_path.to_string_lossy().to_string();
        for (k, v) in fixtures.iter() {
            if path_str.contains(k) {
                return Ok(v.clone());
            }
        }

        let default_fix = self.default_fixture.lock().unwrap();
        if let Some(res) = default_fix.as_ref() {
            return Ok(res.clone());
        }

        // Default synthetic fallback jika fixture tidak didaftarkan secara eksplisit
        let line1 = Self::build_line("1. Berapa usia Anda saat ini?", 20.0, 20.0, 20.0, 0.98);
        let line2 = Self::build_line("(Pilih satu jawaban)", 50.0, 16.0, 20.0, 0.95);
        let line3 = Self::build_line("A. 18 - 24 tahun", 80.0, 18.0, 30.0, 0.97);
        let line4 = Self::build_line("B. 25 - 34 tahun", 110.0, 18.0, 30.0, 0.96);
        let line5 = Self::build_line("C. 35 - 44 tahun", 140.0, 18.0, 30.0, 0.97);

        Ok(OcrRawResult {
            raw_text: "1. Berapa usia Anda saat ini?\n(Pilih satu jawaban)\nA. 18 - 24 tahun\nB. 25 - 34 tahun\nC. 35 - 44 tahun".to_string(),
            text_angle: 0.0,
            lines: vec![line1, line2, line3, line4, line5],
            engine: self.engine_name.to_string(),
            image_width: 800,
            image_height: 600,
        })
    }
}
