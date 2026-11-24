use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureMode {
    Region,
    Window,
    ClipboardImage,
    Manual,
}

impl CaptureMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            CaptureMode::Region => "region",
            CaptureMode::Window => "window",
            CaptureMode::ClipboardImage => "clipboard_image",
            CaptureMode::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureRegionInput {
    pub session_id: Option<String>,
    pub rect: Option<CaptureRect>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureWindowInput {
    pub session_id: Option<String>,
    pub window_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    pub id: u64,
    pub title: String,
    pub app_name: Option<String>,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_minimized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturePreview {
    pub capture_id: String,
    pub session_id: Option<String>,
    pub source_mode: CaptureMode,
    pub captured_at: String,
    pub file_path: String,
    pub file_name: String,
    pub file_size_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub target_label: Option<String>,
    pub preview_data_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscardResult {
    pub capture_id: String,
    pub deleted: bool,
    pub file_path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub deleted_count: u64,
    pub reclaimed_bytes: u64,
    pub message: String,
}
