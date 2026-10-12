use tauri::AppHandle;
use crate::{monitoring, models::{MonitoringStatus, MonitoringSnapshot, MonitoringFileEvent}, projects::approved_workspace};

#[tauri::command]
pub fn get_monitoring_status(
    app: AppHandle,
    workspace_id: String,
) -> Result<MonitoringStatus, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    Ok(monitoring::status(&app, &workspace))
}

#[tauri::command]
pub async fn start_workspace_monitoring(
    app: AppHandle,
    workspace_id: String,
) -> Result<MonitoringStatus, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    tauri::async_runtime::spawn_blocking(move || monitoring::start_monitoring(&app, &workspace))
        .await
        .map_err(|error| format!("Monitoring start worker could not complete: {error}"))?
}

#[tauri::command]
pub async fn stop_workspace_monitoring(
    app: AppHandle,
    workspace_id: String,
) -> Result<MonitoringStatus, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    tauri::async_runtime::spawn_blocking(move || monitoring::stop_monitoring(&app, &workspace))
        .await
        .map_err(|error| format!("Monitoring stop worker could not complete: {error}"))?
}

#[tauri::command]
pub async fn get_monitoring_snapshot(
    app: AppHandle,
    workspace_id: String,
) -> Result<MonitoringSnapshot, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    tauri::async_runtime::spawn_blocking(move || monitoring::snapshot(&app, &workspace))
        .await
        .map_err(|error| format!("Monitoring snapshot worker could not complete: {error}"))?
}

#[tauri::command]
pub fn list_monitoring_file_events(
    app: AppHandle,
    workspace_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<MonitoringFileEvent>, String> {
    monitoring::list_file_events(&app, workspace_id.as_deref(), limit.unwrap_or(60))
}

