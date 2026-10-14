use tauri::AppHandle;
use crate::{access::{resolve_workspace_path, validate_workspace_root, AccessOperation}, models::{AccessCheck, CommandPolicy, Workspace, WorkspaceAccessMode, WorkspaceChangePolicy}, storage::{load_workspaces, mutate_workspaces}};

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
        set_change_policy(workspace, change_policy);
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
        set_command_policy(workspace, command_policy);
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

 
// Separate file-change approval from command-execution approval.
fn set_change_policy(workspace: &mut Workspace, policy: WorkspaceChangePolicy) {
    workspace.change_policy = policy;
}

fn set_command_policy(workspace: &mut Workspace, policy: CommandPolicy) {
    workspace.command_policy = policy;
}

#[cfg(test)]
mod policy_tests {
    use super::*;

    fn workspace() -> Workspace {
        Workspace {
            id: "test".into(),
            name: "Test".into(),
            path: ".".into(),
            added_at: 0,
            access_mode: WorkspaceAccessMode::ReadWrite,
            change_policy: WorkspaceChangePolicy::Review,
            command_policy: CommandPolicy::Review,
        }
    }

    #[test]
    fn automatic_file_changes_do_not_enable_automatic_commands() {
        let mut workspace = workspace();
        set_change_policy(&mut workspace, WorkspaceChangePolicy::Automatic);
        assert_eq!(workspace.change_policy, WorkspaceChangePolicy::Automatic);
        assert_eq!(workspace.command_policy, CommandPolicy::Review);
        set_change_policy(&mut workspace, WorkspaceChangePolicy::Review);
        assert_eq!(workspace.command_policy, CommandPolicy::Review);
    }

    #[test]
    fn command_policy_changes_do_not_change_file_review() {
        let mut workspace = workspace();
        set_command_policy(&mut workspace, CommandPolicy::Automatic);
        assert_eq!(workspace.command_policy, CommandPolicy::Automatic);
        assert_eq!(workspace.change_policy, WorkspaceChangePolicy::Review);
        set_command_policy(&mut workspace, CommandPolicy::Disabled);
        assert_eq!(workspace.change_policy, WorkspaceChangePolicy::Review);
    }
}
