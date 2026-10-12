use tauri::AppHandle;
use crate::{browser, desktop_control, gmail_access, integrations, projects::approved_workspace};

#[tauri::command]
pub fn list_deep_integrations(
    app: AppHandle,
    workspace_id: String,
) -> Result<Vec<integrations::DeepIntegration>, String> {
    let _workspace = approved_workspace(&app, &workspace_id)?;
    integrations::list(&app, &workspace_id)
}

#[tauri::command]
pub fn set_deep_integration_enabled(
    app: AppHandle,
    workspace_id: String,
    integration_id: String,
    enabled: bool,
) -> Result<Vec<integrations::DeepIntegration>, String> {
    let _workspace = approved_workspace(&app, &workspace_id)?;
    integrations::set_enabled(&app, &workspace_id, &integration_id, enabled)
}

#[tauri::command]
pub fn list_desktop_control_applications(
    app: AppHandle,
    workspace_id: String,
) -> Result<Vec<desktop_control::DesktopControlApplication>, String> {
    let _workspace = approved_workspace(&app, &workspace_id)?;
    desktop_control::list(&app, &workspace_id)
}

#[tauri::command]
pub fn get_desktop_control_enabled(app: AppHandle, workspace_id: String) -> Result<bool, String> {
    let _workspace = approved_workspace(&app, &workspace_id)?;
    desktop_control::is_enabled(&app, &workspace_id)
}

#[tauri::command]
pub fn set_desktop_control_enabled(
    app: AppHandle,
    workspace_id: String,
    enabled: bool,
) -> Result<bool, String> {
    let _workspace = approved_workspace(&app, &workspace_id)?;
    desktop_control::set_global_enabled(&app, &workspace_id, enabled)
}

#[tauri::command]
pub fn get_gmail_access_enabled(app: AppHandle, workspace_id: String) -> Result<bool, String> {
    let _workspace = approved_workspace(&app, &workspace_id)?;
    gmail_access::is_enabled(&app)
}

#[tauri::command]
pub fn set_gmail_access_enabled(
    app: AppHandle,
    workspace_id: String,
    enabled: bool,
) -> Result<bool, String> {
    let _workspace = approved_workspace(&app, &workspace_id)?;
    let enabled = gmail_access::set_enabled(&app, &workspace_id, enabled)?;
    browser::set_shared_login_enabled(&workspace_id, enabled)?;
    Ok(enabled)
}

