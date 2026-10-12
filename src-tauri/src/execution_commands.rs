use tauri::AppHandle;
use crate::{activity, execution, models::{ExecutionStatus, CommandPreset, CommandOutcome, CommandRecord}, projects::approved_workspace};

#[tauri::command]
pub fn get_execution_status() -> Result<ExecutionStatus, String> {
    Ok(execution::execution_status())
}

#[tauri::command]
pub fn list_command_presets(
    app: AppHandle,
    workspace_id: String,
) -> Result<Vec<CommandPreset>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    execution::list_presets(&workspace)
}

#[tauri::command]
pub async fn run_workspace_command(
    app: AppHandle,
    workspace_id: String,
    preset_id: String,
) -> Result<CommandOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let workspace = approved_workspace(&app, &workspace_id)?;
        execution::request_command(&app, &workspace, &preset_id)
    })
    .await
    .map_err(|error| format!("The command task could not complete: {error}"))?
}

#[tauri::command]
pub fn list_command_history(
    app: AppHandle,
    workspace_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<CommandRecord>, String> {
    execution::list_history(&app, workspace_id.as_deref(), limit.unwrap_or(40))
}

#[tauri::command]
pub async fn approve_command(app: AppHandle, command_id: String) -> Result<CommandRecord, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let record = execution::list_history(&app, None, 100)?
            .into_iter()
            .find(|record| record.id == command_id)
            .ok_or_else(|| "That command request no longer exists.".to_string())?;
        let workspace = approved_workspace(&app, &record.workspace_id)?;
        let approved = execution::approve_command(&app, &workspace, &command_id)?;
        activity::sync_sandbox_command(&app, &approved);
        Ok(approved)
    })
    .await
    .map_err(|error| format!("The command approval task could not complete: {error}"))?
}

#[tauri::command]
pub fn reject_command(app: AppHandle, command_id: String) -> Result<CommandRecord, String> {
    let record = execution::reject_command(&app, &command_id)?;
    activity::sync_sandbox_command(&app, &record);
    Ok(record)
}

