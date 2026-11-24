use super::engine::{RawCapturedImage, ScreenCaptureEngine};
use crate::domain::capture::{CaptureRect, WindowInfo};
use crate::domain::error::AppError;
use std::sync::Mutex;

pub struct InMemoryScreenCaptureEngine {
    windows: Mutex<Vec<WindowInfo>>,
    default_capture_width: u32,
    default_capture_height: u32,
    fail_on_next_capture: Mutex<Option<String>>,
}

impl InMemoryScreenCaptureEngine {
    pub fn new() -> Self {
        Self {
            windows: Mutex::new(vec![
                WindowInfo {
                    id: 1001,
                    title: "Google Chrome - YouGov Survey 2026".to_string(),
                    app_name: Some("chrome.exe".to_string()),
                    x: 100,
                    y: 100,
                    width: 1280,
                    height: 800,
                    is_minimized: false,
                },
                WindowInfo {
                    id: 1002,
                    title: "Microsoft Edge - Toluna Quick Question".to_string(),
                    app_name: Some("msedge.exe".to_string()),
                    x: 200,
                    y: 200,
                    width: 1024,
                    height: 768,
                    is_minimized: false,
                },
                WindowInfo {
                    id: 1003,
                    title: "Notepad - Survey Notes".to_string(),
                    app_name: Some("notepad.exe".to_string()),
                    x: 300,
                    y: 300,
                    width: 600,
                    height: 400,
                    is_minimized: true,
                },
            ]),
            default_capture_width: 800,
            default_capture_height: 600,
            fail_on_next_capture: Mutex::new(None),
        }
    }

    pub fn with_windows(windows: Vec<WindowInfo>) -> Self {
        Self {
            windows: Mutex::new(windows),
            default_capture_width: 800,
            default_capture_height: 600,
            fail_on_next_capture: Mutex::new(None),
        }
    }

    pub fn set_fail_on_next_capture(&self, error_message: Option<String>) {
        if let Ok(mut fail) = self.fail_on_next_capture.lock() {
            *fail = error_message;
        }
    }

    fn generate_synthetic_rgba(width: u32, height: u32, pattern_byte: u8) -> Vec<u8> {
        let total = (width as usize) * (height as usize) * 4;
        let mut pixels = Vec::with_capacity(total);
        for i in 0..(width * height) {
            let r = pattern_byte;
            let g = ((i % 256) ^ (pattern_byte as u32)) as u8;
            let b = ((i / 256) % 256) as u8;
            let a = 255u8;
            pixels.push(r);
            pixels.push(g);
            pixels.push(b);
            pixels.push(a);
        }
        pixels
    }
}

impl Default for InMemoryScreenCaptureEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenCaptureEngine for InMemoryScreenCaptureEngine {
    fn list_windows(&self) -> Result<Vec<WindowInfo>, AppError> {
        let windows = self
            .windows
            .lock()
            .map_err(|_| AppError::internal("Lock mutex window list terkunci."))?;
        Ok(windows.clone())
    }

    fn capture_region(&self, bounds: Option<CaptureRect>) -> Result<RawCapturedImage, AppError> {
        if let Ok(mut fail_slot) = self.fail_on_next_capture.lock() {
            if let Some(msg) = fail_slot.take() {
                return Err(AppError::internal(msg));
            }
        }

        let (w, h, label) = if let Some(rect) = bounds {
            if rect.width == 0 || rect.height == 0 {
                return Err(AppError::validation(
                    "bounds",
                    "Ukuran area tangkapan layar tidak valid: lebar dan tinggi harus lebih besar dari 0.",
                ));
            }
            (
                rect.width,
                rect.height,
                format!(
                    "Area Layar ({}, {}, {}x{})",
                    rect.x, rect.y, rect.width, rect.height
                ),
            )
        } else {
            (
                self.default_capture_width,
                self.default_capture_height,
                "Seluruh Layar Utama".to_string(),
            )
        };

        let rgba = Self::generate_synthetic_rgba(w, h, 0x42);
        Ok(RawCapturedImage {
            width: w,
            height: h,
            rgba_pixels: rgba,
            target_label: Some(label),
        })
    }

    fn capture_window(&self, window_id: u64) -> Result<RawCapturedImage, AppError> {
        if let Ok(mut fail_slot) = self.fail_on_next_capture.lock() {
            if let Some(msg) = fail_slot.take() {
                return Err(AppError::internal(msg));
            }
        }

        let windows = self
            .windows
            .lock()
            .map_err(|_| AppError::internal("Lock mutex window list terkunci."))?;

        let window = windows.iter().find(|w| w.id == window_id).ok_or_else(|| {
            AppError::not_found(format!(
                "Jendela aplikasi dengan ID {window_id} tidak ditemukan atau telah ditutup."
            ))
        })?;

        let w = window.width.max(1);
        let h = window.height.max(1);
        let rgba = Self::generate_synthetic_rgba(w, h, 0x99);

        Ok(RawCapturedImage {
            width: w,
            height: h,
            rgba_pixels: rgba,
            target_label: Some(format!("Jendela: {}", window.title)),
        })
    }
}
