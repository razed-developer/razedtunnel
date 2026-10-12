
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

use crate::{
    access::{resolve_workspace_path, validate_workspace_root, AccessOperation},
    activity, ai_workspace,
    app_state::AppState,
    browser, changes, checkpoint, continuity, conversation, desktop_control, environment,
    execution, filesystem, git, github, gmail_access, hardening, integrations, launcher, mcp_auth,
    media_inspection, mcp_server,
    model_hub::{
        self, ModelHubSnapshot, ModelProviderId, ModelSelection, ModelTestResult, RuntimeStatus,
    },
    model_trial::{self, ModelTrialSnapshot, TrialMode},
    models::{
        AccessCheck, ActivityKind, ActivityStatus, ActivityTimeline, AiAccessStatus,
        BrowserActionOutcome, BrowserActionRecord, BrowserApplication, BrowserAutomationStatus,
        BrowserDiagnostics, BrowserDownload, BrowserDownloadSetup, BrowserPageInspection,
        BrowserScreenshot, BrowserTab, BrowserUploadResult, BrowserVisualSelection, ChangeOutcome,
        ChangeRecord, ChatConnectionStatus, CheckpointClearResult, CheckpointComparison,
        CheckpointRestoreResult, CheckpointSummary, CommandOutcome, CommandPolicy, CommandPreset,
        CommandRecord, DirectoryEntry, EnvironmentToolDiagnostic, ExecutionStatus, FileContent,
        FileInfo, GatewayStatus, GitActionRecord, GitCommitSummary, GitDiff, GitDiffCheck,
        GitRepositoryStatus, HistoryClearResult, HistorySettings,
        LaunchActionOutcome, LaunchActionRecord, LaunchApplication, ManagedProcessOutcome,
        ManagedProcessOutput, ManagedProcessRecord, ManagedProcessWaitResult,
        MediaDecodeValidation, MediaFrameExtraction, MediaInspection, MonitoringFileEvent,
        MonitoringSnapshot, MonitoringStatus, ProjectMemory, ProjectSetupOutcome,
        ProjectSetupStatus, ProjectSnapshot, PublicTunnelStatus, RuntimeDiagnostics,
        SafetyScanCheck, SafetyScanResult, SearchMatch, SystemResourceSnapshot, TeamSessionSummary,
        TeamSnapshot, TempWorkspaceCleanupResult, TempWorkspaceFileResult, TempWorkspaceInfo,
        TerminalCommandOutcome, TerminalCommandRecord, VersionRestoreResult, VersionTimeline,
        WorkflowReadiness, Workspace, WorkspaceAccessMode, WorkspaceChangePolicy,
        WorkspaceEnvironmentDiagnostics, WorkspaceHealth,
    },
    monitoring, phone, projects::{canonical_workspace_path, workspace_name, approved_workspace, register_workspace_path, register_workspace_path_with_policy}, project_context, project_index, project_memory, project_setup,
    public_tunnel::PublicTunnelProvider,
    repository, semantic,
    storage::{
        load_history_settings, load_workspaces, mutate_workspaces, save_ai_access_paused,
        save_history_settings,
    },
    system_resources, team, temp_workspace, terminal, updates, versioning, workflow,
};

// Video-only imports must not pull legacy modules into the lean command layer.
#[cfg(feature = "legacy_video")]
use crate::{
    video, video_assets, video_director, video_narration, video_preview, video_production,
    video_qa, video_render, video_scene, video_story_render,
};

fn status(app: &AppHandle, state: &AppState) -> Result<GatewayStatus, String> {
    let (running, port) = state.gateway_status()?;

    let (task_tool_successes, task_tool_failures) = mcp_server::task_tool_outcomes();
    Ok(GatewayStatus {
        task_tool_successes,
        task_tool_failures,
        running,
        port,
        workspace_count: load_workspaces(app)?.len(),
    })
}

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
    team::forget_workspace(&app, &id);
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

#[tauri::command]
pub fn open_url(
    app: AppHandle,
    workspace_id: String,
    url: String,
    application_id: Option<String>,
) -> Result<LaunchActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    launcher::request_open_url(&app, &workspace, url, application_id)
}

#[tauri::command]
pub fn open_workspace_path(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    application_id: Option<String>,
) -> Result<LaunchActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    launcher::request_open_workspace_path(&app, &workspace, relative_path, application_id)
}

#[tauri::command]
pub fn launch_application(
    app: AppHandle,
    workspace_id: String,
    application_id: String,
) -> Result<LaunchActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    launcher::request_launch_application(&app, &workspace, application_id)
}

#[tauri::command]
pub fn list_launch_history(
    app: AppHandle,
    workspace_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<LaunchActionRecord>, String> {
    launcher::list_history(&app, workspace_id.as_deref(), limit.unwrap_or(60))
}

#[tauri::command]
pub fn approve_launch_action(
    app: AppHandle,
    launch_id: String,
) -> Result<LaunchActionRecord, String> {
    let record = launcher::get_action(&app, &launch_id)?;
    let workspace = approved_workspace(&app, &record.workspace_id)?;
    let approved = launcher::approve_action(&app, &workspace, &launch_id)?;
    activity::sync_launch(&app, &approved);
    Ok(approved)
}

#[tauri::command]
pub fn reject_launch_action(
    app: AppHandle,
    launch_id: String,
) -> Result<LaunchActionRecord, String> {
    let record = launcher::reject_action(&app, &launch_id)?;
    activity::sync_launch(&app, &record);
    Ok(record)
}

#[tauri::command]
pub fn list_automation_browsers(
    app: AppHandle,
    workspace_id: String,
) -> Result<Vec<BrowserApplication>, String> {
    let _workspace = approved_workspace(&app, &workspace_id)?;
    Ok(browser::list_applications())
}

#[tauri::command]
pub fn get_browser_automation_status(
    app: AppHandle,
    workspace_id: String,
) -> Result<BrowserAutomationStatus, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    Ok(browser::status(&app, &scope))
}

