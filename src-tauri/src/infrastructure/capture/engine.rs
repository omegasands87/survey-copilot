use crate::domain::capture::{CaptureRect, WindowInfo};
use crate::domain::error::AppError;

#[derive(Debug, Clone)]
pub struct RawCapturedImage {
    pub width: u32,
    pub height: u32,
    pub rgba_pixels: Vec<u8>,
    pub target_label: Option<String>,
}

pub trait ScreenCaptureEngine: Send + Sync {
    /// Enumerates currently visible top-level application windows.
    fn list_windows(&self) -> Result<Vec<WindowInfo>, AppError>;

    /// Captures a specified rectangular region of the screen or full virtual screen if None.
    fn capture_region(&self, bounds: Option<CaptureRect>) -> Result<RawCapturedImage, AppError>;

    /// Captures the specified application window by window handle / ID.
    fn capture_window(&self, window_id: u64) -> Result<RawCapturedImage, AppError>;
}
