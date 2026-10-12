use tauri::AppHandle;
use crate::{activity, git, models::{GitRepositoryStatus, GitDiff, GitDiffCheck, GitCommitSummary, GitActionRecord, ChangeOutcome}, projects::approved_workspace};

#[tauri::command]
pub async fn get_git_status(
    app: AppHandle,
    workspace_id: String,
) -> Result<GitRepositoryStatus, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let workspace = approved_workspace(&app, &workspace_id)?;
        Ok(git::repository_status(&workspace))
    })
    .await
    .map_err(|error| format!("Git status task could not complete: {error}"))?
}

#[tauri::command]
pub fn get_git_diff(app: AppHandle, workspace_id: String, staged: bool) -> Result<GitDiff, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    git::diff(&workspace, staged)
}

#[tauri::command]
pub fn get_git_diff_check(
    app: AppHandle,
    workspace_id: String,
    staged: bool,
) -> Result<GitDiffCheck, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    git::diff_check(&workspace, staged)
}

#[tauri::command]
pub fn get_git_log(
    app: AppHandle,
    workspace_id: String,
    limit: Option<usize>,
) -> Result<Vec<GitCommitSummary>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    git::recent_commits(&workspace, limit.unwrap_or(12))
}

#[tauri::command]
pub fn request_git_stage(
    app: AppHandle,
    workspace_id: String,
    paths: Vec<String>,
) -> Result<GitActionRecord, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    git::request_stage(&app, &workspace, paths)
}

#[tauri::command]
pub fn request_git_commit(
    app: AppHandle,
    workspace_id: String,
    message: String,
) -> Result<GitActionRecord, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    git::request_commit(&app, &workspace, message)
}

#[tauri::command]
pub fn list_git_actions(
    app: AppHandle,
    workspace_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<GitActionRecord>, String> {
    git::list_actions(&app, workspace_id.as_deref(), limit.unwrap_or(40))
}

#[tauri::command]
pub fn approve_git_action(app: AppHandle, action_id: String) -> Result<GitActionRecord, String> {
    let record = git::approve_action(&app, &action_id)?;
    activity::sync_git(&app, &record);
    Ok(record)
}

#[tauri::command]
pub fn reject_git_action(app: AppHandle, action_id: String) -> Result<GitActionRecord, String> {
    let record = git::reject_action(&app, &action_id)?;
    activity::sync_git(&app, &record);
    Ok(record)
}

#[tauri::command]
pub fn request_git_restore_file(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
) -> Result<ChangeOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    git::request_restore_file(&app, &workspace, relative_path, None)
}