#[tauri::command]
pub async fn start_browser_automation(
    app: AppHandle,
    workspace_id: String,
    application_id: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_start(&app_for_task, &scope, &application_id)
    })
    .await
    .map_err(|error| format!("Browser start task could not complete: {error}"))?
}

#[tauri::command]
pub async fn stop_browser_automation(
    app: AppHandle,
    workspace_id: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || browser::request_stop(&app_for_task, &scope))
        .await
        .map_err(|error| format!("Browser stop task could not complete: {error}"))?
}

#[tauri::command]
pub async fn list_browser_tabs(
    app: AppHandle,
    workspace_id: String,
) -> Result<Vec<BrowserTab>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || browser::list_tabs(&app_for_task, &scope))
        .await
        .map_err(|error| format!("Browser tab task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_open_tab(
    app: AppHandle,
    workspace_id: String,
    url: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_open_tab(&app_for_task, &scope, &url)
    })
    .await
    .map_err(|error| format!("Browser open-tab task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_activate_tab(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_activate_tab(&app_for_task, &scope, &tab_id)
    })
    .await
    .map_err(|error| format!("Browser activate-tab task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_close_tab(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_close_tab(&app_for_task, &scope, &tab_id)
    })
    .await
    .map_err(|error| format!("Browser close-tab task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_navigate(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    url: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_navigate(&app_for_task, &scope, &tab_id, &url)
    })
    .await
    .map_err(|error| format!("Browser navigation task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_click(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    selector: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_click(&app_for_task, &scope, &tab_id, &selector)
    })
    .await
    .map_err(|error| format!("Browser click task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_type(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    selector: String,
    text: String,
    clear_first: bool,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_type(
            &app_for_task,
            &scope,
            &tab_id,
            &selector,
            &text,
            clear_first,
        )
    })
    .await
    .map_err(|error| format!("Browser typing task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_scroll(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    delta_x: i32,
    delta_y: i32,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_scroll(&app_for_task, &scope, &tab_id, delta_x, delta_y)
    })
    .await
    .map_err(|error| format!("Browser scroll task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_reload(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
) -> Result<BrowserActionOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::request_reload(&app_for_task, &scope, &tab_id)
    })
    .await
    .map_err(|error| format!("Browser reload task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_configure_downloads(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    task_id: String,
) -> Result<BrowserDownloadSetup, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::configure_downloads(&app_for_task, &scope, &tab_id, &task_id)
    })
    .await
    .map_err(|error| format!("Browser download configuration task could not complete: {error}"))?
}

#[tauri::command]
pub fn list_browser_downloads(
    app: AppHandle,
    workspace_id: String,
) -> Result<Vec<BrowserDownload>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    browser::list_downloads(&app, &scope)
}

#[tauri::command]
pub async fn cancel_browser_download(
    app: AppHandle,
    workspace_id: String,
    guid: String,
) -> Result<BrowserDownload, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::cancel_download(&app_for_task, &scope, &guid)
    })
    .await
    .map_err(|error| format!("Browser download cancel task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_upload_file(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    selector: String,
    relative_path: String,
) -> Result<BrowserUploadResult, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::upload_file(&app_for_task, &scope, &tab_id, &selector, &relative_path)
    })
    .await
    .map_err(|error| format!("Browser upload task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_inspect_page(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    selector: Option<String>,
    max_chars: Option<usize>,
) -> Result<BrowserPageInspection, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::inspect_page(
            &app_for_task,
            &scope,
            &tab_id,
            selector.as_deref(),
            max_chars.unwrap_or(12000),
        )
    })
    .await
    .map_err(|error| format!("Browser inspection task could not complete: {error}"))?
}

#[tauri::command]
pub async fn browser_pick_element(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    x_ratio: f64,
    y_ratio: f64,
) -> Result<BrowserVisualSelection, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::pick_visual_element(&app_for_task, &scope, &tab_id, x_ratio, y_ratio)
    })
    .await
    .map_err(|error| format!("Browser visual selection task could not complete: {error}"))?
}

#[tauri::command]
pub fn get_browser_visual_selection(
    app: AppHandle,
    workspace_id: String,
) -> Result<Option<BrowserVisualSelection>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    browser::get_visual_selection(&scope)
}

#[tauri::command]
pub async fn browser_take_screenshot(
    app: AppHandle,
    workspace_id: String,
    tab_id: String,
    full_page: bool,
) -> Result<BrowserScreenshot, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        browser::screenshot(&app_for_task, &scope, &tab_id, full_page)
    })
    .await
    .map_err(|error| format!("Browser screenshot task could not complete: {error}"))?
}

#[tauri::command]
pub fn get_browser_diagnostics(
    app: AppHandle,
    workspace_id: String,
    tab_id: Option<String>,
    limit: Option<usize>,
) -> Result<BrowserDiagnostics, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let scope = browser::BrowserScope::new(&workspace, None);
    browser::diagnostics(&app, &scope, tab_id.as_deref(), limit.unwrap_or(40))
}
#[tauri::command]
pub fn list_browser_history(
    app: AppHandle,
    workspace_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<BrowserActionRecord>, String> {
    browser::list_history(&app, workspace_id.as_deref(), limit.unwrap_or(60))
}

#[tauri::command]
pub async fn approve_browser_action(
    app: AppHandle,
    action_id: String,
) -> Result<BrowserActionRecord, String> {
    let record = browser::get_action(&app, &action_id)?;
    let workspace = approved_workspace(&app, &record.workspace_id)?;
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let approved = browser::approve_action(&app_for_task, &workspace, &action_id)?;
        activity::sync_browser(&app_for_task, &approved);
        Ok(approved)
    })
    .await
    .map_err(|error| format!("Browser approval task could not complete: {error}"))?
}

