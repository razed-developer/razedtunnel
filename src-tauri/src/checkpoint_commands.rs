use tauri::{AppHandle, State};
use crate::{activity, checkpoint, hardening, app_state::AppState, models::{CheckpointSummary, CheckpointComparison, CheckpointRestoreResult, CheckpointClearResult, ActivityKind, ActivityStatus}, projects::approved_workspace, storage::load_workspaces};

#[tauri::command]
pub fn create_checkpoint(
    app: AppHandle,
    workspace_id: String,
) -> Result<CheckpointSummary, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let summary = checkpoint::create_checkpoint(&app, &workspace)?;
    hardening::log_event(
        &app,
        "INFO",
        "checkpoint.create",
        &format!(
            "workspace_id={} files={} bytes={}",
            workspace.id, summary.file_count, summary.total_bytes
        ),
    );
    Ok(summary)
}

#[tauri::command]
pub fn list_checkpoints(
    app: AppHandle,
    workspace_id: Option<String>,
) -> Result<Vec<CheckpointSummary>, String> {
    if let Some(workspace_id) = workspace_id {
        let workspace = approved_workspace(&app, &workspace_id)?;
        return checkpoint::list_checkpoints(&app, Some(&workspace.id));
    }
    let approved: std::collections::BTreeSet<String> = load_workspaces(&app)?
        .into_iter()
        .map(|workspace| workspace.id)
        .collect();
    Ok(checkpoint::list_checkpoints(&app, None)?
        .into_iter()
        .filter(|summary| approved.contains(&summary.workspace_id))
        .collect())
}

#[tauri::command]
pub fn compare_checkpoint(
    app: AppHandle,
    workspace_id: String,
    checkpoint_id: String,
) -> Result<CheckpointComparison, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    checkpoint::compare_checkpoint(&app, &workspace, &checkpoint_id)
}

#[tauri::command]
pub fn restore_checkpoint(
    app: AppHandle,
    state: State<'_, AppState>,
    workspace_id: String,
    checkpoint_id: String,
) -> Result<CheckpointRestoreResult, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let was_paused = state.ai_access_paused();
    state.set_ai_access_paused(true);
    let result = checkpoint::restore_checkpoint(&app, &workspace, &checkpoint_id);
    state.set_ai_access_paused(was_paused);
    let result = result?;
    hardening::log_event(
        &app,
        "WARN",
        "checkpoint.restore",
        &format!(
            "workspace_id={} checkpoint_id={} restored_files={} removed_files={} pre_restore_checkpoint={}",
            workspace.id, checkpoint_id, result.restored_files, result.removed_files, result.pre_restore_checkpoint.id
        ),
    );
    let checkpoint_label = result
        .checkpoint
        .name
        .as_deref()
        .unwrap_or(&result.checkpoint.id);
    let _ = activity::record(
        &app,
        &workspace,
        None,
        ActivityKind::Files,
        "checkpointRestore",
        format!("Restored checkpoint {checkpoint_label}"),
        Some(format!(
            "Restored {} changed files · removed {} files · recovery checkpoint {}",
            result.restored_files, result.removed_files, result.pre_restore_checkpoint.id
        )),
        ActivityStatus::Succeeded,
        None,
    );
    Ok(result)
}

#[tauri::command]
pub fn delete_checkpoint(
    app: AppHandle,
    workspace_id: String,
    checkpoint_id: String,
) -> Result<(), String> {
    let _workspace = approved_workspace(&app, &workspace_id)?;
    checkpoint::delete_checkpoint(&app, &workspace_id, &checkpoint_id)?;
    hardening::log_event(
        &app,
        "INFO",
        "checkpoint.delete",
        &format!(
            "workspace_id={} checkpoint_id={}",
            workspace_id, checkpoint_id
        ),
    );
    Ok(())
}

#[tauri::command]
pub fn rename_checkpoint(
    app: AppHandle,
    workspace_id: String,
    checkpoint_id: String,
    name: Option<String>,
) -> Result<CheckpointSummary, String> {
    let _workspace = approved_workspace(&app, &workspace_id)?;
    let summary =
        checkpoint::rename_checkpoint(&app, &workspace_id, &checkpoint_id, name.as_deref())?;
    hardening::log_event(
        &app,
        "INFO",
        "checkpoint.rename",
        &format!(
            "workspace_id={} checkpoint_id={}",
            workspace_id, checkpoint_id
        ),
    );
    Ok(summary)
}

#[tauri::command]
pub fn set_checkpoint_pinned(
    app: AppHandle,
    workspace_id: String,
    checkpoint_id: String,
    pinned: bool,
) -> Result<CheckpointSummary, String> {
    let _workspace = approved_workspace(&app, &workspace_id)?;
    let summary = checkpoint::set_checkpoint_pinned(&app, &workspace_id, &checkpoint_id, pinned)?;
    hardening::log_event(
        &app,
        "INFO",
        "checkpoint.pin",
        &format!(
            "workspace_id={} checkpoint_id={} pinned={}",
            workspace_id, checkpoint_id, pinned
        ),
    );
    Ok(summary)
}

#[tauri::command]
pub fn clear_checkpoints(
    app: AppHandle,
    workspace_id: Option<String>,
) -> Result<CheckpointClearResult, String> {
    let workspace_ids = if let Some(workspace_id) = workspace_id {
        vec![approved_workspace(&app, &workspace_id)?.id]
    } else {
        load_workspaces(&app)?
            .into_iter()
            .map(|workspace| workspace.id)
            .collect::<Vec<_>>()
    };
    let removed_checkpoints = checkpoint::clear_checkpoints(&app, &workspace_ids)?;
    hardening::log_event(
        &app,
        "INFO",
        "checkpoint.clear",
        &format!(
            "scope={} checkpoints={}",
            if workspace_ids.len() == 1 {
                "project"
            } else {
                "all-projects"
            },
            removed_checkpoints
        ),
    );
    Ok(CheckpointClearResult {
        removed_checkpoints,
    })
}

