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
