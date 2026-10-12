use tauri::AppHandle;
use crate::{environment, filesystem, launcher, project_index, system_resources, workflow, projects::approved_workspace, models::{ProjectSnapshot, WorkflowReadiness, WorkspaceEnvironmentDiagnostics, EnvironmentToolDiagnostic, SystemResourceSnapshot, FileInfo, LaunchApplication}};

#[tauri::command]
pub async fn inspect_project(
    app: AppHandle,
    workspace_id: String,
    entry_limit: Option<usize>,
) -> Result<ProjectSnapshot, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let entry_limit = entry_limit.unwrap_or(800);
    tauri::async_runtime::spawn_blocking(move || {
        project_index::project_snapshot(&workspace, entry_limit)
    })
    .await
    .map_err(|error| format!("Project inspection worker could not complete: {error}"))?
}

#[tauri::command]
pub async fn get_workflow_readiness(
    app: AppHandle,
    workspace_id: String,
) -> Result<WorkflowReadiness, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    tauri::async_runtime::spawn_blocking(move || workflow::readiness(&workspace))
        .await
        .map_err(|error| format!("Workflow readiness worker could not complete: {error}"))
}

#[tauri::command]
pub async fn get_environment_diagnostics(
    app: AppHandle,
    workspace_id: String,
) -> Result<WorkspaceEnvironmentDiagnostics, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    tauri::async_runtime::spawn_blocking(move || environment::diagnostics(&workspace))
        .await
        .map_err(|error| format!("Environment diagnostics worker could not complete: {error}"))
}

#[tauri::command]
pub async fn list_tool_capabilities(
    app: AppHandle,
    workspace_id: String,
) -> Result<Vec<EnvironmentToolDiagnostic>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    tauri::async_runtime::spawn_blocking(move || environment::tool_capabilities(&workspace))
        .await
        .map_err(|error| format!("Tool-capability worker could not complete: {error}"))
}

#[tauri::command]
pub async fn get_system_resource_snapshot(
    app: AppHandle,
    workspace_id: String,
) -> Result<SystemResourceSnapshot, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    tauri::async_runtime::spawn_blocking(move || system_resources::snapshot(&workspace))
        .await
        .map_err(|error| format!("System-resource worker could not complete: {error}"))
}

#[tauri::command]
pub fn get_file_info(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
) -> Result<FileInfo, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    filesystem::file_info(&workspace, &relative_path)
}

#[tauri::command]
pub fn list_launchable_applications(
    app: AppHandle,
    workspace_id: String,
) -> Result<Vec<LaunchApplication>, String> {
    let _workspace = approved_workspace(&app, &workspace_id)?;
    Ok(launcher::list_applications())
}