#[tauri::command]
pub fn reject_browser_action(
    app: AppHandle,
    action_id: String,
) -> Result<BrowserActionRecord, String> {
    let record = browser::reject_action(&app, &action_id)?;
    activity::sync_browser(&app, &record);
    Ok(record)
}

#[tauri::command]
pub fn get_monitoring_status(
    app: AppHandle,
    workspace_id: String,
) -> Result<MonitoringStatus, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    Ok(monitoring::status(&app, &workspace))
}

#[tauri::command]
pub async fn start_workspace_monitoring(
    app: AppHandle,
    workspace_id: String,
) -> Result<MonitoringStatus, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    tauri::async_runtime::spawn_blocking(move || monitoring::start_monitoring(&app, &workspace))
        .await
        .map_err(|error| format!("Monitoring start worker could not complete: {error}"))?
}

#[tauri::command]
pub async fn stop_workspace_monitoring(
    app: AppHandle,
    workspace_id: String,
) -> Result<MonitoringStatus, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    tauri::async_runtime::spawn_blocking(move || monitoring::stop_monitoring(&app, &workspace))
        .await
        .map_err(|error| format!("Monitoring stop worker could not complete: {error}"))?
}

#[tauri::command]
pub async fn get_monitoring_snapshot(
    app: AppHandle,
    workspace_id: String,
) -> Result<MonitoringSnapshot, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    tauri::async_runtime::spawn_blocking(move || monitoring::snapshot(&app, &workspace))
        .await
        .map_err(|error| format!("Monitoring snapshot worker could not complete: {error}"))?
}

