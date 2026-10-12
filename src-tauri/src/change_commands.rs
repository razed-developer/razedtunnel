use tauri::AppHandle;
use crate::{activity, changes, models::{ChangeOutcome, ChangeRecord}, projects::approved_workspace};

#[tauri::command]
pub fn create_file(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    content: String,
) -> Result<ChangeOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    changes::create_file(&app, &workspace, relative_path, content, None)
}

#[tauri::command]
pub fn write_file(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    content: String,
) -> Result<ChangeOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    changes::write_file(&app, &workspace, relative_path, content, None)
}

#[tauri::command]
pub fn patch_file(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    expected: String,
    replacement: String,
) -> Result<ChangeOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    changes::patch_file(&app, &workspace, relative_path, expected, replacement, None)
}

#[tauri::command]
pub fn create_directory(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    recursive: bool,
) -> Result<ChangeOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    changes::create_directory(&app, &workspace, relative_path, recursive, None)
}

#[tauri::command]
pub fn rename_entry(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    new_name: String,
) -> Result<ChangeOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    changes::rename_entry(&app, &workspace, relative_path, new_name, None)
}

#[tauri::command]
pub fn move_entry(
    app: AppHandle,
    workspace_id: String,
    source_path: String,
    destination_path: String,
) -> Result<ChangeOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    changes::move_entry(&app, &workspace, source_path, destination_path, None)
}

#[tauri::command]
pub fn delete_entry(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    recursive: bool,
) -> Result<ChangeOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    changes::delete_entry(&app, &workspace, relative_path, recursive, None)
}

#[tauri::command]
pub fn list_changes(
    app: AppHandle,
    workspace_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<ChangeRecord>, String> {
    changes::list_changes(&app, workspace_id.as_deref(), limit.unwrap_or(40))
}

#[tauri::command]
pub fn approve_change(app: AppHandle, change_id: String) -> Result<ChangeOutcome, String> {
    let outcome = changes::approve_change(&app, &change_id)?;
    activity::sync_change(&app, &outcome.change);
    Ok(outcome)
}

#[tauri::command]
pub fn reject_change(app: AppHandle, change_id: String) -> Result<ChangeRecord, String> {
    let record = changes::reject_change(&app, &change_id)?;
    activity::sync_change(&app, &record);
    Ok(record)
}

#[tauri::command]
pub fn undo_change(app: AppHandle, change_id: String) -> Result<ChangeRecord, String> {
    let record = changes::undo_change(&app, &change_id)?;
    activity::sync_change(&app, &record);
    Ok(record)
}

