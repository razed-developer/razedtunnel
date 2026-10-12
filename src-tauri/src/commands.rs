
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
