use tauri::AppHandle;
use crate::{models::{TempWorkspaceInfo, TempWorkspaceCleanupResult, TempWorkspaceFileResult}, projects::approved_workspace, temp_workspace};

#[tauri::command]
pub fn create_temp_workspace(
    app: AppHandle,
    workspace_id: String,
    task_id: String,
    label: String,
) -> Result<TempWorkspaceInfo, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    temp_workspace::create(&workspace, &task_id, &label)
}

#[tauri::command]
pub fn list_temp_workspaces(
    app: AppHandle,
    workspace_id: String,
) -> Result<Vec<TempWorkspaceInfo>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    temp_workspace::list(&workspace)
}

#[tauri::command]
pub fn inspect_temp_workspace(
    app: AppHandle,
    workspace_id: String,
    task_id: String,
) -> Result<TempWorkspaceInfo, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    temp_workspace::inspect(&workspace, &task_id)
}

#[tauri::command]
pub fn set_temp_workspace_preserved(
    app: AppHandle,
    workspace_id: String,
    task_id: String,
    preserved: bool,
) -> Result<TempWorkspaceInfo, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    temp_workspace::set_preserved(&workspace, &task_id, preserved)
}

#[tauri::command]
pub fn cleanup_temp_workspace(
    app: AppHandle,
    workspace_id: String,
    task_id: String,
    force_preserved: bool,
) -> Result<TempWorkspaceCleanupResult, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    temp_workspace::cleanup(&workspace, &task_id, force_preserved)
}

#[tauri::command]
pub fn temp_workspace_file_action(
    app: AppHandle,
    workspace_id: String,
    task_id: String,
    action: String,
    source_relative: String,
    destination_relative: Option<String>,
    overwrite: Option<bool>,
) -> Result<TempWorkspaceFileResult, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    match action.as_str() {
        "copyToWorkspace" => temp_workspace::copy_to_workspace(
            &workspace,
            &task_id,
            &source_relative,
            destination_relative.as_deref().ok_or_else(|| {
                "destinationRelative is required for copyToWorkspace.".to_string()
            })?,
            overwrite.unwrap_or(false),
        ),
        "moveToWorkspace" => temp_workspace::move_to_workspace(
            &workspace,
            &task_id,
            &source_relative,
            destination_relative.as_deref().ok_or_else(|| {
                "destinationRelative is required for moveToWorkspace.".to_string()
            })?,
            overwrite.unwrap_or(false),
        ),
        "rename" => temp_workspace::rename_file(
            &workspace,
            &task_id,
            &source_relative,
            destination_relative
                .as_deref()
                .ok_or_else(|| "destinationRelative is required for rename.".to_string())?,
            overwrite.unwrap_or(false),
        ),
        "delete" => temp_workspace::delete_file(&workspace, &task_id, &source_relative),
        _ => Err(
            "Temporary file action must be copyToWorkspace, moveToWorkspace, rename, or delete."
                .to_string(),
        ),
    }
}

