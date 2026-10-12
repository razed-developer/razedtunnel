use tauri::{AppHandle, State};
use crate::{activity, browser, changes, checkpoint, execution, git, hardening, launcher, monitoring, terminal, versioning, app_state::AppState, models::{ActivityTimeline, HistoryClearResult, HistorySettings, VersionRestoreResult, VersionTimeline, WorkspaceAccessMode}, projects::approved_workspace, storage::{load_history_settings, load_workspaces, save_history_settings}};

#[tauri::command]
pub async fn get_version_timeline(
    app: AppHandle,
    workspace_id: Option<String>,
) -> Result<VersionTimeline, String> {
    tauri::async_runtime::spawn_blocking(move || {
        versioning::timeline(&app, workspace_id.as_deref())
    })
    .await
    .map_err(|error| format!("Version timeline task could not complete: {error}"))?
}

#[tauri::command]
pub async fn get_activity_timeline(
    app: AppHandle,
    workspace_id: Option<String>,
) -> Result<ActivityTimeline, String> {
    tauri::async_runtime::spawn_blocking(move || activity::timeline(&app, workspace_id.as_deref()))
        .await
        .map_err(|error| format!("Activity timeline task could not complete: {error}"))?
}

#[tauri::command]
pub fn clear_version_history(
    app: AppHandle,
    workspace_id: String,
) -> Result<HistoryClearResult, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let (removed_versions, removed_changes) =
        changes::clear_workspace_history(&app, &workspace.id)?;
    let removed_sandbox = execution::clear_workspace_history(&app, &workspace.id)?;
    let (removed_terminal, removed_processes) =
        terminal::clear_workspace_history(&app, &workspace.id)?;
    let removed_launches = launcher::clear_workspace_history(&app, &workspace.id)?;
    let removed_browser = browser::clear_workspace_history(&app, &workspace.id)?;
    let removed_git = git::clear_workspace_history(&app, &workspace.id)?;
    let removed_monitoring = monitoring::clear_workspace_history(&app, &workspace.id)?;
    let removed_activities = activity::clear_workspace(&app, &workspace.id)?;
    let removed_operational_records = removed_sandbox
        .saturating_add(removed_terminal)
        .saturating_add(removed_processes)
        .saturating_add(removed_launches)
        .saturating_add(removed_browser)
        .saturating_add(removed_git)
        .saturating_add(removed_monitoring);
    hardening::log_event(
        &app,
        "INFO",
        "history.clear",
        &format!(
            "workspace_id={} versions={} changes={} activities={} operational_records={}",
            workspace.id,
            removed_versions,
            removed_changes,
            removed_activities,
            removed_operational_records
        ),
    );
    Ok(HistoryClearResult {
        removed_versions,
        removed_changes,
        removed_activities,
        removed_operational_records,
    })
}

#[tauri::command]
pub fn get_history_settings(app: AppHandle) -> Result<HistorySettings, String> {
    load_history_settings(&app)
}

#[tauri::command]
pub fn update_history_settings(
    app: AppHandle,
    version_history_limit: Option<usize>,
    checkpoint_limit: Option<usize>,
) -> Result<HistorySettings, String> {
    if version_history_limit.is_some_and(|limit| !matches!(limit, 100 | 250 | 500)) {
        return Err(
            "Choose a version-history retention of 100, 250, 500, or Keep all.".to_string(),
        );
    }
    if checkpoint_limit.is_some_and(|limit| !matches!(limit, 10 | 25 | 50)) {
        return Err("Choose a checkpoint retention of 10, 25, 50, or Keep all.".to_string());
    }

    let settings = HistorySettings {
        version_history_limit,
        checkpoint_limit,
    };
    save_history_settings(&app, &settings)?;

    for workspace in load_workspaces(&app)? {
        if let Some(limit) = settings.version_history_limit {
            versioning::apply_retention(&app, &workspace.id, limit)?;
            activity::apply_retention(&app, &workspace.id, limit)?;
        }
        if let Some(limit) = settings.checkpoint_limit {
            checkpoint::apply_retention(&app, &workspace.id, limit)?;
        }
    }

    hardening::log_event(
        &app,
        "INFO",
        "history.settings",
        &format!(
            "version_limit={:?} checkpoint_limit={:?}",
            settings.version_history_limit, settings.checkpoint_limit
        ),
    );
    Ok(settings)
}

#[tauri::command]
pub fn restore_version(
    app: AppHandle,
    state: State<'_, AppState>,
    workspace_id: String,
    version_id: Option<String>,
) -> Result<VersionRestoreResult, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    if workspace.access_mode != WorkspaceAccessMode::ReadWrite {
        return Err(
            "Switch this project to read + write before restoring version history.".to_string(),
        );
    }
    let was_paused = state.ai_access_paused();
    state.set_ai_access_paused(true);
    let result = (|| {
        let recovery_checkpoint_id = match checkpoint::create_named_checkpoint(
            &app,
            &workspace,
            Some("Before version restore"),
        ) {
            Ok(recovery) => Some(recovery.id),
            Err(error) if checkpoint::is_capacity_error(&error) => None,
            Err(error) => return Err(error),
        };
        let restored = versioning::restore_version(
            &app,
            &workspace,
            version_id.as_deref(),
            recovery_checkpoint_id.clone(),
        )?;
        if recovery_checkpoint_id.is_some() {
            checkpoint::apply_configured_retention(&app, &workspace.id);
        }
        Ok(restored)
    })();
    state.set_ai_access_paused(was_paused);
    result
}

