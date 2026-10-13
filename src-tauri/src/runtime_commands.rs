use tauri::AppHandle;
use crate::{hardening, models::RuntimeDiagnostics};

#[tauri::command]
pub fn get_runtime_diagnostics(app: AppHandle) -> Result<RuntimeDiagnostics, String> {
    hardening::diagnostics(&app)
}

#[tauri::command]
pub fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<RuntimeDiagnostics, String> {
    hardening::set_launch_at_login(&app, enabled)?;
    hardening::diagnostics(&app)
}
