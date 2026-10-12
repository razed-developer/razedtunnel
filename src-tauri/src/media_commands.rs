use tauri::AppHandle;
use crate::{media_inspection, models::{MediaInspection, MediaFrameExtraction, MediaDecodeValidation}, projects::approved_workspace};

#[tauri::command]
pub async fn inspect_media_file(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
) -> Result<MediaInspection, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        media_inspection::inspect(&app_for_task, &workspace, &relative_path)
    })
    .await
    .map_err(|error| format!("Media inspection worker could not complete: {error}"))?
}

#[tauri::command]
pub async fn extract_media_frame(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    task_id: String,
    timestamp_seconds: f64,
) -> Result<MediaFrameExtraction, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        media_inspection::extract_frame(
            &app_for_task,
            &workspace,
            &relative_path,
            &task_id,
            timestamp_seconds,
        )
    })
    .await
    .map_err(|error| format!("Media frame extraction worker could not complete: {error}"))?
}

#[tauri::command]
pub async fn validate_media_decode(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    check_seconds: Option<f64>,
) -> Result<MediaDecodeValidation, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        media_inspection::validate_decode(&app_for_task, &workspace, &relative_path, check_seconds)
    })
    .await
    .map_err(|error| format!("Media decode-validation worker could not complete: {error}"))?
}