#[tauri::command]
pub fn list_monitoring_file_events(
    app: AppHandle,
    workspace_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<MonitoringFileEvent>, String> {
    monitoring::list_file_events(&app, workspace_id.as_deref(), limit.unwrap_or(60))
}

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

#[tauri::command]
pub fn run_safety_scan(app: AppHandle, workspace_id: String) -> Result<SafetyScanResult, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let snapshot = project_index::project_snapshot(&workspace, 1600)?;
    let readiness = workflow::readiness(&workspace);
    let sandbox = execution::execution_status();
    let git_status = git::repository_status(&workspace);
    let pending_reviews = changes::list_changes(&app, Some(&workspace.id), 100)?
        .into_iter()
        .filter(|change| change.status == crate::models::ChangeStatus::Pending)
        .count();

    let mut ignored_items = vec![
        ".gitignore and .ignore rules are respected.".to_string(),
        "Generated dependency/build folders are excluded from AI discovery and checkpoints."
            .to_string(),
    ];
    ignored_items.extend(
        snapshot
            .overview
            .ignored_entries
            .iter()
            .map(|entry| format!("Excluded: {entry}")),
    );
    let unlisted_ignored = snapshot
        .overview
        .ignored_entry_count
        .saturating_sub(snapshot.overview.ignored_entries.len());
    if unlisted_ignored > 0 {
        ignored_items.push(format!(
            "{unlisted_ignored} additional ignored or protected entries were excluded without listing sensitive path names."
        ));
    }
    ignored_items.push("Ignored content is not copied into RepoTunnel checkpoints.".to_string());

    let mut checks = vec![
        SafetyScanCheck {
            key: "workspace-boundary".to_string(),
            title: "Workspace boundary".to_string(),
            status: "pass".to_string(),
            detail: "AI file access is restricted to this approved project root; parent traversal and symlink escapes are blocked.".to_string(),
            items: vec![format!("Approved root: {}", workspace.path), "Parent-directory traversal blocked".to_string(), "Symlink escapes outside the approved root blocked".to_string()],
        },
        SafetyScanCheck {
            key: "secret-protection".to_string(),
            title: "Secret protection".to_string(),
            status: "pass".to_string(),
            detail: "Common credential files, private keys, and .env secrets are blocked from AI file access.".to_string(),
            items: vec![".env and .env.* protected".to_string(), "Private-key formats (.pem, .key, .p12, .pfx, keystores) protected".to_string(), "Credential/config files such as .npmrc and service-account files protected".to_string()],
        },
        SafetyScanCheck {
            key: "version-protection".to_string(),
            title: "Version protection".to_string(),
            status: "pass".to_string(),
            detail: if workspace.access_mode == crate::models::WorkspaceAccessMode::ReadOnly {
                "This project is read-only; AI edits are blocked.".to_string()
            } else if workspace.change_policy == WorkspaceChangePolicy::Review {
                "AI file proposals wait for local approval; approved edits are saved into reversible local version history.".to_string()
            } else {
                "Compatible AI file edits apply automatically and RepoTunnel saves reversible local versions.".to_string()
            },
            items: vec![format!("Access mode: {:?}", workspace.access_mode), "Automatic local version history enabled".to_string(), "Previous and later saved versions remain available after restore".to_string()],
        },
        SafetyScanCheck {
            key: "ignored-content".to_string(),
            title: "Ignored & generated content".to_string(),
            status: "pass".to_string(),
            detail: format!(
                "{} ignored entries were excluded from the project scan; safe examples are listed below when available.",
                snapshot.overview.ignored_entry_count
            ),
            items: ignored_items,
        },
        SafetyScanCheck {
            key: "sandbox".to_string(),
            title: "Command sandbox".to_string(),
            status: if sandbox.sandbox_available { "pass" } else { "warning" }.to_string(),
            detail: sandbox.message.clone().unwrap_or_else(|| if sandbox.sandbox_available {
                "The native OS sandbox is available for isolated project checks.".to_string()
            } else {
                "The native OS sandbox is unavailable, so project commands will be refused.".to_string()
            }),
            items: vec![sandbox.sandbox_version.clone().unwrap_or_else(|| "Native OS sandbox not detected".to_string()), "Network disabled for disposable project checks".to_string(), "Checks run in a disposable project copy".to_string()],
        },
        SafetyScanCheck {
            key: "git".to_string(),
            title: "Git protection".to_string(),
            status: if git_status.available { "pass" } else { "warning" }.to_string(),
            detail: if git_status.available {
                format!("Git is available{}; staging and commits require local approval.", git_status.branch.as_ref().map(|branch| format!(" on {branch}")).unwrap_or_default())
            } else {
                git_status.message.clone().unwrap_or_else(|| "This project is not currently a Git repository.".to_string())
            },
            items: vec![git_status.branch.as_ref().map(|branch| format!("Branch: {branch}")).unwrap_or_else(|| "No Git branch detected".to_string()), format!("Staged: {} · Unstaged: {} · Untracked: {}", git_status.staged_count, git_status.unstaged_count, git_status.untracked_count), "Staging and commits require local approval".to_string()],
        },
    ];

    if snapshot.overview.truncated {
        checks.push(SafetyScanCheck {
            key: "index-limit".to_string(),
            title: "Project size".to_string(),
            status: "warning".to_string(),
            detail: "The safety scan reached its project-index display limit; core access protections remain active.".to_string(),
            items: vec![format!("Accessible files counted: {}", snapshot.overview.file_count), format!("Ignored entries: {}", snapshot.overview.ignored_entry_count)],
        });
    }

    let level = if checks.iter().any(|check| check.status == "warning")
        || readiness.level != crate::models::WorkflowReadinessLevel::Ready
    {
        "attention"
    } else {
        "protected"
    };

    Ok(SafetyScanResult {
        workspace_id: workspace.id,
        workspace_name: workspace.name,
        level: level.to_string(),
        file_count: snapshot.overview.file_count,
        ignored_entry_count: snapshot.overview.ignored_entry_count,
        pending_reviews,
        checks,
    })
}

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

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_tools_status(app: AppHandle) -> Result<video::VideoToolsStatus, String> {
    video::tools_status(&app)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub async fn install_video_tools(app: AppHandle) -> Result<video::VideoToolsStatus, String> {
    tauri::async_runtime::spawn_blocking(move || video::install_missing_tools(&app))
        .await
        .map_err(|error| format!("Video helper installation task could not complete: {error}"))?
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn start_video_analysis(
    app: AppHandle,
    workspace_id: String,
    source: String,
    mode: String,
    start_seconds: Option<f64>,
    end_seconds: Option<f64>,
    max_frames: Option<usize>,
) -> Result<video::VideoAnalysisJob, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video::start_analysis(
        app,
        workspace,
        source,
        mode,
        start_seconds,
        end_seconds,
        max_frames,
    )
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_analysis_job(job_id: String) -> Result<video::VideoAnalysisJob, String> {
    video::get_job(&job_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn list_video_analysis_jobs(
    workspace_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<video::VideoAnalysisJob>, String> {
    video::list_jobs(workspace_id.as_deref(), limit.unwrap_or(20))
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_analysis_result(
    app: AppHandle,
    job_id: String,
) -> Result<video::VideoAnalysisResult, String> {
    video::get_result(&app, &job_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn cancel_video_analysis(job_id: String) -> Result<video::VideoAnalysisJob, String> {
    video::cancel_analysis(&job_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn clear_video_cache(app: AppHandle) -> Result<video::VideoToolsStatus, String> {
    video::clear_cache(&app)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_video_project(
    app: AppHandle,
    workspace_id: String,
    name: String,
    production_mode: Option<String>,
    aspect_ratio: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    fps: Option<u32>,
) -> Result<video_production::VideoProductionProject, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::create_project_with_mode(
        &workspace,
        &name,
        production_mode.as_deref(),
        aspect_ratio.as_deref(),
        width,
        height,
        fps,
    )
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn list_video_projects(
    app: AppHandle,
    workspace_id: String,
) -> Result<Vec<video_production::VideoProductionProject>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::list_projects(&workspace)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn import_video_project_folder(
    app: AppHandle,
    workspace_id: String,
    folder_path: String,
) -> Result<video_production::VideoProductionProject, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::import_folder(&workspace, &folder_path)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn set_video_project_pinned(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    pinned: bool,
) -> Result<video_production::VideoProductionProject, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::set_project_pinned(&workspace, &project_id, pinned)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn set_video_project_resource_policy(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    policy: video_production::VideoProductionResourcePolicy,
) -> Result<video_production::VideoProductionProject, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::set_resource_policy(&workspace, &project_id, policy)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn upsert_video_project_scene(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    scene: video_production::VideoProductionSceneInput,
) -> Result<video_production::VideoProductionScene, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::upsert_scene(&workspace, &project_id, scene)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_project_scene(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    scene_id: String,
) -> Result<video_production::VideoProductionScene, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::get_scene(&workspace, &project_id, &scene_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn list_video_project_scenes(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<Vec<video_production::VideoProductionScene>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::list_scenes(&workspace, &project_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn list_video_project_files(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<Vec<video_production::VideoProjectFile>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::list_project_files(&workspace, &project_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn read_video_project_text_file(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    relative_path: String,
) -> Result<String, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::read_project_text_file(&workspace, &project_id, &relative_path)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn delete_video_project(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<(), String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::delete_project(&workspace, &project_id)?;
    video_preview::clear_all(&app);
    Ok(())
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_project(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<video_production::VideoProductionProject, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::get_project(&workspace, &project_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_story_capabilities() -> video_director::StoryProductionCapabilities {
    video_director::production_capabilities()
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn compile_video_story_plan(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    input: video_director::StoryDirectorInput,
) -> Result<video_director::StoryDirectorPlan, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_director::compile_plan(&workspace, &project_id, input)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_story_plan(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<video_director::StoryDirectorPlan, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_director::get_plan(&workspace, &project_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_story_animatic_plan(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<video_director::StoryAnimaticPlan, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_director::get_animatic_plan(&workspace, &project_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn render_video_story_animatic(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<video_story_render::StoryAnimaticRender, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_story_render::render_animatic(&app, &workspace, &project_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_story_qa(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<video_director::NarrativeQaReport, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_director::get_narrative_qa(&workspace, &project_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_story_render_queue(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<video_director::StoryRenderQueue, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_director::get_render_queue(&workspace, &project_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn record_video_story_shot_render(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    input: video_director::StoryShotRenderInput,
) -> Result<video_director::StoryRenderQueue, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_director::record_shot_render(&workspace, &project_id, input)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn start_video_story_shot_render(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    shot_id: String,
    force: Option<bool>,
) -> Result<video_story_render::StoryShotRenderJob, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_story_render::start_shot_render(
        &app,
        &workspace,
        &project_id,
        &shot_id,
        force.unwrap_or(false),
    )
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_story_shot_render(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    job_id: String,
) -> Result<video_story_render::StoryShotRenderJob, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_story_render::get_shot_render(&workspace, &project_id, &job_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn cancel_video_story_shot_render(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    job_id: String,
) -> Result<video_story_render::StoryShotRenderJob, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_story_render::cancel_shot_render(&workspace, &project_id, &job_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn update_video_project_status(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    status: String,
    detail: Option<String>,
) -> Result<video_production::VideoProductionProject, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::update_project_status(&workspace, &project_id, &status, detail.as_deref())
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn write_video_project_document(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    document: String,
    content: String,
) -> Result<video_production::VideoProductionDocument, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::write_document(&workspace, &project_id, &document, &content)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn read_video_project_document(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    document: String,
) -> Result<video_production::VideoProductionDocument, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_production::read_document(&workspace, &project_id, &document)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn start_video_project_recording(
    app: AppHandle,
    state: State<'_, AppState>,
    workspace_id: String,
    project_id: String,
    app_session_id: String,
    fps: Option<u32>,
    max_seconds: Option<u32>,
) -> Result<video_production::VideoRecordingStatus, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let target =
        state
            .ai_workspace
            .recording_target_app_session(&app, &workspace_id, &app_session_id)?;
    video_production::start_ai_workspace_recording(
        &app,
        &workspace,
        &project_id,
        &app_session_id,
        &target.display,
        &target.xauth_path,
        target.x,
        target.y,
        target.width,
        target.height,
        fps,
        max_seconds,
    )
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_project_recording(
    workspace_id: String,
    project_id: Option<String>,
) -> Result<Option<video_production::VideoRecordingStatus>, String> {
    video_production::get_recording_status(&workspace_id, project_id.as_deref())
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn stop_video_project_recording(
    workspace_id: String,
    project_id: String,
) -> Result<video_production::VideoRecordingStatus, String> {
    video_production::stop_recording(&workspace_id, &project_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn validate_video_project_scene(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    scene: video_scene::VideoSceneSpec,
) -> Result<video_scene::VideoSceneLayoutReport, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_scene::validate_scene_layout(&workspace, &project_id, &scene)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub async fn render_video_project_scene(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    scene: video_scene::VideoSceneSpec,
) -> Result<video_scene::VideoSceneRender, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        video_scene::render_scene(&app_for_task, &workspace, &project_id, scene)
    })
    .await
    .map_err(|error| format!("Generated-scene render task could not complete: {error}"))?
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub async fn render_video_project_diagram(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    diagram: video_scene::VideoDiagramSpec,
) -> Result<video_scene::VideoSceneRender, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        video_scene::render_diagram(&app_for_task, &workspace, &project_id, diagram)
    })
    .await
    .map_err(|error| format!("Semantic diagram render task could not complete: {error}"))?
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn start_video_project_render(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    request: video_render::VideoRenderRequest,
) -> Result<video_render::VideoRenderJob, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_render::start_render_job(&app, &workspace, &project_id, request)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_project_render(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    job_id: String,
) -> Result<video_render::VideoRenderJob, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_render::get_render_job(&workspace, &project_id, &job_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn list_video_project_renders(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<Vec<video_render::VideoRenderJob>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_render::list_render_jobs(&workspace, &project_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn cancel_video_project_render(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    job_id: String,
) -> Result<video_render::VideoRenderJob, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_render::cancel_render_job(&workspace, &project_id, &job_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_project_pipeline_status(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<video_render::VideoPipelineStatus, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_render::pipeline_status(&workspace, &project_id)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn clean_video_project(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    request: video_render::VideoCleanupRequest,
) -> Result<video_render::VideoCleanupReport, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_render::clean_project(&workspace, &project_id, request)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn qa_video_project(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    asset_path: Option<String>,
) -> Result<video_qa::VideoQaReport, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_qa::qa_project(&app, &workspace, &project_id, asset_path.as_deref())
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub async fn render_video_project_timeline(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    request: video_render::VideoRenderRequest,
) -> Result<video_render::VideoRenderResult, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        video_render::render_project(&app_for_task, &workspace, &project_id, request)
    })
    .await
    .map_err(|error| format!("Video timeline render task could not complete: {error}"))?
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub async fn prepare_video_project_preview(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
) -> Result<video_preview::VideoPreviewSource, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        video_preview::prepare_preview(&app_for_task, &workspace, &project_id)
    })
    .await
    .map_err(|error| format!("Video preview preparation could not complete: {error}"))?
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub async fn prepare_video_project_file_preview(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    relative_path: String,
) -> Result<video_preview::VideoPreviewSource, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        video_preview::prepare_file_preview(&app_for_task, &workspace, &project_id, &relative_path)
    })
    .await
    .map_err(|error| format!("Video file preview preparation could not complete: {error}"))?
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub async fn read_video_preview_chunk(
    app: AppHandle,
    preview_path: String,
    offset: u64,
    length: u64,
) -> Result<tauri::ipc::Response, String> {
    let app_for_task = app.clone();
    let bytes = tauri::async_runtime::spawn_blocking(move || {
        video_preview::read_preview_chunk(&app_for_task, &preview_path, offset, length)
    })
    .await
    .map_err(|error| format!("Video preview read task could not complete: {error}"))??;
    Ok(tauri::ipc::Response::new(bytes))
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn list_video_asset_sources() -> Vec<video_assets::VideoAssetSource> {
    video_assets::registry()
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn record_video_asset_license(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    input: video_assets::VideoAssetLicenseInput,
) -> Result<video_assets::VideoAssetLicenseRecord, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_assets::record_license(&workspace, &project_id, input)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn get_video_narration_providers(
    app: AppHandle,
) -> Vec<video_narration::NarrationProviderStatus> {
    video_narration::provider_status(&app)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub fn create_video_project_subtitles(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    language: String,
    text: String,
    duration_seconds: Option<f64>,
) -> Result<video_narration::SubtitleAsset, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    video_narration::create_subtitles(&workspace, &project_id, &language, &text, duration_seconds)
}

#[cfg(feature = "legacy_video")]
#[tauri::command]
pub async fn synthesize_video_project_narration(
    app: AppHandle,
    workspace_id: String,
    project_id: String,
    request: video_narration::NarrationRequest,
) -> Result<video_narration::NarrationAsset, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        video_narration::synthesize(&app_for_task, &workspace, &project_id, request)
    })
    .await
    .map_err(|error| format!("Narration task could not complete: {error}"))?
}

#[tauri::command]
pub fn get_ai_access_status(state: State<'_, AppState>) -> Result<AiAccessStatus, String> {
    Ok(AiAccessStatus {
        paused: state.ai_access_paused(),
    })
}

#[tauri::command]
pub fn set_ai_access_paused(
    app: AppHandle,
    state: State<'_, AppState>,
    paused: bool,
) -> Result<AiAccessStatus, String> {
    if paused {
        // Emergency pause must take effect even if persisting the setting fails.
        state.set_ai_access_paused(true);
        terminal::stop_all_activity(&app);
        browser::stop_all_activity();
        #[cfg(feature = "legacy_video")]
        video::stop_all_activity();
        #[cfg(feature = "legacy_video")]
        video_production::stop_all_activity();
        save_ai_access_paused(&app, true).map_err(|error| {
            format!(
                "AI access is paused for this session, but RepoTunnel could not save the paused state: {error}"
            )
        })?;
    } else {
        // Resume remains persistence-first so a storage failure cannot fail open.
        save_ai_access_paused(&app, false)?;
        state.set_ai_access_paused(false);
    }
    hardening::log_event(
        &app,
        "INFO",
        if paused {
            "ai_access.pause"
        } else {
            "ai_access.resume"
        },
        if paused {
            "MCP workspace access paused by the user."
        } else {
            "MCP workspace access resumed by the user."
        },
    );
    Ok(AiAccessStatus { paused })
}

#[tauri::command]
pub fn get_gateway_status(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<GatewayStatus, String> {
    status(&app, state.inner())
}

#[tauri::command]
pub fn start_gateway(app: AppHandle, state: State<'_, AppState>) -> Result<GatewayStatus, String> {
    state.start_gateway(app.clone())?;
    hardening::log_event(
        &app,
        "INFO",
        "gateway.start",
        "Loopback MCP gateway started.",
    );
    if state.public_tunnel_status(&app)?.configured {
        if let Err(error) = state.start_public_tunnel(app.clone()) {
            hardening::log_event(&app, "WARN", "public_tunnel.start", &error);
        }
    }
    status(&app, state.inner())
}

#[tauri::command]
pub async fn stop_gateway(app: AppHandle) -> Result<GatewayStatus, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_for_task.state::<AppState>();
        state.stop_gateway()?;
        hardening::log_event(
            &app_for_task,
            "INFO",
            "gateway.stop",
            "Loopback MCP gateway and managed remote connections stopped cleanly.",
        );
        status(&app_for_task, state.inner())
    })
    .await
    .map_err(|error| format!("Gateway stop task could not complete: {error}"))?
}

#[tauri::command]
pub async fn get_public_tunnel_status(app: AppHandle) -> Result<PublicTunnelStatus, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_for_task.state::<AppState>();
        state.public_tunnel_status(&app_for_task)
    })
    .await
    .map_err(|error| format!("Public connection status task could not complete: {error}"))?
}

#[tauri::command]
pub fn configure_public_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
    provider: PublicTunnelProvider,
    credential: String,
    public_url: Option<String>,
) -> Result<PublicTunnelStatus, String> {
    let status = state.configure_public_tunnel(app.clone(), provider, credential, public_url)?;
    hardening::log_event(
        &app,
        "INFO",
        "public_tunnel.configure",
        &format!(
            "User-specific {} public connection configured and started.",
            status.provider
        ),
    );
    Ok(status)
}

#[tauri::command]
pub async fn restart_public_tunnel(app: AppHandle) -> Result<PublicTunnelStatus, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_for_task.state::<AppState>();
        state.stop_public_tunnel()?;
        state.start_public_tunnel(app_for_task.clone())?;
        hardening::log_event(
            &app_for_task,
            "INFO",
            "public_tunnel.restart",
            "Public connection restarted.",
        );
        state.public_tunnel_status(&app_for_task)
    })
    .await
    .map_err(|error| format!("Public connection restart task could not complete: {error}"))?
}

#[tauri::command]
pub async fn provision_direct_https_certificate(
    app: AppHandle,
    staging: bool,
) -> Result<PublicTunnelStatus, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_for_task.state::<AppState>();
        let status = state.provision_direct_certificate(app_for_task.clone(), staging)?;
        hardening::log_event(
            &app_for_task,
            "INFO",
            "public_tunnel.direct_certificate",
            if staging {
                "Direct HTTPS Let's Encrypt staging certificate provisioned."
            } else {
                "Direct HTTPS Let's Encrypt trusted certificate provisioned."
            },
        );
        Ok::<PublicTunnelStatus, String>(status)
    })
    .await
    .map_err(|error| format!("Direct HTTPS certificate task could not complete: {error}"))?
}

#[tauri::command]
pub fn revoke_mcp_access(app: AppHandle) -> Result<(), String> {
    mcp_auth::revoke_tokens(&app)?;
    hardening::log_event(
        &app,
        "INFO",
        "mcp_auth.revoke",
        "Current MCP OAuth access and refresh credentials revoked.",
    );
    Ok(())
}

#[tauri::command]
pub async fn clear_public_tunnel(app: AppHandle) -> Result<PublicTunnelStatus, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_for_task.state::<AppState>();
        state.clear_public_tunnel(&app_for_task)?;
        hardening::log_event(
            &app_for_task,
            "INFO",
            "public_tunnel.clear",
            "Saved public connection credentials and endpoint identity removed.",
        );
        state.public_tunnel_status(&app_for_task)
    })
    .await
    .map_err(|error| format!("Public connection reset task could not complete: {error}"))?
}

#[tauri::command]
pub fn get_chat_connection_status(
    state: State<'_, AppState>,
) -> Result<ChatConnectionStatus, String> {
    state.chat_connection_status()
}

#[tauri::command]
pub fn start_chat_connection(
    app: AppHandle,
    state: State<'_, AppState>,
    tunnel_id: String,
    api_key: String,
) -> Result<ChatConnectionStatus, String> {
    let result = state.start_chat_connection(app.clone(), tunnel_id, api_key);
    if result.is_ok() {
        hardening::log_event(
            &app,
            "INFO",
            "chat.connect",
            "Secure MCP Tunnel process started.",
        );
    }
    result
}

#[tauri::command]
pub fn stop_chat_connection(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ChatConnectionStatus, String> {
    state.stop_chat_connection()?;
    hardening::log_event(
        &app,
        "INFO",
        "chat.disconnect",
        "Secure MCP Tunnel stopped.",
    );
    state.chat_connection_status()
}

#[tauri::command]
pub async fn get_model_hub(app: AppHandle) -> Result<ModelHubSnapshot, String> {
    model_hub::snapshot(&app).await
}

#[tauri::command]
pub async fn refresh_model_runtime(
    app: AppHandle,
    provider: ModelProviderId,
) -> Result<RuntimeStatus, String> {
    let endpoint = model_hub::configured_endpoint(&app, provider)?;
    Ok(model_hub::discover_provider(provider, endpoint).await)
}

#[tauri::command]
pub fn update_model_runtime_endpoint(
    app: AppHandle,
    provider: ModelProviderId,
    endpoint: String,
) -> Result<String, String> {
    model_hub::update_endpoint(&app, provider, endpoint)
}

#[tauri::command]
pub fn set_selected_local_model(
    app: AppHandle,
    selection: Option<ModelSelection>,
) -> Result<Option<ModelSelection>, String> {
    model_hub::set_selected_model(&app, selection)
}

#[tauri::command]
pub async fn test_local_model(
    app: AppHandle,
    selection: ModelSelection,
) -> Result<ModelTestResult, String> {
    let configured = model_hub::configured_endpoint(&app, selection.provider)?;
    let requested = model_hub::validate_loopback_endpoint(&selection.endpoint)?;
    if requested != configured {
        return Err("This model no longer matches the configured local runtime. Refresh Model Hub and try again.".to_string());
    }
    model_hub::test_model(selection).await
}

#[tauri::command]
pub async fn get_model_trial(app: AppHandle) -> Result<ModelTrialSnapshot, String> {
    model_trial::snapshot(&app).await
}

#[tauri::command]
pub async fn run_model_trial(
    app: AppHandle,
    mode: TrialMode,
    selections: Vec<ModelSelection>,
) -> Result<ModelTrialSnapshot, String> {
    model_trial::run_trial(&app, mode, selections).await
}

#[tauri::command]
pub fn cancel_model_trial(app: AppHandle) -> Result<bool, String> {
    model_trial::cancel_trial(&app)
}

#[tauri::command]
pub fn list_home_conversations(
    app: AppHandle,
    workspace_id: Option<String>,
) -> Result<Vec<conversation::ConversationSummary>, String> {
    if let Some(workspace_id) = workspace_id {
        let _workspace = approved_workspace(&app, &workspace_id)?;
        conversation::list(&app, Some(&workspace_id))
    } else {
        conversation::list(&app, Some(conversation::GENERAL_WORKSPACE_ID))
    }
}

#[tauri::command]
pub fn get_home_conversation(
    app: AppHandle,
    conversation_id: String,
) -> Result<conversation::HomeConversation, String> {
    let conversation = conversation::get(&app, &conversation_id)?;
    if conversation.workspace_id != conversation::GENERAL_WORKSPACE_ID {
        let _workspace = approved_workspace(&app, &conversation.workspace_id)?;
    }
    Ok(conversation)
}

#[tauri::command]
pub fn create_home_conversation(
    app: AppHandle,
    workspace_id: Option<String>,
) -> Result<conversation::HomeConversation, String> {
    let workspace = workspace_id
        .as_deref()
        .map(|workspace_id| approved_workspace(&app, workspace_id))
        .transpose()?;
    conversation::create(&app, workspace.as_ref())
}

#[tauri::command]
pub fn delete_home_conversation(app: AppHandle, conversation_id: String) -> Result<(), String> {
    let existing = conversation::get(&app, &conversation_id)?;
    if existing.workspace_id != conversation::GENERAL_WORKSPACE_ID {
        let _workspace = approved_workspace(&app, &existing.workspace_id)?;
    }
    conversation::delete(&app, &conversation_id)
}

#[tauri::command]
pub fn list_home_context_files(
    app: AppHandle,
    workspace_id: String,
    limit: Option<usize>,
) -> Result<Vec<crate::models::ProjectEntry>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    project_context::safe_attachment_candidates(&workspace, limit.unwrap_or(250))
}

#[tauri::command]
pub async fn begin_home_chat(
    app: AppHandle,
    workspace_id: Option<String>,
    conversation_id: String,
    question: String,
    context: project_context::ProjectContextRequest,
    allow_changes: bool,
) -> Result<conversation::HomeChatStartResult, String> {
    let workspace = workspace_id
        .as_deref()
        .map(|workspace_id| approved_workspace(&app, workspace_id))
        .transpose()?;
    conversation::begin(
        app,
        workspace,
        conversation_id,
        question,
        context,
        allow_changes,
    )
    .await
}

#[tauri::command]
pub fn cancel_home_chat(generation_id: String) -> Result<bool, String> {
    conversation::cancel(&generation_id)
}

#[tauri::command]
pub fn get_runtime_diagnostics(app: AppHandle) -> Result<RuntimeDiagnostics, String> {
    hardening::diagnostics(&app)
}

#[tauri::command]
pub fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<RuntimeDiagnostics, String> {
    hardening::set_launch_at_login(&app, enabled)?;
    hardening::diagnostics(&app)
}

#[tauri::command]
pub fn get_update_status(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<updates::UpdateStatus, String> {
    updates::status(&app, Some(state.inner()))
}

#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    state: State<'_, AppState>,
    manual: bool,
) -> Result<updates::UpdateStatus, String> {
    updates::check(&app, manual, Some(state.inner())).await
}

#[tauri::command]
pub fn set_auto_update_checks(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<updates::UpdateStatus, String> {
    updates::set_auto_check(&app, enabled, Some(state.inner()))
}

#[tauri::command]
pub fn defer_update(
    app: AppHandle,
    state: State<'_, AppState>,
    version: String,
) -> Result<updates::UpdateStatus, String> {
    updates::defer(&app, &version, Some(state.inner()))
}

#[tauri::command]
pub async fn install_update_and_restart(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<updates::UpdateInstallResult, String> {
    updates::install_and_restart(&app, state.inner()).await
}

#[tauri::command]
pub fn list_team_sessions(
    app: AppHandle,
    workspace_id: Option<String>,
) -> Result<Vec<TeamSessionSummary>, String> {
    team::list_sessions(&app, workspace_id.as_deref())
}

#[tauri::command]
pub fn get_team_session(app: AppHandle, session_id: String) -> Result<TeamSnapshot, String> {
    team::get_snapshot(&app, &session_id, None)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_team_session(
    app: AppHandle,
    workspace_id: String,
    goal: String,
    success_criteria: Vec<String>,
    agent_a_name: String,
    agent_a_role: String,
    agent_b_name: String,
    agent_b_role: String,
) -> Result<TeamSnapshot, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let snapshot = team::create_session(
        &app,
        &workspace,
        goal,
        success_criteria,
        agent_a_name,
        agent_a_role,
        agent_b_name,
        agent_b_role,
    )?;
    let _ = activity::record(
        &app,
        &workspace,
        None,
        ActivityKind::Team,
        "teamCreate",
        "Created a two-agent Team Mode session",
        Some(snapshot.session.goal.clone()),
        ActivityStatus::Succeeded,
        Some(snapshot.session.id.clone()),
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn post_team_user_message(
    app: AppHandle,
    session_id: String,
    text: String,
) -> Result<TeamSnapshot, String> {
    let before = team::get_snapshot(&app, &session_id, None)?;
    let workspace = approved_workspace(&app, &before.session.workspace_id)?;
    let snapshot = team::post_user_message(&app, &session_id, text)?;
    let _ = activity::record(
        &app,
        &workspace,
        None,
        ActivityKind::Team,
        "teamUserMessage",
        "Posted guidance to the AI team",
        None,
        ActivityStatus::Succeeded,
        Some(session_id),
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn pause_team_session(app: AppHandle, session_id: String) -> Result<TeamSnapshot, String> {
    let before = team::get_snapshot(&app, &session_id, None)?;
    let workspace = approved_workspace(&app, &before.session.workspace_id)?;
    let snapshot = team::pause_session(&app, &session_id)?;
    let _ = activity::record(
        &app,
        &workspace,
        None,
        ActivityKind::Team,
        "teamPause",
        "Paused AI Team Mode",
        None,
        ActivityStatus::Stopped,
        Some(session_id),
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn resume_team_session(app: AppHandle, session_id: String) -> Result<TeamSnapshot, String> {
    let before = team::get_snapshot(&app, &session_id, None)?;
    let workspace = approved_workspace(&app, &before.session.workspace_id)?;
    let snapshot = team::resume_session(&app, &session_id)?;
    let _ = activity::record(
        &app,
        &workspace,
        None,
        ActivityKind::Team,
        "teamResume",
        "Resumed AI Team Mode",
        None,
        ActivityStatus::Succeeded,
        Some(session_id),
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn cancel_team_session(
    app: AppHandle,
    session_id: String,
    summary: Option<String>,
) -> Result<TeamSnapshot, String> {
    let before = team::get_snapshot(&app, &session_id, None)?;
    let workspace = approved_workspace(&app, &before.session.workspace_id)?;
    let snapshot = team::cancel_session(&app, &session_id, summary)?;
    let _ = activity::record(
        &app,
        &workspace,
        None,
        ActivityKind::Team,
        "teamCancel",
        "Cancelled AI Team Mode",
        snapshot.session.completion_summary.clone(),
        ActivityStatus::Stopped,
        Some(session_id),
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn complete_team_session(
    app: AppHandle,
    session_id: String,
    summary: String,
) -> Result<TeamSnapshot, String> {
    let before = team::get_snapshot(&app, &session_id, None)?;
    let workspace = approved_workspace(&app, &before.session.workspace_id)?;
    let snapshot = team::complete_session(&app, &session_id, None, summary)?;
    let _ = activity::record(
        &app,
        &workspace,
        None,
        ActivityKind::Team,
        "teamEnd",
        "Ended persistent AI Team",
        snapshot.session.completion_summary.clone(),
        ActivityStatus::Succeeded,
        Some(session_id),
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn delete_team_session(app: AppHandle, session_id: String) -> Result<(), String> {
    team::delete_session(&app, &session_id)
}

