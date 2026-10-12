use tauri::{AppHandle, State};
use crate::{app_state::AppState, hardening, models::RuntimeDiagnostics, updates};

#[tauri::command]
pub fn get_runtime_diagnostics(app: AppHandle) -> Result<RuntimeDiagnostics, String> {
    hardening::diagnostics(&app)
}

#[tauri::command]
pub fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<RuntimeDiagnostics, String> {
    hardening::set_launch_at_login(&app, enabled)?;
    hardening::diagnostics(&app)
}

#[tauri::command]
pub fn get_update_status(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<updates::UpdateStatus, String> {
    updates::status(&app, Some(state.inner()))
}

#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    state: State<'_, AppState>,
    manual: bool,
) -> Result<updates::UpdateStatus, String> {
    updates::check(&app, manual, Some(state.inner())).await
}

#[tauri::command]
pub fn set_auto_update_checks(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<updates::UpdateStatus, String> {
    updates::set_auto_check(&app, enabled, Some(state.inner()))
}

#[tauri::command]
pub fn defer_update(
    app: AppHandle,
    state: State<'_, AppState>,
    version: String,
) -> Result<updates::UpdateStatus, String> {
    updates::defer(&app, &version, Some(state.inner()))
}

#[tauri::command]
pub async fn install_update_and_restart(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<updates::UpdateInstallResult, String> {
    updates::install_and_restart(&app, state.inner()).await
}

