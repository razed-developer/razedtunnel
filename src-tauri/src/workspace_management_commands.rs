use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use crate::{app_state::AppState, continuity, desktop_control, hardening, integrations, monitoring, project_memory, repository, semantic, access::validate_workspace_root, models::{Workspace, WorkspaceHealth}, projects::{approved_workspace, canonical_workspace_path, workspace_name, register_workspace_path, register_workspace_path_with_policy}, storage::{load_workspaces, mutate_workspaces}};

#[tauri::command]
pub async fn select_workspace(app: AppHandle) -> Result<Option<String>, String> {
    let selected = app
        .dialog()
        .file()
        .set_title("Choose a project folder")
        .blocking_pick_folder();

    selected
        .map(|file_path| {
            file_path
                .into_path()
                .map(|path| path.to_string_lossy().into_owned())
                .map_err(|error| format!("Could not resolve the selected project folder: {error}"))
        })
        .transpose()
}

#[tauri::command]
pub fn list_workspaces(app: AppHandle) -> Result<Vec<Workspace>, String> {
    load_workspaces(&app)
}

#[tauri::command]
pub fn get_workspace_health(
    app: AppHandle,
    workspace_id: String,
) -> Result<WorkspaceHealth, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    match validate_workspace_root(&workspace) {
        Ok(()) => Ok(WorkspaceHealth {
            workspace_id,
            available: true,
            message: None,
        }),
        Err(error) => Ok(WorkspaceHealth {
            workspace_id,
            available: false,
            message: Some(error),
        }),
    }
}

#[tauri::command]
pub fn relocate_workspace(
    app: AppHandle,
    workspace_id: String,
    path: String,
) -> Result<Workspace, String> {
    let canonical_path = canonical_workspace_path(&path)?;
    let canonical_string = canonical_path.to_string_lossy().into_owned();
    let canonical_name = workspace_name(&canonical_path);
    let updated = mutate_workspaces(&app, |workspaces| {
        if workspaces
            .iter()
            .any(|workspace| workspace.id != workspace_id && workspace.path == canonical_string)
        {
            return Err("That project folder is already approved in RepoTunnel.".to_string());
        }

        let workspace = workspaces
            .iter_mut()
            .find(|workspace| workspace.id == workspace_id)
            .ok_or_else(|| "That project is no longer registered with RepoTunnel.".to_string())?;

        workspace.path = canonical_string;
        workspace.name = canonical_name;
        Ok(workspace.clone())
    })?;
    hardening::log_event(
        &app,
        "INFO",
        "workspace.relocate",
        &format!("workspace_id={} name={}", updated.id, updated.name),
    );
    Ok(updated)
}

#[tauri::command]
pub fn add_workspace(app: AppHandle, path: String) -> Result<Workspace, String> {
    register_workspace_path(&app, path)
}

#[tauri::command]
pub async fn clone_git_workspace(
    app: AppHandle,
    url: String,
    parent_path: String,
) -> Result<Workspace, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let url = url.trim();
        if !url.starts_with("https://") || url.chars().any(char::is_whitespace) {
            return Err("Enter an HTTPS Git repository URL.".to_string());
        }
        let parsed = url::Url::parse(url).map_err(|_| "Invalid repository URL.".to_string())?;
        if parsed.host_str().is_none() || !parsed.username().is_empty() || parsed.password().is_some()
            || parsed.query().is_some() || parsed.fragment().is_some() || parsed.port().is_some()
        {
            return Err("Repository URL must not contain credentials, ports, queries, or fragments.".to_string());
        }
        let repo_name = parsed.path_segments()
            .and_then(|segments| segments.filter(|part| !part.is_empty()).last())
            .unwrap_or("")
            .trim_end_matches(".git");
        if repo_name.is_empty() || repo_name == "." || repo_name == ".."
            || !repo_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err("Repository URL must end in a valid repository name.".to_string());
        }
        let parent = canonical_workspace_path(&parent_path)?;
        let destination = parent.join(repo_name);
        if destination.exists() {
            return Err("Destination already exists. Choose another parent folder or register the existing checkout.".to_string());
        }
        let mut command = std::process::Command::new("git");
        // Git for Windows may reject the \\?\\ prefix returned by canonicalize().
        // Run from the validated parent and pass only the relative checkout name.
        command.args(["-c", "credential.interactive=never", "clone", "--", url])
            .arg(repo_name)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GCM_INTERACTIVE", "Never")
            .current_dir(&parent);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000);
        }
        let output = command.output().map_err(|e| format!("Could not launch Git: {e}"))?;
        if !output.status.success() {
            return Err(format!("Git clone failed: {}", String::from_utf8_lossy(&output.stderr).chars().take(600).collect::<String>()));
        }
        register_workspace_path_with_policy(&app, destination.to_string_lossy().into_owned(), true)
    }).await.map_err(|e| format!("Clone task failed: {e}"))?
}

#[tauri::command]
pub fn create_project(app: AppHandle, name: String) -> Result<Workspace, String> {
    repository::create_and_register(&app, &name)
}

#[tauri::command]
pub fn remove_workspace(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<Workspace>, String> {
    let workspaces = mutate_workspaces(&app, |workspaces| {
        let original_count = workspaces.len();
        workspaces.retain(|workspace| workspace.id != id);

        if workspaces.len() == original_count {
            return Err("That project is no longer registered with RepoTunnel.".to_string());
        }

        Ok(workspaces.clone())
    })?;
    monitoring::forget_workspace(&app, &id);
    project_memory::forget(&app, &id);
    continuity::forget(&app, &id);
    integrations::forget_workspace(&app, &id);
    desktop_control::forget_workspace(&app, &id);
    state.ai_workspace.forget_workspace(&app, &id);
    semantic::forget_workspace(&id);
    hardening::log_event(
        &app,
        "INFO",
        "workspace.remove",
        &format!("workspace_id={id}"),
    );
    Ok(workspaces)
}

