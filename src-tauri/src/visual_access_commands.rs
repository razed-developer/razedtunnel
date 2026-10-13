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
pub(crate) fn list_visual_access_applications(app: AppHandle) -> Result<Vec<DesktopControlApplication>, String> {
    std::thread::Builder::new()
        .name("visual-access-uia".to_string())
        .spawn(move || desktop_control::list(&app, ""))
        .map_err(|error| format!("Could not start UI Automation worker: {error}"))?
        .join()
        .map_err(|_| "UI Automation worker stopped unexpectedly.".to_string())?
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
