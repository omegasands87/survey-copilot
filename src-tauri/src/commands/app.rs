#[tauri::command]
pub fn app_status() -> &'static str {
    "ready"
}
