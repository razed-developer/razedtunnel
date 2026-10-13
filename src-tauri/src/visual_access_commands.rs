use tauri::AppHandle;
use crate::desktop_control::{self, DesktopControlApplication};

#[tauri::command]
pub(crate) fn get_visual_access_status(app: AppHandle) -> Result<bool, String> {
    desktop_control::is_enabled(&app, "")
}

#[tauri::command]
pub(crate) fn set_visual_access_enabled(app: AppHandle, enabled: bool) -> Result<bool, String> {
    desktop_control::set_global_enabled(&app, "", enabled)
}

#[tauri::command]
pub(crate) async fn list_visual_access_applications(app: AppHandle) -> Result<Vec<DesktopControlApplication>, String> {
    // UI Automation needs a fresh COM apartment, and enumeration must not block
    // the Tauri UI thread while Connections mounts.
    tauri::async_runtime::spawn_blocking(move || desktop_control::list(&app, ""))
        .await
        .map_err(|error| format!("UI Automation discovery worker failed: {error}"))?
}

#[tauri::command]
pub(crate) async fn capture_visual_access_window(
    app: AppHandle,
    application_id: String,
) -> Result<desktop_control::DesktopScreenshot, String> {
    // The existing screenshot service enforces local Desktop Access and validates
    // the selected application/window. UIA must initialize on a fresh COM thread.
    tauri::async_runtime::spawn_blocking(move || {
        desktop_control::screenshot(&app, "", &application_id, None)
    })
    .await
    .map_err(|error| format!("Window capture worker failed: {error}"))?
}
