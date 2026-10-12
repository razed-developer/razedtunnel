use tauri::AppHandle;
use crate::{activity, launcher, models::{LaunchActionOutcome, LaunchActionRecord}, projects::approved_workspace};

#[tauri::command]
pub fn open_url(
    app: AppHandle,
    workspace_id: String,
    url: String,
    application_id: Option<String>,
) -> Result<LaunchActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    launcher::request_open_url(&app, &workspace, url, application_id)
}

#[tauri::command]
pub fn open_workspace_path(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    application_id: Option<String>,
) -> Result<LaunchActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    launcher::request_open_workspace_path(&app, &workspace, relative_path, application_id)
}

#[tauri::command]
pub fn launch_application(
    app: AppHandle,
    workspace_id: String,
    application_id: String,
) -> Result<LaunchActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    launcher::request_launch_application(&app, &workspace, application_id)
}

#[tauri::command]
pub fn list_launch_history(
    app: AppHandle,
    workspace_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<LaunchActionRecord>, String> {
    launcher::list_history(&app, workspace_id.as_deref(), limit.unwrap_or(60))
}

#[tauri::command]
pub fn approve_launch_action(
    app: AppHandle,
    launch_id: String,
) -> Result<LaunchActionRecord, String> {
    let record = launcher::get_action(&app, &launch_id)?;
    let workspace = approved_workspace(&app, &record.workspace_id)?;
    let approved = launcher::approve_action(&app, &workspace, &launch_id)?;
    activity::sync_launch(&app, &approved);
    Ok(approved)
}

#[tauri::command]
pub fn reject_launch_action(
    app: AppHandle,
    launch_id: String,
) -> Result<LaunchActionRecord, String> {
    let record = launcher::reject_action(&app, &launch_id)?;
    activity::sync_launch(&app, &record);
    Ok(record)
}

