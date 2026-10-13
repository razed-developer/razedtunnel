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
    desktop_control::list(&app, "")
}
