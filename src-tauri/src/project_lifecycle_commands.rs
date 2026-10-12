use tauri::AppHandle;
use crate::{activity, continuity, project_memory, project_setup, projects::approved_workspace, models::{ProjectSetupStatus, ProjectSetupOutcome, ProjectMemory}};

#[tauri::command]
pub fn get_project_setup(
    app: AppHandle,
    workspace_id: String,
) -> Result<ProjectSetupStatus, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    project_setup::detect(&workspace)
}

#[tauri::command]
pub async fn prepare_project(
    app: AppHandle,
    workspace_id: String,
) -> Result<ProjectSetupOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let workspace = approved_workspace(&app, &workspace_id)?;
        let result = project_setup::prepare(&app, &workspace)?;
        activity::sync_terminal(&app, &result.command);
        Ok(result)
    })
    .await
    .map_err(|error| format!("Project setup task could not complete: {error}"))?
}

#[tauri::command]
pub async fn get_project_memory(
    app: AppHandle,
    workspace_id: String,
) -> Result<ProjectMemory, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let workspace = approved_workspace(&app, &workspace_id)?;
        project_memory::get(&app, &workspace)
    })
    .await
    .map_err(|error| format!("Project memory task could not complete: {error}"))?
}

#[tauri::command]
pub async fn get_resume_snapshot(
    app: AppHandle,
    workspace_id: String,
) -> Result<continuity::ResumeSnapshot, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let workspace = approved_workspace(&app, &workspace_id)?;
        continuity::resume_snapshot(&app, &workspace)
    })
    .await
    .map_err(|error| format!("Resume snapshot task could not complete: {error}"))?
}

#[tauri::command]
pub fn update_project_memory(
    app: AppHandle,
    workspace_id: String,
    summary: String,
    goals: Vec<String>,
    decisions: Vec<String>,
    preferences: Vec<String>,
    next_steps: Vec<String>,
) -> Result<ProjectMemory, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    project_memory::update(
        &app,
        &workspace,
        summary,
        goals,
        decisions,
        preferences,
        next_steps,
    )
}

