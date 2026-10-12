use tauri::AppHandle;
use crate::{access::{resolve_workspace_path, validate_workspace_root, AccessOperation}, launcher, models::{AccessCheck, CommandPolicy, LaunchActionOutcome, Workspace, WorkspaceAccessMode, WorkspaceChangePolicy}, projects::approved_workspace, storage::{load_workspaces, mutate_workspaces}};

#[tauri::command]
pub fn update_workspace_access(
    app: AppHandle,
    id: String,
    access_mode: WorkspaceAccessMode,
) -> Result<Workspace, String> {
    mutate_workspaces(&app, |workspaces| {
        let workspace = workspaces
            .iter_mut()
            .find(|workspace| workspace.id == id)
            .ok_or_else(|| "That project is no longer registered with RepoTunnel.".to_string())?;

        validate_workspace_root(workspace)?;
        workspace.access_mode = access_mode;
        Ok(workspace.clone())
    })
}

#[tauri::command]
pub fn update_workspace_change_policy(
    app: AppHandle,
    id: String,
    change_policy: WorkspaceChangePolicy,
) -> Result<Workspace, String> {
    mutate_workspaces(&app, |workspaces| {
        let workspace = workspaces
            .iter_mut()
            .find(|workspace| workspace.id == id)
            .ok_or_else(|| "That project is no longer registered with RepoTunnel.".to_string())?;

        validate_workspace_root(workspace)?;
        workspace.change_policy = change_policy;
        workspace.command_policy = match change_policy {
            WorkspaceChangePolicy::Automatic => CommandPolicy::Automatic,
            WorkspaceChangePolicy::Review => CommandPolicy::Review,
        };
        Ok(workspace.clone())
    })
}

#[tauri::command]
pub fn update_workspace_command_policy(
    app: AppHandle,
    id: String,
    command_policy: CommandPolicy,
) -> Result<Workspace, String> {
    mutate_workspaces(&app, |workspaces| {
        let workspace = workspaces
            .iter_mut()
            .find(|workspace| workspace.id == id)
            .ok_or_else(|| "That project is no longer registered with RepoTunnel.".to_string())?;

        validate_workspace_root(workspace)?;
        if workspace.change_policy == WorkspaceChangePolicy::Automatic
            && command_policy != CommandPolicy::Automatic
        {
            return Err("AI Auto always runs commands automatically. Switch the project to AI Review before changing command policy.".to_string());
        }
        workspace.command_policy = command_policy;
        Ok(workspace.clone())
    })
}

#[tauri::command]
pub fn check_workspace_access(
    app: AppHandle,
    id: String,
    relative_path: String,
    write: bool,
    must_exist: bool,
) -> Result<AccessCheck, String> {
    let workspaces = load_workspaces(&app)?;
    let workspace = workspaces
        .iter()
        .find(|workspace| workspace.id == id)
        .ok_or_else(|| "That project is not approved in RepoTunnel.".to_string())?;

    let operation = if write {
        AccessOperation::Write
    } else {
        AccessOperation::Read
    };

    match resolve_workspace_path(workspace, &relative_path, operation, must_exist) {
        Ok(_) => Ok(AccessCheck {
            allowed: true,
            reason: None,
        }),
        Err(reason) => Ok(AccessCheck {
            allowed: false,
            reason: Some(reason),
        }),
    }
}

#[tauri::command]
pub fn open_workspace_path_local(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
) -> Result<LaunchActionOutcome, String> {
    let workspace = crate::editor_commands::local_user_workspace(approved_workspace(&app, &workspace_id)?);
    launcher::request_open_workspace_path(&app, &workspace, relative_path, None)
}



