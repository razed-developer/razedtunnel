use tauri::{AppHandle, Manager};
use crate::{ai_workspace, app_state::AppState, projects::approved_workspace};

#[tauri::command]
pub async fn get_ai_workspace_status(
    app: AppHandle,
    workspace_id: String,
) -> Result<ai_workspace::AiWorkspaceStatus, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _workspace = approved_workspace(&app_for_task, &workspace_id)?;
        let state = app_for_task.state::<AppState>();
        state.ai_workspace.status(&app_for_task, &workspace_id)
    })
    .await
    .map_err(|error| format!("AI Workspace status task could not complete: {error}"))?
}

#[tauri::command]
pub async fn start_ai_workspace(
    app: AppHandle,
    workspace_id: String,
    application_id: String,
    target: Option<String>,
) -> Result<ai_workspace::AiWorkspaceStatus, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let workspace = approved_workspace(&app_for_task, &workspace_id)?;
        let state = app_for_task.state::<AppState>();
        state.ai_workspace.start(
            &app_for_task,
            &workspace,
            &application_id,
            target.as_deref(),
        )
    })
    .await
    .map_err(|error| format!("AI Workspace start task could not complete: {error}"))?
}

#[tauri::command]
pub async fn stop_ai_workspace(
    app: AppHandle,
    workspace_id: String,
    app_session_id: String,
) -> Result<ai_workspace::AiWorkspaceStatus, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _workspace = approved_workspace(&app_for_task, &workspace_id)?;
        let state = app_for_task.state::<AppState>();
        state
            .ai_workspace
            .stop_app_session(&app_for_task, &workspace_id, &app_session_id)
    })
    .await
    .map_err(|error| format!("AI Workspace stop task could not complete: {error}"))?
}

#[tauri::command]
pub async fn get_ai_workspace_frame(
    app: AppHandle,
    workspace_id: String,
    app_session_id: String,
    max_width: Option<u32>,
) -> Result<ai_workspace::AiWorkspaceFrame, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _workspace = approved_workspace(&app_for_task, &workspace_id)?;
        let state = app_for_task.state::<AppState>();
        state.ai_workspace.frame_app_session(
            &app_for_task,
            &workspace_id,
            &app_session_id,
            None,
            max_width.unwrap_or(1440),
            false,
        )
    })
    .await
    .map_err(|error| format!("AI Workspace frame task could not complete: {error}"))?
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn ai_workspace_action(
    app: AppHandle,
    workspace_id: String,
    app_session_id: String,
    action: String,
    window_id: Option<String>,
    x_ratio: Option<f64>,
    y_ratio: Option<f64>,
    click_count: Option<u8>,
    shortcut: Option<String>,
    text: Option<String>,
    delta_x: Option<i32>,
    delta_y: Option<i32>,
) -> Result<serde_json::Value, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _workspace = approved_workspace(&app_for_task, &workspace_id)?;
        let state = app_for_task.state::<AppState>();
        state.ai_workspace.action_app_session(
            &app_for_task,
            &workspace_id,
            &app_session_id,
            &action,
            window_id.as_deref(),
            x_ratio,
            y_ratio,
            click_count,
            shortcut.as_deref(),
            text.as_deref(),
            delta_x,
            delta_y,
        )
    })
    .await
    .map_err(|error| format!("AI Workspace action task could not complete: {error}"))?
}

#[tauri::command]
pub async fn ai_workspace_sequence(
    app: AppHandle,
    workspace_id: String,
    app_session_id: String,
    window_id: Option<String>,
    steps: Vec<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _workspace = approved_workspace(&app_for_task, &workspace_id)?;
        let state = app_for_task.state::<AppState>();
        state.ai_workspace.sequence_app_session(
            &app_for_task,
            &workspace_id,
            &app_session_id,
            window_id.as_deref(),
            &steps,
        )
    })
    .await
    .map_err(|error| format!("AI Workspace sequence task could not complete: {error}"))?
}

