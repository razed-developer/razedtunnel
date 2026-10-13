use axum::http::request::Parts;
use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Extension, wrapper::Parameters},
    model::{
        CallToolResult, ContentBlock, ExtensionCapabilities, Implementation,
        ListResourcesResult, MetaObject, PaginatedRequestParams, ReadResourceRequestParams,
        ReadResourceResponse, ReadResourceResult, Resource, ResourceContents, ServerCapabilities,
        ServerInfo,
    },
    schemars,
    service::{NotificationContext, RequestContext},
    tool, tool_handler, tool_router, ErrorData as McpError, RoleServer, ServerHandler,
};
use crate::mcp_git_helpers::compact_git_action_for_ai;
use crate::mcp_observation::{record_observation, monitoring_file_detail};
use crate::mcp_ai_workspace_helpers::{resolve_ai_workspace_app_session, ai_workspace_type_text, ensure_ai_workspace_terminal_text_allowed, ai_workspace_session_payload, require_ai_workspace_session_match};
use crate::mcp_browser_access::{browser_scope_for_request, require_google_tab_access, require_ai_browser_tab_access};
use crate::mcp_continuation_fallback::{ensure_chatgpt_work_fallback, ensure_chatgpt_work_fallback_with_identities};
use crate::mcp_request_identity::{request_edit_group_id, openai_conversation_session, continuation_identity_keys_from, continuation_identity_keys, request_resource_owner_key};
use crate::mcp_browser_params::{BrowserActionParam, BrowserActionParams, BrowserContextParams, BrowserSemanticActionParam, BrowserSemanticActionParams, BrowserSemanticSequenceOperationParam, BrowserSemanticSequenceParams, BrowserSemanticSequenceCancelParams, BrowserInspectParams, BrowserDownloadConfigureParams, BrowserDownloadCancelParams, BrowserUploadParams, BrowserSemanticSnapshotParams, BrowserSemanticFindParams, BrowserScreenshotParams, BrowserDiagnosticsParams};
use crate::mcp_desktop_params::{DesktopInspectParams, DesktopSemanticSnapshotParams, UiSemanticActionParam, DesktopSemanticActionParams, AiWorkspaceSemanticActionParams, UiSemanticSequenceOperationParam, DesktopSemanticSequenceParams, AiWorkspaceSemanticSequenceParams, DesktopActionParams, DesktopScreenshotParams};
use crate::mcp_ai_workspace_params::{AiWorkspaceFrameParams, AiWorkspaceInspectParams, AiWorkspaceSemanticSnapshotParams, AiWorkspaceActionParams, AiWorkspaceSequenceParams};
use crate::mcp_launch_params::{LaunchTargetKindParam, LaunchTargetParams, IntegrationActionParams, AiWorkspaceSessionParams};
use crate::mcp_execution_params::{WorkspaceCommandParams, ListCommandsParams, TerminalCommandParams, ManagedProcessStartParams, ListProcessesParams, WorkspaceRuntimeStatusParams, ProcessIdParams, ProcessOutputParams, ProcessWaitParams, StopProcessParams};
use crate::mcp_filesystem_params::{ProjectSnapshotParams, WorkspacePathParams, SearchFilesParams, FastSearchFilesParams, ReadFileRangeParams, ListDirectoryPageParams, InspectProjectPageParams, FileContentParams, PatchFileParams, CreateDirectoryParams, RenameEntryParams, MoveEntryParams, DeleteEntryParams};
use crate::mcp_continuation_params::{ArmSelfContinuationParams, SelfContinuationModeParam, UpdateSelfContinuationParams, SelfContinuationWatchParams, SelfContinuationAttemptParams, SelfContinuationDeliveryParams};
use crate::mcp_workspace_params::{CloneRepositoryParams, CreateProjectParams, ProjectMemoryUpdateParams, ExternalFileActionParam, ExternalFileAccessParams, WorkspaceIdParams, CreateTempWorkspaceParams, TempWorkspaceParams, PreserveTempWorkspaceParams, CleanupTempWorkspaceParams, TempWorkspaceFileActionParam, TempWorkspaceFileActionParams};
use serde::Serialize;
use std::{
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};
use tauri::{AppHandle, Manager};

use crate::{
    activity, ai_resources,
    app_state::AppState,
    browser, changes, chatgpt_bridge, continuity, desktop_control, environment, execution,
    external_access::{self, ExternalFileAction},
    filesystem, git, github, gmail_access, integrations, large_project_read, launcher,
    media_inspection,
    models::{
        ActivityKind, ActivityStatus, BrowserScreenshot, CommandPolicy,
        GitRepositoryStatus, ManagedProcessRecord, Workspace, WorkspaceAccessMode, WorkspaceChangePolicy,
    },
    monitoring, project_index, project_memory, project_setup, repository, secret_guard,
    self_continuation,
    storage::load_workspaces,
    system_resources, temp_workspace, terminal, workflow,
};


const SELF_CONTINUATION_RESOURCE_URI: &str = "ui://widget/repotunnel-self-continuation-v8.html";
const LEGACY_V7_SELF_CONTINUATION_RESOURCE_URI: &str =
    "ui://widget/repotunnel-self-continuation-v7.html";
const LEGACY_V6_SELF_CONTINUATION_RESOURCE_URI: &str =
    "ui://widget/repotunnel-self-continuation-v6.html";
const LEGACY_V5_SELF_CONTINUATION_RESOURCE_URI: &str =
    "ui://widget/repotunnel-self-continuation-v5.html";
const PREVIOUS_SELF_CONTINUATION_RESOURCE_URI: &str =
    "ui://widget/repotunnel-self-continuation-v4.html";
const LEGACY_SELF_CONTINUATION_RESOURCE_URI: &str = "ui://repotunnel/self-continuation/v3.html";
const LEGACY_V2_SELF_CONTINUATION_RESOURCE_URI: &str = "ui://repotunnel/self-continuation/v2.html";
const LEGACY_V1_SELF_CONTINUATION_RESOURCE_URI: &str = "ui://repotunnel/self-continuation/v1.html";
const SELF_CONTINUATION_APP_HTML: &str = include_str!("../resources/self_continuation_app.html");

static SELF_CONTINUATION_RESOURCE_LIST_COUNT: AtomicU64 = AtomicU64::new(0);
static SELF_CONTINUATION_RESOURCE_READ_COUNT: AtomicU64 = AtomicU64::new(0);
static SELF_CONTINUATION_CURRENT_RESOURCE_READ_COUNT: AtomicU64 = AtomicU64::new(0);
static SELF_CONTINUATION_LAST_RESOURCE_READ_AT: AtomicU64 = AtomicU64::new(0);
static SELF_CONTINUATION_ATTEMPT_COUNT: AtomicU64 = AtomicU64::new(0);
static SELF_CONTINUATION_LAST_ATTEMPT_AT: AtomicU64 = AtomicU64::new(0);
static AI_WORKSPACE_APP_SESSION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn next_ai_workspace_app_session_id() -> String {
    let sequence = AI_WORKSPACE_APP_SESSION_SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1;
    format!(
        "aiwapp-mcp-{}-{}-{sequence}",
        std::process::id(),
        crate::mcp_time::unix_epoch_millis()
    )
}

fn is_self_continuation_resource_uri(uri: &str) -> bool {
    matches!(
        uri,
        SELF_CONTINUATION_RESOURCE_URI
            | LEGACY_V7_SELF_CONTINUATION_RESOURCE_URI
            | LEGACY_V6_SELF_CONTINUATION_RESOURCE_URI
            | LEGACY_V5_SELF_CONTINUATION_RESOURCE_URI
            | PREVIOUS_SELF_CONTINUATION_RESOURCE_URI
            | LEGACY_SELF_CONTINUATION_RESOURCE_URI
            | LEGACY_V2_SELF_CONTINUATION_RESOURCE_URI
            | LEGACY_V1_SELF_CONTINUATION_RESOURCE_URI
    )
}

fn self_continuation_ui_diagnostics() -> serde_json::Value {
    serde_json::json!({
        "resourceUri": SELF_CONTINUATION_RESOURCE_URI,
        "resourceListCount": SELF_CONTINUATION_RESOURCE_LIST_COUNT.load(Ordering::Relaxed),
        "resourceReadCount": SELF_CONTINUATION_RESOURCE_READ_COUNT.load(Ordering::Relaxed),
        "currentResourceReadCount": SELF_CONTINUATION_CURRENT_RESOURCE_READ_COUNT.load(Ordering::Relaxed),
        "lastResourceReadAt": SELF_CONTINUATION_LAST_RESOURCE_READ_AT.load(Ordering::Relaxed),
        "attemptCount": SELF_CONTINUATION_ATTEMPT_COUNT.load(Ordering::Relaxed),
        "lastAttemptAt": SELF_CONTINUATION_LAST_ATTEMPT_AT.load(Ordering::Relaxed),
    })
}

fn with_self_continuation_ui_diagnostics<T: Serialize>(
    value: T,
) -> Result<serde_json::Value, String> {
    let mut value = serde_json::to_value(value)
        .map_err(|error| format!("Could not serialize self-continuation status: {error}"))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Self-continuation status was not a JSON object.".to_string())?;
    object.insert(
        "uiDiagnostics".to_string(),
        self_continuation_ui_diagnostics(),
    );
    Ok(value)
}

fn self_continuation_resource_meta() -> MetaObject {
    let mut meta = MetaObject::new();
    meta.insert(
        "ui".to_string(),
        serde_json::json!({
            "prefersBorder": false,
            "csp": {
                "connectDomains": [],
                "resourceDomains": []
            }
        }),
    );
    // Keep ChatGPT compatibility aliases while the standards-first MCP Apps
    // metadata above is the authoritative path.
    meta.insert(
        "openai/widgetDescription".to_string(),
        serde_json::json!(
            "Background RepoTunnel recovery bridge. It stays visually hidden and submits only a queued continuation after work becomes stale."
        ),
    );
    meta.insert(
        "openai/widgetPrefersBorder".to_string(),
        serde_json::json!(false),
    );
    meta.insert(
        "openai/widgetCSP".to_string(),
        serde_json::json!({
            "connect_domains": [],
            "resource_domains": []
        }),
    );
    meta
}

const SERVER_INSTRUCTIONS: &str = include_str!("../resources/mcp_server_instructions.txt");

pub(crate) use crate::mcp_task_metrics::task_tool_outcomes;
use crate::mcp_task_metrics::record_task_outcome;

#[derive(Clone)]
pub(crate) struct RepoTunnelMcp {
    app: AppHandle,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceSummary {
    id: String,
    name: String,
    access_mode: WorkspaceAccessMode,
    change_policy: WorkspaceChangePolicy,
    command_policy: CommandPolicy,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RepoTunnelCapabilities {
    version: String,
    platform: String,
    workspace_runtime: WorkspaceRuntimeCapabilities,
    browser_runtime: BrowserRuntimeCapabilities,
    semantic_interaction: SemanticInteractionCapabilities,
    continuity: ContinuityCapabilities,
    desktop: DesktopCapabilities,
    generic_middleware: GenericMiddlewareCapabilities,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceRuntimeCapabilities {
    lightweight_runtime_status: bool,
    managed_jobs: bool,
    bounded_output_tail: bool,
    cancel_jobs: bool,
    restart_reattachment: bool,
    persistent_cargo_cache: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserRuntimeCapabilities {
    persistent_workspace_profile: bool,
    shared_authenticated_profile: bool,
    google_sign_in_permission: bool,
    ai_session_tab_isolation: bool,
    separate_ai_windows: bool,
    persistent_non_secret_context_headers: bool,
    user_agent_override: bool,
    atomic_navigation_receipt: bool,
    mutation_receipts: bool,
    navigation_generation: bool,
    successful_network_history: bool,
    response_body_capture: bool,
    websocket_frame_capture: bool,
    scope_allowlist: bool,
    raw_secret_header_persistence: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SemanticInteractionCapabilities {
    browser_accessibility: bool,
    linux_at_spi: bool,
    windows_uia: bool,
    macos_ax: bool,
    short_lived_refs: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContinuityCapabilities {
    resume_v2: bool,
    factual_activity_history: bool,
    semantic_project_memory: bool,
    mcp_app_self_continuation: bool,
    assistant_generation_signal: bool,
    automatic_chat_session_end_detection: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopCapabilities {
    ai_workspace: bool,
    real_desktop_control: bool,
    repotunnel_self_control_blocked: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenericMiddlewareCapabilities {
    resource_snapshot: bool,
    capability_oriented_tool_discovery: bool,
    ai_owned_resource_cleanup: bool,
    owned_temp_workspaces: bool,
    safe_temp_cleanup: bool,
    browser_download_tracking: bool,
    browser_file_upload: bool,
    generic_media_inspection: bool,
    media_frame_extraction: bool,
    media_decode_validation: bool,
    automatic_large_install: bool,
    bundled_asset_library: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceRuntimeStatus {
    workspace_id: String,
    git: GitRepositoryStatus,
    processes: Vec<ManagedProcessRecord>,
    running_processes: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GitDiffView {
    mode: GitDiffOutputModeParam,
    stats: crate::models::GitDiffStats,
    content: Option<String>,
    content_truncated: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ListActivityParams {
    /// Optional workspace ID. Omit to list recent records across approved projects.
    workspace_id: Option<String>,
    /// Maximum records to return. RepoTunnel clamps this to 1..100.
    limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct MediaInspectParams {
    /// ID returned by list_workspaces for the approved project.
    workspace_id: String,
    /// Existing workspace-relative image/audio/video/subtitle path, including a file inside .repotunnel-tmp.
    relative_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct MediaFrameParams {
    workspace_id: String,
    relative_path: String,
    /// Existing RepoTunnel temporary task; the extracted PNG is written under frames/.
    task_id: String,
    /// Video timestamp in seconds. Must be finite and >= 0.
    timestamp_seconds: f64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct MediaDecodeParams {
    workspace_id: String,
    relative_path: String,
    /// Optional bounded decode duration in seconds. RepoTunnel caps this at 600 seconds. Defaults to 30 seconds when full_decode is not true.
    check_seconds: Option<f64>,
    /// Explicitly request decoding the entire file. Defaults to false because full decode can be expensive on low-end hardware.
    full_decode: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct WorkspaceMonitoringParams {
    /// ID returned by list_workspaces for the approved project.
    workspace_id: String,
    /// True to persistently enable project-file monitoring, false to disable it. Monitoring is observational and does not edit project files.
    enabled: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct MonitoringFileEventsParams {
    /// Optional workspace ID. Omit to list recent monitoring file events across approved projects.
    workspace_id: Option<String>,
    /// Maximum events to return. RepoTunnel clamps this to 1..200.
    limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ListChangesParams {
    /// Optional workspace ID. Omit to list recent changes across approved projects.
    workspace_id: Option<String>,
    /// Maximum records to return. RepoTunnel clamps this to 1..100.
    limit: Option<usize>,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
enum GitDiffOutputModeParam {
    Compact,
    Summary,
    Full,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GitDiffParams {
    /// ID returned by list_workspaces for the approved Git repository.
    workspace_id: String,
    /// True for the staged/index diff; false for unstaged working-tree changes.
    staged: bool,
    /// compact = counts only; summary = counts plus up to 100 changed paths; full = summary plus bounded patch text. Defaults to summary.
    mode: Option<GitDiffOutputModeParam>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GitLogParams {
    /// ID returned by list_workspaces for the approved Git repository.
    workspace_id: String,
    /// Maximum commits to return. RepoTunnel clamps this to 1..50.
    limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GitStageParams {
    /// ID returned by list_workspaces for the approved Git repository.
    workspace_id: String,
    /// Exact workspace-relative file paths to stage. RepoTunnel accepts 1..100 paths.
    paths: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GitCommitParams {
    /// ID returned by list_workspaces for the approved Git repository.
    workspace_id: String,
    /// Commit message. RepoTunnel commits currently staged changes only.
    message: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GitPushParams {
    /// ID returned by list_workspaces for the approved Git repository.
    workspace_id: String,
    /// Git remote name. Defaults to origin.
    remote: Option<String>,
    /// Branch to push. Defaults to the current branch.
    branch: Option<String>,
    /// Set the upstream tracking branch. Defaults true.
    set_upstream: Option<bool>,
    /// Use --force-with-lease instead of an ordinary push. Defaults false.
    force_with_lease: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GithubCreatePrParams {
    /// ID returned by list_workspaces for the approved Git repository.
    workspace_id: String,
    /// Pull request title.
    title: String,
    /// Pull request body. Defaults to empty.
    body: Option<String>,
    /// Base branch. Omit to use the repository default.
    base: Option<String>,
    /// Head branch. Omit to use the current branch.
    head: Option<String>,
    /// Create as a draft pull request.
    draft: Option<bool>,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
enum GithubMergeMethodParam {
    Merge,
    Squash,
    Rebase,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GithubMergePrParams {
    /// ID returned by list_workspaces for the approved Git repository.
    workspace_id: String,
    /// Pull request number.
    number: u64,
    /// Merge strategy. Defaults to merge.
    method: Option<GithubMergeMethodParam>,
    /// Delete the branch after merge.
    delete_branch: Option<bool>,
    /// Enable GitHub auto-merge when branch protection requires it.
    auto: Option<bool>,
    /// Use maintainer/admin merge privileges when available.
    admin: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct QueueChatGptBridgeMessageParams {
    /// ID returned by list_workspaces for the workspace this continuation belongs to.
    workspace_id: String,
    /// Connected target ID returned by list_chatgpt_extension_targets. Required once to establish a new conversation binding; after that it may be omitted.
    target_id: Option<String>,
    /// Exact 1–2 sentence, current-project-only next-work message the extension should submit to ChatGPT after the target tab becomes idle.
    message: String,
    /// Optional delay before the extension may claim the message. Defaults to zero and is clamped to one hour.
    delay_seconds: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ChatGptBridgeWorkspaceParams {
    /// ID returned by list_workspaces for the workspace whose continuation state should be inspected or closed.
    workspace_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ChatGptBridgeJobParams {
    /// ID returned by list_workspaces for the workspace that owns this continuation job.
    workspace_id: String,
    /// Job ID returned by queue_chatgpt_extension_message or list_chatgpt_extension_jobs.
    job_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct CompleteChatGptBridgeWorkParams {
    /// ID returned by list_workspaces for the workspace whose current continuation guard should be closed.
    workspace_id: String,
    /// Set true when the AI is intentionally stopping because human input is required; false/omit when the requested work is complete.
    waiting_for_user: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct GitRestoreParams {
    /// ID returned by list_workspaces for the approved Git repository.
    workspace_id: String,
    /// Tracked UTF-8 text file path relative to the workspace root.
    relative_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ListGitActionsParams {
    /// Optional workspace ID. Omit to list recent Git action requests across approved projects.
    workspace_id: Option<String>,
    /// Maximum records to return. RepoTunnel clamps this to 1..100.
    limit: Option<usize>,
}

fn ensure_ai_access(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    if state.ai_access_paused() {
        return Err("AI access is paused in RepoTunnel. The user must resume AI access locally before MCP tools can access project information or perform project actions.".to_string());
    }
    Ok(())
}

pub(crate) fn approved_workspace(app: &AppHandle, workspace_id: &str) -> Result<Workspace, String> {
    ensure_ai_access(app)?;

    load_workspaces(app)?
        .into_iter()
        .find(|workspace| workspace.id == workspace_id)
        .ok_or_else(|| "That project is not approved in RepoTunnel.".to_string())
}

fn serialize_success_payload<T: Serialize>(value: T) -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct SuccessEnvelope<T> {
        ok: bool,
        result: T,
    }

    serde_json::to_string(&SuccessEnvelope { ok: true, result: value })
}

fn success_result<T: Serialize>(value: T) -> CallToolResult {
    match serialize_success_payload(value) {
        Ok(content) => CallToolResult::success(vec![ContentBlock::text(content)]),
        Err(error) => CallToolResult::error(vec![ContentBlock::text(format!(
            "RepoTunnel could not serialize the tool result: {error}"
        ))]),
    }
}

fn error_result(message: impl Into<String>) -> CallToolResult {
    let message = message.into();
    let content = serde_json::to_string(&serde_json::json!({
        "ok": false,
        "error": message,
    }))
    .unwrap_or_else(|_| "{\"ok\":false,\"error\":\"RepoTunnel operation failed.\"}".to_string());

    CallToolResult::error(vec![ContentBlock::text(content)])
}

async fn run_filesystem_task<T, F>(task: F) -> Result<CallToolResult, McpError>
where
    T: Serialize + Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    Ok(match tokio::task::spawn_blocking(task).await {
        Ok(Ok(value)) => {
            // Serialization can fail even when the filesystem operation succeeded.
            // Count the actual MCP result, not just the task closure outcome.
            match serialize_success_payload(value) {
                Ok(content) => {
                    record_task_outcome(true);
                    CallToolResult::success(vec![ContentBlock::text(content)])
                }
                Err(error) => {
                    record_task_outcome(false);
                    error_result(format!("RepoTunnel could not serialize the tool result: {error}"))
                }
            }
        },
        Ok(Err(message)) => {
            record_task_outcome(false);
            error_result(message)
        },
        Err(error) => {
            record_task_outcome(false);
            error_result(format!("The local filesystem task could not complete: {error}"))
        },
    })
}

async fn run_structured_task<T, F>(task: F) -> Result<CallToolResult, McpError>
where
    T: Serialize + Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    Ok(match tokio::task::spawn_blocking(task).await {
        Ok(Ok(value)) => match serde_json::to_value(value) {
            Ok(value) => {
                record_task_outcome(true);
                CallToolResult::structured(value)
            },
            Err(error) => {
                record_task_outcome(false);
                error_result(format!("RepoTunnel could not serialize the structured tool result: {error}"))
            },
        },
        Ok(Err(message)) => {
            record_task_outcome(false);
            error_result(message)
        },
        Err(error) => {
            record_task_outcome(false);
            error_result(format!("The local structured task could not complete: {error}"))
        },
    })
}

async fn run_browser_screenshot_task<F>(task: F) -> Result<CallToolResult, McpError>
where
    F: FnOnce() -> Result<BrowserScreenshot, String> + Send + 'static,
{
    const MAX_MCP_SCREENSHOT_BYTES: u64 = 8 * 1024 * 1024;
    Ok(match tokio::task::spawn_blocking(task).await {
        Ok(Ok(screenshot)) if screenshot.size_bytes <= MAX_MCP_SCREENSHOT_BYTES => {
            let metadata = serde_json::json!({
                "ok": true,
                "result": {
                    "id": screenshot.id,
                    "tabId": screenshot.tab_id,
                    "createdAt": screenshot.created_at,
                    "mimeType": screenshot.mime_type.clone(),
                    "sizeBytes": screenshot.size_bytes,
                    "fullPage": screenshot.full_page,
                }
            });
            let text = serde_json::to_string(&metadata)
                .unwrap_or_else(|_| "{\"ok\":true}".to_string());
            CallToolResult::success(vec![
                ContentBlock::text(text),
                ContentBlock::image(screenshot.data_base64, screenshot.mime_type),
            ])
        }
        Ok(Ok(screenshot)) => error_result(format!(
            "The screenshot is {} bytes, which exceeds RepoTunnel's 8 MiB MCP image limit. Retry with full_page=false or inspect the page DOM instead.",
            screenshot.size_bytes
        )),
        Ok(Err(message)) => error_result(message),
        Err(error) => error_result(format!("The browser screenshot task could not complete: {error}")),
    })
}

fn required_text(value: Option<String>, field: &str, action: &str) -> Result<String, String> {
    let value = value.unwrap_or_default();
    if value.trim().is_empty() {
        Err(format!("{field} is required for browser action {action}."))
    } else {
        Ok(value)
    }
}

impl RepoTunnelMcp {
    pub(crate) fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

#[tool_router(router = base_tool_router)]
impl RepoTunnelMcp {
    #[tool(
        description = "Create a new empty local project only when the human explicitly asks to create a project from scratch. RepoTunnel creates a new folder inside ~/Projects, refuses to overwrite an existing folder, and immediately registers it as an approved workspace so normal file tools can build the project from chat."
    )]
    async fn create_project(
        &self,
        Parameters(params): Parameters<CreateProjectParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            repository::create_and_register(&app, &params.name)
        })
        .await
    }

    #[tool(
        description = "Detect the approved project's framework, package manager, dependency readiness, safe setup command when preparation is needed, and likely dev command/URL. Use this instead of asking the human which package manager or dev command a project uses.",
        annotations(read_only_hint = true)
    )]
    async fn get_project_setup(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            project_setup::detect(&workspace)
        })
        .await
    }

    #[tool(
        description = "Read RepoTunnel's persistent project memory for an approved project. It contains concise project context, goals, important decisions, user preferences/constraints, and next steps stored outside the repository so a later AI session can resume without rediscovering everything.",
        annotations(read_only_hint = true)
    )]
    async fn get_project_memory(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            project_memory::get(&app, &workspace)
        })
        .await
    }

    #[tool(
        description = "Get RepoTunnel Continuity Resume v2 after a ChatGPT turn limit, connector reconnect, app restart, or other interruption. It returns a small authoritative brief built from live Git/activity/process state, a bounded semantic-context preview, and compact durable milestones. Factual state always wins over stale saved memory; use get_project_memory only when deeper context is actually needed.",
        annotations(read_only_hint = true)
    )]
    async fn get_resume_snapshot(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            continuity::resume_snapshot(&app, &workspace)
        })
        .await
    }

    #[tool(
        description = "Update RepoTunnel's persistent project memory for the approved project after meaningful decisions/progress. Keep it concise and factual; do not store secrets, credentials, raw logs, or temporary chatter. This memory lives in RepoTunnel app data, not in the user's repository.",
        annotations(
            read_only_hint = false,
            destructive_hint = true,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn update_project_memory(
        &self,
        Parameters(params): Parameters<ProjectMemoryUpdateParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            project_memory::update(
                &app,
                &workspace,
                params.summary,
                params.goals,
                params.decisions,
                params.preferences,
                params.next_steps,
            )
        })
        .await
    }

    #[tool(
        description = "Clone a GitHub repository explicitly supplied by the human and register the checkout as an approved RepoTunnel workspace. Accepts owner/repository or an HTTPS github.com repository URL. Clones into ~/Projects using the machine's existing Git/GitHub authentication, never stores credentials, never overwrites an existing unrelated folder, and works for both normal single-AI use and Team Mode bootstrap."
    )]
    async fn clone_repository(
        &self,
        Parameters(params): Parameters<CloneRepositoryParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            repository::clone_and_register(&app, &params.repository)
        })
        .await
    }

    #[tool(
        description = "Request access to one file outside the approved workspace without exposing the rest of the host filesystem. RepoTunnel always opens a native local file picker: the user must explicitly select the file, and cancelling denies access. action=read returns bounded UTF-8 content once without revealing the absolute host path. action=import copies the selected regular file to an explicit workspace-relative destination after secret scanning; the AI can then work on it using normal workspace tools. Sensitive credential files and symlinks remain blocked. Works in normal mode."
    )]
    async fn request_external_file(
        &self,
        Parameters(params): Parameters<ExternalFileAccessParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let action = match params.action {
                ExternalFileActionParam::Read => ExternalFileAction::Read,
                ExternalFileActionParam::Import => {
                    params.destination_path.as_deref().ok_or_else(|| {
                        "destination_path is required for action=import.".to_string()
                    })?;
                    ExternalFileAction::Import
                }
            };
            let result = external_access::request_file(
                &app,
                &workspace,
                action,
                params.reason.as_deref(),
                params.destination_path.as_deref(),
            )?;
            let status = if result.approved {
                ActivityStatus::Succeeded
            } else {
                ActivityStatus::Rejected
            };
            let summary = if result.approved {
                match action {
                    ExternalFileAction::Read => {
                        "User approved one-time external file reading".to_string()
                    }
                    ExternalFileAction::Import => format!(
                        "User approved external file import to {}",
                        result.imported_path.as_deref().unwrap_or("project")
                    ),
                }
            } else {
                "User cancelled external file access".to_string()
            };
            let _ = activity::record(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Files,
                "externalFileAccess",
                summary,
                result.source_name.clone(),
                status,
                None,
            );
            Ok(result)
        })
        .await
    }

    #[tool(
        description = "List the local projects the user has explicitly approved in RepoTunnel. Use this first when you need a workspace ID. Returns project names, IDs, access modes, write policies, and command policies, but does not expose absolute local paths.",
        annotations(read_only_hint = true)
    )]
    async fn list_workspaces(&self) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let summaries = load_workspaces(&app)?
                .into_iter()
                .map(|workspace| WorkspaceSummary {
                    id: workspace.id,
                    name: workspace.name,
                    access_mode: workspace.access_mode,
                    change_policy: workspace.change_policy,
                    command_policy: workspace.command_policy,
                })
                .collect::<Vec<_>>();
            Ok(summaries)
        })
        .await
    }

    #[tool(
        description = "Describe RepoTunnel's current core runtime capabilities and important limitations so an AI can choose supported workflows without inspecting internal tool metadata. This is a compact product/runtime capability summary, not permission to act.",
        annotations(read_only_hint = true)
    )]
    async fn capabilities(&self) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            Ok(RepoTunnelCapabilities {
                version: env!("CARGO_PKG_VERSION").to_string(),
                platform: std::env::consts::OS.to_string(),
                workspace_runtime: WorkspaceRuntimeCapabilities {
                    lightweight_runtime_status: true,
                    managed_jobs: true,
                    bounded_output_tail: true,
                    cancel_jobs: true,
                    restart_reattachment: terminal::restart_reattachment_supported(),
                    persistent_cargo_cache: true,
                },
                browser_runtime: BrowserRuntimeCapabilities {
                    persistent_workspace_profile: true,
                    shared_authenticated_profile: true,
                    google_sign_in_permission: true,
                    ai_session_tab_isolation: true,
                    separate_ai_windows: true,
                    persistent_non_secret_context_headers: true,
                    user_agent_override: true,
                    atomic_navigation_receipt: true,
                    mutation_receipts: true,
                    navigation_generation: true,
                    successful_network_history: true,
                    response_body_capture: false,
                    websocket_frame_capture: false,
                    scope_allowlist: false,
                    raw_secret_header_persistence: false,
                },
                semantic_interaction: SemanticInteractionCapabilities {
                    browser_accessibility: true,
                    linux_at_spi: cfg!(target_os = "linux"),
                    windows_uia: cfg!(target_os = "windows"),
                    macos_ax: cfg!(target_os = "macos"),
                    short_lived_refs: true,
                },
                continuity: ContinuityCapabilities {
                    resume_v2: true,
                    factual_activity_history: true,
                    semantic_project_memory: true,
                    mcp_app_self_continuation: true,
                    assistant_generation_signal: false,
                    automatic_chat_session_end_detection: false,
                },
                desktop: DesktopCapabilities {
                    ai_workspace: cfg!(target_os = "linux"),
                    real_desktop_control: true,
                    repotunnel_self_control_blocked: true,
                },
                generic_middleware: GenericMiddlewareCapabilities {
                    resource_snapshot: true,
                    capability_oriented_tool_discovery: true,
                    ai_owned_resource_cleanup: true,
                    owned_temp_workspaces: true,
                    safe_temp_cleanup: true,
                    browser_download_tracking: true,
                    browser_file_upload: true,
                    generic_media_inspection: true,
                    media_frame_extraction: true,
                    media_decode_validation: true,
                    automatic_large_install: false,
                    bundled_asset_library: false,
                },
            })
        })
        .await
    }

    #[tool(
        description = "List ChatGPT conversation tabs currently connected through the RepoTunnel Chrome extension bridge. Targets are registered only while the exact saved ChatGPT tab is open and the extension is alive. Use this before queue_chatgpt_extension_message when more than one target is connected.",
        annotations(read_only_hint = true)
    )]
    async fn list_chatgpt_extension_targets(&self) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_structured_task(move || {
            ensure_ai_access(&app)?;
            chatgpt_bridge::list_targets()
        })
        .await
    }

    #[tool(
        description = "Replace the one pending continuation checkpoint for this exact ChatGPT conversation and workspace with a precise 1–2 sentence AI-written next action about only the current project. Update this short checkpoint as work changes; do not write a long progress/status essay. workspace_id is required. A new conversation must establish its binding once with an explicit target_id returned by list_chatgpt_extension_targets; RepoTunnel never guesses from a sole connected tab or from another MCP session. After that exact conversation binding exists, target_id may be omitted. Checkpoints are workspace-scoped and cannot replace a pending checkpoint owned by another workspace. The extension claims only after the exact saved ChatGPT tab is genuinely idle and its composer is empty, then submits this exact message and ACKs delivery. Never use a generic 'continue'.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn queue_chatgpt_extension_message(
        &self,
        Parameters(params): Parameters<QueueChatGptBridgeMessageParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let continuation_identities = continuation_identity_keys(&parts, &context);
        run_structured_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            chatgpt_bridge::queue_message_for_identities(
                &workspace.id,
                &continuation_identities,
                params.target_id,
                params.message,
                params.delay_seconds,
            )
        })
        .await
    }

    #[tool(
        description = "Begin or refresh the RepoTunnel ChatGPT continuation guard for substantial work in this exact conversation and workspace, including long read-only inspection/research. Call this once after an exact continuation target binding exists and again after a reconnect/resume when work will continue. It is idempotent and refreshes the crash-recovery grace without changing an exact checkpoint.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn begin_chatgpt_extension_work(
        &self,
        Parameters(params): Parameters<ChatGptBridgeWorkspaceParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let continuation_identities =
            continuation_identity_keys_from(openai_conversation_session(&context));
        run_structured_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            chatgpt_bridge::ensure_automatic_fallback_for_identities(
                &workspace.id,
                &continuation_identities,
            )?
            .ok_or_else(|| {
                "This ChatGPT conversation has no exact extension target binding yet. List targets and queue one explicit checkpoint with target_id first."
                    .to_string()
            })
        })
        .await
    }

    #[tool(
        description = "List recent RepoTunnel ChatGPT extension delivery jobs only for this exact ChatGPT conversation and workspace. Includes pending/claimed/sending/delivered/failed/cancelled/uncertain state; claim tokens are never returned.",
        annotations(read_only_hint = true)
    )]
    async fn list_chatgpt_extension_jobs(
        &self,
        Parameters(params): Parameters<ChatGptBridgeWorkspaceParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let continuation_identities =
            continuation_identity_keys_from(openai_conversation_session(&context));
        run_structured_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            chatgpt_bridge::list_jobs_for_identities(&workspace.id, &continuation_identities)
        })
        .await
    }

    #[tool(
        description = "Cancel one specific ChatGPT continuation checkpoint only when it belongs to this exact ChatGPT conversation and workspace. Use complete_chatgpt_extension_work instead when the whole current work request is complete or intentionally waiting for the human.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn cancel_chatgpt_extension_job(
        &self,
        Parameters(params): Parameters<ChatGptBridgeJobParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let continuation_identities =
            continuation_identity_keys_from(openai_conversation_session(&context));
        run_structured_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            chatgpt_bridge::cancel_job_for_identities(
                &workspace.id,
                &continuation_identities,
                &params.job_id,
            )
        })
        .await
    }

    #[tool(
        description = "Close the current RepoTunnel ChatGPT continuation guard for this exact conversation and workspace. MUST be called immediately before a normal final response when the requested work is complete, or when intentionally stopping for human input. This atomically cancels any still-active automatic or exact checkpoint so finished work cannot wake the chat again.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn complete_chatgpt_extension_work(
        &self,
        Parameters(params): Parameters<CompleteChatGptBridgeWorkParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let continuation_identities =
            continuation_identity_keys_from(openai_conversation_session(&context));
        run_structured_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let reason = if params.waiting_for_user.unwrap_or(false) {
                "Continuation work closed because human input is required."
            } else {
                "Continuation work completed normally."
            };
            chatgpt_bridge::complete_work_for_identities(
                &workspace.id,
                &continuation_identities,
                reason,
            )
        })
        .await
    }

    #[tool(
        description = "Arm RepoTunnel AI self-continuation for substantial multi-step work. Provide exactly one specific AI-written continuation sentence describing the remaining work and what is already complete; do not use a generic 'continue' message. RepoTunnel does not assume a fixed ChatGPT session length: recovery is queued only after confirmed work activity stops and remains idle for the fixed two-minute recovery grace. This tool deliberately renders no UI; immediately call mount_self_continuation_app exactly once with the returned watch_id.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn arm_self_continuation(
        &self,
        Parameters(params): Parameters<ArmSelfContinuationParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        let result = run_structured_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let watch = self_continuation::arm(
                &app,
                &workspace.id,
                conversation_session.as_deref(),
                &params.continuation_sentence,
            )?;
            with_self_continuation_ui_diagnostics(self_continuation::status_from_watch(&watch))
        })
        .await?;

        Ok(result)
    }

    #[tool(
        description = "Render the tiny RepoTunnel self-continuation MCP App for an already-armed watch. Call this exactly once immediately after arm_self_continuation. This read-only render step is deliberately separate from the mutating arm operation so ChatGPT can fetch the UI template without coupling template loading to an approval/state-changing tool call.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn mount_self_continuation_app(
        &self,
        Parameters(params): Parameters<SelfContinuationWatchParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        let result = run_structured_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let status = self_continuation::inspect(
                &app,
                &workspace.id,
                conversation_session.as_deref(),
                &params.watch_id,
            )?;
            with_self_continuation_ui_diagnostics(status)
        })
        .await?;

        Ok(result.with_meta(Some(tool_meta(
            Some(SELF_CONTINUATION_RESOURCE_URI),
            &["model", "app"],
        ))))
    }

    #[tool(
        description = "Replace the current AI-written self-continuation sentence or change recovery state. Use working while actively progressing, waiting_user before intentionally waiting for human input, completed before intentional task completion, and disabled to turn the watch off. When meaningful remaining work changes, replace the sentence instead of accumulating multiple notes.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn update_self_continuation(
        &self,
        Parameters(params): Parameters<UpdateSelfContinuationParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_structured_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let mode = match params.state {
                SelfContinuationModeParam::Working => self_continuation::ContinuationMode::Working,
                SelfContinuationModeParam::WaitingUser => {
                    self_continuation::ContinuationMode::WaitingUser
                }
                SelfContinuationModeParam::Completed => {
                    self_continuation::ContinuationMode::Completed
                }
                SelfContinuationModeParam::Disabled => {
                    self_continuation::ContinuationMode::Disabled
                }
            };
            let watch = self_continuation::update(
                &app,
                &workspace.id,
                conversation_session.as_deref(),
                &params.watch_id,
                mode,
                params.continuation_sentence.as_deref(),
            )?;
            Ok(self_continuation::status_from_watch(&watch))
        })
        .await
    }

    #[tool(
        description = "Refresh an armed self-continuation watch while meaningful AI work is still actively progressing but no new continuation sentence is needed. When an armed task is actively reasoning for a while without producing RepoTunnel project activity, heartbeat at least once per minute. Active managed processes are suppressed automatically. Do not heartbeat after the work is complete; mark the watch completed instead.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn heartbeat_self_continuation(
        &self,
        Parameters(params): Parameters<SelfContinuationWatchParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_structured_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let watch = self_continuation::heartbeat(
                &app,
                &workspace.id,
                conversation_session.as_deref(),
                &params.watch_id,
            )?;
            Ok(self_continuation::status_from_watch(&watch))
        })
        .await
    }

    #[tool(
        description = "MCP App-only deadline recovery attempt. Evaluate the durable self-continuation watch once, account for current project activity and active managed processes, and atomically claim a pending recovery when delivery is eligible. The v8 widget calls this only at the server-provided next-check deadline rather than polling repeatedly.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn attempt_self_continuation_recovery(
        &self,
        Parameters(params): Parameters<SelfContinuationAttemptParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        SELF_CONTINUATION_ATTEMPT_COUNT.fetch_add(1, Ordering::Relaxed);
        SELF_CONTINUATION_LAST_ATTEMPT_AT.store(crate::mcp_time::unix_epoch_millis(), Ordering::Relaxed);
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_structured_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            self_continuation::attempt(
                &app,
                &workspace.id,
                conversation_session.as_deref(),
                &params.watch_id,
                &params.claimant_id,
            )
        })
        .await
    }

    #[tool(
        description = "Legacy MCP App-only self-continuation poll retained only so previously mounted polling widgets can shut themselves down cleanly after a current watch is armed. New widgets use attempt_self_continuation_recovery and do not poll.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn poll_self_continuation(
        &self,
        Parameters(params): Parameters<SelfContinuationWatchParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_structured_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let status = self_continuation::poll(
                &app,
                &workspace.id,
                conversation_session.as_deref(),
                &params.watch_id,
            )?;
            with_self_continuation_ui_diagnostics(status)
        })
        .await
    }

    #[tool(
        description = "MCP App-only delivery lease for a pending self-continuation recovery. Only one mounted widget may claim a recovery at a time; the short lease expires automatically so failed/offline delivery remains retryable.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn claim_self_continuation_recovery(
        &self,
        Parameters(params): Parameters<SelfContinuationDeliveryParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_structured_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            self_continuation::claim(
                &app,
                &workspace.id,
                conversation_session.as_deref(),
                &params.watch_id,
                &params.recovery_id,
                &params.claimant_id,
            )
        })
        .await
    }

    #[tool(
        description = "MCP App-only idempotent acknowledgement for a self-continuation recovery that was successfully submitted to ChatGPT. The acknowledgement must come from the widget instance that claimed the delivery lease.",
        annotations(
            read_only_hint = false,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn ack_self_continuation_recovery(
        &self,
        Parameters(params): Parameters<SelfContinuationDeliveryParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_structured_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            self_continuation::ack(
                &app,
                &workspace.id,
                conversation_session.as_deref(),
                &params.watch_id,
                &params.recovery_id,
                &params.claimant_id,
            )
        })
        .await
    }

    #[tool(
        description = "Inspect an approved codebase using RepoTunnel's smart project index. Returns a filtered project tree plus file counts, detected languages, common manifests, binary/large-file counts, and ignore statistics. Respects .gitignore/.ignore rules and skips generated dependency/build folders.",
        annotations(read_only_hint = true)
    )]
    async fn inspect_project(
        &self,
        Parameters(params): Parameters<ProjectSnapshotParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let snapshot =
                project_index::project_snapshot(&workspace, params.entry_limit.unwrap_or(600))?;
            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Files,
                "inspectProject",
                format!(
                    "Inspected project · {} indexed entries",
                    snapshot.entries.len()
                ),
                None,
            );
            Ok(snapshot)
        })
        .await
    }

    #[tool(
        description = "Inspect a large project tree in bounded resumable pages. Returns up to pageSize accessible file/folder entries plus nextCursor and never requires rebuilding the already-scanned portion of the same session. Respects RepoTunnel's existing ignore/generated-folder and protected-path rules. Prefer this over inspect_project when a large repository risks a long full-tree scan.",
        annotations(read_only_hint = true)
    )]
    async fn inspect_project_page(
        &self,
        Parameters(params): Parameters<InspectProjectPageParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let result = large_project_read::inspect_project_page(
                &workspace,
                &params.relative_path,
                params.cursor.as_deref(),
                params.page_size,
            )?;
            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Files,
                "inspectProjectPage",
                format!(
                    "Inspected {} · {} entries this page",
                    if params.relative_path.is_empty() {
                        "."
                    } else {
                        &params.relative_path
                    },
                    result.entries.len()
                ),
                Some(format!(
                    "{} scanned · {}ms · {}",
                    result.scanned_entries,
                    result.elapsed_ms,
                    if result.done {
                        "done"
                    } else {
                        "continuation available"
                    }
                )),
            );
            Ok(result)
        })
        .await
    }

    #[tool(
        description = "Preflight the complete AI development workflow for an approved project. Reports whether project inspection, safe editing, sandboxed verification, and Git completion are currently available and explains any limitations. Call this before starting a multi-step bug fix or feature task.",
        annotations(read_only_hint = true)
    )]
    async fn get_workflow_readiness(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            Ok(workflow::readiness(&workspace))
        })
        .await
    }

    #[tool(
        description = "List relevant files and folders at a workspace-relative path. RepoTunnel omits protected secrets, ignored entries, and generated dependency/build folders from directory discovery.",
        annotations(read_only_hint = true)
    )]
    async fn list_directory(
        &self,
        Parameters(params): Parameters<WorkspacePathParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let entries = filesystem::list_directory(&workspace, &params.relative_path)?;
            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Files,
                "listDirectory",
                format!(
                    "Listed {} · {} entries",
                    if params.relative_path.is_empty() {
                        "."
                    } else {
                        &params.relative_path
                    },
                    entries.len()
                ),
                None,
            );
            Ok(entries)
        })
        .await
    }

    #[tool(
        description = "Read an existing UTF-8 text file inside an approved workspace. Use this before editing a file so changes are based on current content.",
        annotations(read_only_hint = true)
    )]
    async fn read_file(
        &self,
        Parameters(params): Parameters<WorkspacePathParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let file = filesystem::read_file(&workspace, &params.relative_path)?;
            if let Some(kind) = secret_guard::detect_secret(file.content.as_bytes()) {
                return Err(format!(
                    "RepoTunnel withheld '{}' because its text appears to contain {kind}. Secrets are never returned to an AI through MCP; remove or replace the credential before asking the AI to edit this file.",
                    params.relative_path
                ));
            }
            record_observation(
                &app, &workspace, trace_group_id.as_deref(), ActivityKind::Files, "readFile",
                format!("Read {}", params.relative_path), Some(format!("{} bytes", file.size)),
            );
            Ok(file)
        })
        .await
    }

    #[tool(
        description = "Search accessible UTF-8 project text for a case-insensitive query. Returns bounded path/line/column previews plus filesSearched, skipped-entry counts (I/O vs protected-policy), and truncated=true when the file/result cap is reached. Inaccessible individual paths are skipped instead of failing the whole search.",
        annotations(read_only_hint = true)
    )]
    async fn search_files(
        &self,
        Parameters(params): Parameters<SearchFilesParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let mut result = filesystem::search_files_detailed(
                &workspace,
                &params.relative_path,
                &params.query,
            )?;
            for item in &mut result.matches {
                item.preview = secret_guard::redact_text(&item.preview);
            }
            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Files,
                "searchFiles",
                format!(
                    "Searched for ‘{}’ · {} matches · {} files searched",
                    params.query,
                    result.matches.len(),
                    result.searched_file_count
                ),
                Some(format!(
                    "Scope: {} · {} skipped{}",
                    if params.relative_path.is_empty() {
                        "."
                    } else {
                        &params.relative_path
                    },
                    result.skipped_entry_count,
                    if result.truncated {
                        " · truncated"
                    } else {
                        ""
                    }
                )),
            );
            Ok(result)
        })
        .await
    }

    #[tool(
        description = "Fast bounded search for large projects. Searches text incrementally instead of discovering thousands of files first, stops each page after a small work budget, and returns nextCursor when more work remains. Reuse the same workspace/path/query with nextCursor to continue from the exact scan position instead of restarting. Identical immediate retries reuse the recent first page, and concurrent heavy searches in one workspace return busy/retryAfterMs instead of piling up duplicate scans. Prefer this over search_files for large repositories.",
        annotations(read_only_hint = true)
    )]
    async fn fast_search_files(
        &self,
        Parameters(params): Parameters<FastSearchFilesParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let mut result = large_project_read::fast_search_page(
                &workspace,
                &params.relative_path,
                &params.query,
                params.cursor.as_deref(),
                params.max_results,
                params.budget_ms,
            )?;
            for item in &mut result.matches {
                item.preview = secret_guard::redact_text(&item.preview);
            }
            if !result.busy {
                record_observation(
                    &app,
                    &workspace,
                    trace_group_id.as_deref(),
                    ActivityKind::Files,
                    "fastSearchFiles",
                    format!(
                        "Fast searched for ‘{}’ · {} matches · {} files this page",
                        params.query,
                        result.matches.len(),
                        result.files_searched_this_page
                    ),
                    Some(format!(
                        "Scope: {} · {}ms · {}",
                        if params.relative_path.is_empty() {
                            "."
                        } else {
                            &params.relative_path
                        },
                        result.elapsed_ms,
                        if result.done {
                            "done"
                        } else {
                            "continuation available"
                        }
                    )),
                );
            }
            Ok(result)
        })
        .await
    }

    #[tool(
        description = "Read a bounded UTF-8 line range from a text file, including files larger than the normal read_file limit. Defaults to 240 lines and returns nextCursor when more data remains. For high startLine values the operation stays time-bounded; if it cannot reach the requested line in one slice, call again with nextCursor instead of restarting. Cursors are rejected if the file changed.",
        annotations(read_only_hint = true)
    )]
    async fn read_file_range(
        &self,
        Parameters(params): Parameters<ReadFileRangeParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let result = large_project_read::read_file_range(
                &workspace,
                &params.relative_path,
                params.start_line,
                params.max_lines,
                params.cursor.as_deref(),
            )?;
            if let Some(kind) = secret_guard::detect_secret(result.content.as_bytes()) {
                return Err(format!(
                    "RepoTunnel withheld the requested range from '{}' because its text appears to contain {kind}. Secrets are never returned to an AI through MCP.",
                    params.relative_path
                ));
            }
            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Files,
                "readFileRange",
                format!(
                    "Read {} · lines {}-{}",
                    params.relative_path, result.start_line, result.end_line
                ),
                Some(format!(
                    "{} bytes total · {}ms · {}",
                    result.size,
                    result.elapsed_ms,
                    if result.eof { "EOF" } else { "continuation available" }
                )),
            );
            Ok(result)
        })
        .await
    }

    #[tool(
        description = "List a large folder in bounded pages instead of failing when it contains more than the normal list_directory limit. Returns up to pageSize accessible entries plus nextCursor. Reuse the same workspace/path with nextCursor to continue the same in-memory read-only listing session. Existing list_directory remains unchanged as the small-folder fallback.",
        annotations(read_only_hint = true)
    )]
    async fn list_directory_page(
        &self,
        Parameters(params): Parameters<ListDirectoryPageParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let result = large_project_read::list_directory_page(
                &workspace,
                &params.relative_path,
                params.cursor.as_deref(),
                params.page_size,
            )?;
            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Files,
                "listDirectoryPage",
                format!(
                    "Listed {} · {} entries this page",
                    if params.relative_path.is_empty() {
                        "."
                    } else {
                        &params.relative_path
                    },
                    result.entries.len()
                ),
                Some(format!(
                    "{} scanned · {}ms · {}",
                    result.scanned_entries,
                    result.elapsed_ms,
                    if result.done {
                        "done"
                    } else {
                        "continuation available"
                    }
                )),
            );
            Ok(result)
        })
        .await
    }

    #[tool(
        description = "Create a new UTF-8 text file. In review mode this queues a diff for local approval; in automatic mode it applies immediately with history and an undo point. Check applied and queued in the result: Review mode returns applied=false, queued=true until the user acts locally."
    )]
    async fn create_file(
        &self,
        Parameters(params): Parameters<FileContentParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let edit_group_id = request_edit_group_id(&parts);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let outcome = changes::create_file(
                &app,
                &workspace,
                params.relative_path,
                params.content,
                edit_group_id.as_deref(),
            )?;
            let _ = activity::record_change_outcome(
                &app,
                &workspace,
                edit_group_id.as_deref(),
                &outcome,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "Replace the complete contents of an existing UTF-8 text file. Prefer patch_file for targeted edits. Review-mode writes are queued locally; automatic-mode writes are backed up and applied immediately. Check applied and queued in the result: Review mode returns applied=false, queued=true until the user acts locally."
    )]
    async fn write_file(
        &self,
        Parameters(params): Parameters<FileContentParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let edit_group_id = request_edit_group_id(&parts);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let outcome = changes::write_file(
                &app,
                &workspace,
                params.relative_path,
                params.content,
                edit_group_id.as_deref(),
            )?;
            let _ = activity::record_change_outcome(
                &app,
                &workspace,
                edit_group_id.as_deref(),
                &outcome,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "Apply a targeted exact-context edit. This is the preferred code-edit tool. The preview is queued for local approval in review mode or applied with backup/history in automatic mode. Check applied in the result."
    )]
    async fn patch_file(
        &self,
        Parameters(params): Parameters<PatchFileParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let edit_group_id = request_edit_group_id(&parts);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let outcome = changes::patch_file(
                &app,
                &workspace,
                params.relative_path,
                params.expected,
                params.replacement,
                edit_group_id.as_deref(),
            )?;
            let _ = activity::record_change_outcome(
                &app,
                &workspace,
                edit_group_id.as_deref(),
                &outcome,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "Create a new folder inside an approved workspace. Review-mode changes require local approval; automatic-mode changes are recorded and applied immediately."
    )]
    async fn create_directory(
        &self,
        Parameters(params): Parameters<CreateDirectoryParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let edit_group_id = request_edit_group_id(&parts);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let outcome = changes::create_directory(
                &app,
                &workspace,
                params.relative_path,
                params.recursive,
                edit_group_id.as_deref(),
            )?;
            let _ = activity::record_change_outcome(
                &app,
                &workspace,
                edit_group_id.as_deref(),
                &outcome,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "Rename an existing file or folder within its current parent folder. This goes through RepoTunnel change review/history and never overwrites an existing destination."
    )]
    async fn rename_entry(
        &self,
        Parameters(params): Parameters<RenameEntryParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let edit_group_id = request_edit_group_id(&parts);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let outcome = changes::rename_entry(
                &app,
                &workspace,
                params.relative_path,
                params.new_name,
                edit_group_id.as_deref(),
            )?;
            let _ = activity::record_change_outcome(
                &app,
                &workspace,
                edit_group_id.as_deref(),
                &outcome,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "Move an existing file or folder to another relative path in the same approved workspace. This goes through RepoTunnel review/history and never overwrites an existing destination."
    )]
    async fn move_entry(
        &self,
        Parameters(params): Parameters<MoveEntryParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let edit_group_id = request_edit_group_id(&parts);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let outcome = changes::move_entry(
                &app,
                &workspace,
                params.source_path,
                params.destination_path,
                edit_group_id.as_deref(),
            )?;
            let _ = activity::record_change_outcome(
                &app,
                &workspace,
                edit_group_id.as_deref(),
                &outcome,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "DESTRUCTIVE: Request deletion of an existing file or folder. File deletions can receive an undo point; recursive directory deletions are recorded but may not be safely undoable. In review mode deletion waits for local approval."
    )]
    async fn delete_entry(
        &self,
        Parameters(params): Parameters<DeleteEntryParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let edit_group_id = request_edit_group_id(&parts);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let outcome = changes::delete_entry(
                &app,
                &workspace,
                params.relative_path,
                params.recursive,
                edit_group_id.as_deref(),
            )?;
            let _ = activity::record_change_outcome(
                &app,
                &workspace,
                edit_group_id.as_deref(),
                &outcome,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "Inspect metadata for an accessible workspace-relative file or folder. This does not modify the project.",
        annotations(read_only_hint = true)
    )]
    async fn get_file_info(
        &self,
        Parameters(params): Parameters<WorkspacePathParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let info = filesystem::file_info(&workspace, &params.relative_path)?;
            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Files,
                "fileInfo",
                format!("Inspected {}", params.relative_path),
                None,
            );
            Ok(info)
        })
        .await
    }

    #[tool(
        description = "Report whether RepoTunnel's native OS command sandbox is available. AI command execution is refused when the required platform sandbox is unavailable.",
        annotations(read_only_hint = true)
    )]
    async fn get_execution_status(&self) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            Ok(execution::execution_status())
        })
        .await
    }

    #[tool(
        description = "List the safe build/test/check/lint command presets RepoTunnel discovered for an approved project. Only these preset IDs can be requested; there is no generic shell or arbitrary command string.",
        annotations(read_only_hint = true)
    )]
    async fn list_command_presets(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            execution::list_presets(&workspace)
        })
        .await
    }

    #[tool(
        description = "Request execution of one exact command preset returned by list_command_presets. Commands run with network disabled in a disposable project copy inside RepoTunnel's native OS sandbox, so command side effects are discarded. Depending on the project's command policy, the request either queues for local approval, runs automatically, or is blocked."
    )]
    async fn run_command(
        &self,
        Parameters(params): Parameters<WorkspaceCommandParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let outcome = execution::request_command(&app, &workspace, &params.preset_id)?;
            let _ = activity::record_sandbox_command(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                &outcome,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "List recent sandboxed command records and their pending/running/completed/failed/rejected/timed-out status. This tool cannot approve or reject a pending command.",
        annotations(read_only_hint = true)
    )]
    async fn list_command_history(
        &self,
        Parameters(params): Parameters<ListCommandsParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            execution::list_history(
                &app,
                params.workspace_id.as_deref(),
                params.limit.unwrap_or(30),
            )
        })
        .await
    }

    #[tool(
        description = "Run a short one-shot shell command with write access to the approved workspace and network access, but without general access to the user's home directory or host filesystem. RepoTunnel uses an OS sandbox and a sanitized environment for AI commands, redacts credential-like output, and refuses to fall back to unrestricted host access if the sandbox is unavailable. Repository metadata is mounted read-only so normal Git inspection commands work. When GitHub is connected in RepoTunnel, authenticated direct GitHub CLI commands and normal Git push can use that shared connection without exposing its credential; GitHub authentication changes and token export remain local-only. Do not infer that GitHub is disconnected merely because a sandboxed or compound shell command cannot see GitHub credentials: use github_connection_status for the authoritative connection state, and prefer RepoTunnel's native GitHub/Git tools for account operations. The legacy user_requested_push flag is still accepted, but a verified RepoTunnel GitHub connection itself grants GitHub publishing access. Git add/commit remain routed through RepoTunnel's native audited Git tools. For dev servers/watchers and for any build/test/install/verification likely to exceed about 30 seconds, use start_process instead so the MCP request returns immediately; then poll with read_process_output/list_processes."
    )]
    async fn run_terminal_command(
        &self,
        Parameters(params): Parameters<TerminalCommandParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let continuation_identities = continuation_identity_keys(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback_with_identities(
                &app,
                &workspace.id,
                &continuation_identities,
            );
            let github_publish_allowed =
                params.user_requested_push.unwrap_or(false) || github::status().connected;
            git::validate_ai_terminal_git_command(
                &workspace,
                &params.command,
                github_publish_allowed,
            )?;
            let outcome = terminal::request_terminal_command(
                &app,
                &workspace,
                params.command,
                params.cwd,
                params.timeout_seconds,
                params.env.unwrap_or_default(),
                true,
                github_publish_allowed,
            )?;
            let _ = activity::record_terminal_outcome(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                &outcome,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "List recent real-workspace terminal commands, including pending review requests and final exit/output status. Use this to verify a queued AI Review command after the user acts on it locally.",
        annotations(read_only_hint = true)
    )]
    async fn list_terminal_history(
        &self,
        Parameters(params): Parameters<ListCommandsParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            terminal::list_terminal_history(
                &app,
                params.workspace_id.as_deref(),
                params.limit.unwrap_or(30),
            )
        })
        .await
    }

    #[tool(
        description = "Start a durable persistent process inside the approved workspace security sandbox, suitable for development servers, watchers, builds, tests, and workers. RepoTunnel supervises the job outside the UI lifetime so its process ID, status, and bounded logs can be recovered after ChatGPT/MCP reconnects or a RepoTunnel UI restart. The AI process can write the project and use the network but cannot browse the user's home directory or host filesystem; credential-like environment overrides are rejected and returned output is redacted. In AI Auto it starts immediately. In AI Review it may queue for local Accept/Reject."
    )]
    async fn start_process(
        &self,
        Parameters(params): Parameters<ManagedProcessStartParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let continuation_identities = continuation_identity_keys(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback_with_identities(
                &app,
                &workspace.id,
                &continuation_identities,
            );
            git::validate_ai_terminal_git_command(&workspace, &params.command, false)?;
            let outcome = terminal::request_process_start(
                &app,
                &workspace,
                params.command,
                params.cwd,
                params.label,
                params.env.unwrap_or_default(),
                true,
            )?;
            let _ = activity::record_process_outcome(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                &outcome,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "Read one lightweight runtime snapshot for an approved workspace: Git branch/HEAD/change state plus RepoTunnel-managed process state. Use this instead of parallel git_status + list_processes calls when both are needed. It deliberately skips browser diagnostics, listener scans, log tails, and read-only activity-observation writes so routine status polling stays fast.",
        annotations(read_only_hint = true)
    )]
    async fn get_workspace_runtime_status(
        &self,
        Parameters(params): Parameters<WorkspaceRuntimeStatusParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let git = git::repository_status(&workspace);
            let processes = terminal::list_processes(
                &app,
                Some(&workspace.id),
                params.process_limit.unwrap_or(25),
            )?;
            let running_processes = processes
                .iter()
                .filter(|record| {
                    matches!(record.status, crate::models::ManagedProcessStatus::Running)
                })
                .count();
            Ok(WorkspaceRuntimeStatus {
                workspace_id: workspace.id,
                git,
                processes,
                running_processes,
            })
        })
        .await
    }

    #[tool(
        description = "Inspect workspace-specific environment fidelity without changing the computer. Returns the host workspace path versus AI-sandbox mapping, RepoTunnel host PATH versus sandbox PATH, detected build/runtime/media/application tools with host and sandbox visibility, SDK environment variables, non-secret GUI session variables, and concrete mismatches. It intentionally does not execute the user's shell profile or rc files.",
        annotations(read_only_hint = true)
    )]
    async fn get_environment_diagnostics(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            Ok(environment::diagnostics(&workspace))
        })
        .await
    }

    #[tool(
        description = "List capability-oriented metadata for known local tools without probing random binaries: installed/not installed, version when safely queryable, host/sandbox path visibility, category, CLI/scriptability, GUI-control suitability, launchability, known file types, and concrete capabilities such as 2D animation, image editing, rendering, lip-sync, media inspection or archive extraction.",
        annotations(read_only_hint = true)
    )]
    async fn list_tool_capabilities(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            Ok(environment::tool_capabilities(&workspace))
        })
        .await
    }

    #[tool(
        description = "Read factual resource availability for the approved workspace so the AI can choose an appropriate external workflow without guessing: logical CPU count, 1-minute load when available, total/available RAM, workspace filesystem capacity/free space, and positively detected GPU devices. Unknown values stay null and are explained instead of being invented.",
        annotations(read_only_hint = true)
    )]
    async fn get_system_resources(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            Ok(system_resources::snapshot(&workspace))
        })
        .await
    }

    #[tool(
        description = "Create or reopen a lightweight RepoTunnel-owned temporary task directory under .repotunnel-tmp/<task_id> inside the approved workspace. Use it for downloads, generated media, extracted archives and intermediate renders. It is excluded from normal project indexing and carries an ownership marker so cleanup can never target unrelated user files."
    )]
    async fn create_temp_workspace(
        &self,
        Parameters(params): Parameters<CreateTempWorkspaceParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            temp_workspace::create(&workspace, &params.task_id, &params.label)
        })
        .await
    }

    #[tool(
        description = "List RepoTunnel-owned temporary task directories for an approved workspace with exact byte/file/directory counts and preserved state. Unmarked directories are ignored rather than treated as RepoTunnel temp data.",
        annotations(read_only_hint = true)
    )]
    async fn list_temp_workspaces(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            temp_workspace::list(&workspace)
        })
        .await
    }

    #[tool(
        description = "Inspect one RepoTunnel-owned temporary task directory and return its current exact disk usage, counts, timestamps and preserve state.",
        annotations(read_only_hint = true)
    )]
    async fn inspect_temp_workspace(
        &self,
        Parameters(params): Parameters<TempWorkspaceParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            temp_workspace::inspect(&workspace, &params.task_id)
        })
        .await
    }

    #[tool(
        description = "Mark a RepoTunnel temporary task as preserved/unpreserved. Preserve unfinished work that must survive cleanup or reconnects; unpreserve it once only disposable intermediates remain."
    )]
    async fn set_temp_workspace_preserved(
        &self,
        Parameters(params): Parameters<PreserveTempWorkspaceParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            temp_workspace::set_preserved(&workspace, &params.task_id, params.preserved)
        })
        .await
    }

    #[tool(
        description = "Delete exactly one RepoTunnel-owned temporary task directory and report freed bytes. Cleanup refuses unmarked directories. A preserved task is not deleted unless force_preserved=true, which must only be used when the human's intended final/unfinished outputs are already safe elsewhere."
    )]
    async fn cleanup_temp_workspace(
        &self,
        Parameters(params): Parameters<CleanupTempWorkspaceParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            temp_workspace::cleanup(
                &workspace,
                &params.task_id,
                params.force_preserved.unwrap_or(false),
            )
        })
        .await
    }

    #[tool(
        description = "Operate on regular files inside one marker-owned RepoTunnel temporary task. copy_to_workspace/move_to_workspace can keep a final file by placing it in a normal approved project path outside .repotunnel-tmp; rename stays inside the same temp task; delete can only remove a regular non-symlink temp file. Parent traversal and marker mutation are rejected."
    )]
    async fn temp_workspace_file_action(
        &self,
        Parameters(params): Parameters<TempWorkspaceFileActionParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            match params.action {
                TempWorkspaceFileActionParam::CopyToWorkspace => temp_workspace::copy_to_workspace(
                    &workspace,
                    &params.task_id,
                    &params.source_relative,
                    params.destination_relative.as_deref().ok_or_else(|| {
                        "destination_relative is required for copy_to_workspace.".to_string()
                    })?,
                    params.overwrite.unwrap_or(false),
                ),
                TempWorkspaceFileActionParam::MoveToWorkspace => temp_workspace::move_to_workspace(
                    &workspace,
                    &params.task_id,
                    &params.source_relative,
                    params.destination_relative.as_deref().ok_or_else(|| {
                        "destination_relative is required for move_to_workspace.".to_string()
                    })?,
                    params.overwrite.unwrap_or(false),
                ),
                TempWorkspaceFileActionParam::Rename => temp_workspace::rename_file(
                    &workspace,
                    &params.task_id,
                    &params.source_relative,
                    params.destination_relative.as_deref().ok_or_else(|| {
                        "destination_relative is required for rename.".to_string()
                    })?,
                    params.overwrite.unwrap_or(false),
                ),
                TempWorkspaceFileActionParam::Delete => temp_workspace::delete_file(
                    &workspace,
                    &params.task_id,
                    &params.source_relative,
                ),
            }
        })
        .await
    }

    #[tool(
        description = "List RepoTunnel-managed persistent processes with running/exited/stopped/failed state, PID when attached, exit status, restart count, and command metadata. Routine status polling reads/refreshed process history directly and does not write a read-only activity observation.",
        annotations(read_only_hint = true)
    )]
    async fn list_processes(
        &self,
        Parameters(params): Parameters<ListProcessesParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            terminal::list_processes(
                &app,
                params.workspace_id.as_deref(),
                params.limit.unwrap_or(50),
            )
        })
        .await
    }

    #[tool(
        description = "Read bounded stdout/stderr from a managed persistent process. Supply the returned next offsets on later calls to monitor only new output. This also refreshes and returns the current process state.",
        annotations(read_only_hint = true)
    )]
    async fn read_process_output(
        &self,
        Parameters(params): Parameters<ProcessOutputParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let process = terminal::get_process(&app, &params.process_id)?;
            let _workspace = approved_workspace(&app, &process.workspace_id)?;
            let output = terminal::read_process_output(
                &app,
                &params.process_id,
                params.stdout_offset.unwrap_or(0),
                params.stderr_offset.unwrap_or(0),
                params.max_bytes.unwrap_or(64 * 1024),
            )?;
            if let Ok(updated) = terminal::get_process(&app, &params.process_id) {
                if !matches!(
                    updated.status,
                    crate::models::ManagedProcessStatus::Running
                        | crate::models::ManagedProcessStatus::Pending
                ) {
                    activity::sync_process(&app, &updated);
                }
            }
            Ok(output)
        })
        .await
    }

    #[tool(
        description = "Wait efficiently for a managed process to emit one of the supplied literal success/failure patterns, exit, or reach a bounded timeout. This avoids repeated read_process_output polling. Scanning starts at the supplied stdout/stderr offsets, failure patterns take precedence within the same observed chunk, and the result includes the current process state plus bounded output.",
        annotations(read_only_hint = true)
    )]
    async fn wait_process(
        &self,
        Parameters(params): Parameters<ProcessWaitParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let process = terminal::get_process(&app, &params.process_id)?;
            let _workspace = approved_workspace(&app, &process.workspace_id)?;
            terminal::wait_process(
                &app,
                &params.process_id,
                params.success_patterns,
                params.failure_patterns,
                params.timeout_seconds.unwrap_or(120),
                params.stdout_offset.unwrap_or(0),
                params.stderr_offset.unwrap_or(0),
                params.max_bytes.unwrap_or(64 * 1024),
            )
        })
        .await
    }

    #[tool(
        description = "Stop a RepoTunnel-managed persistent process and its process group. By default RepoTunnel attempts a graceful stop before forcing termination. This never requires an extra confirmation in AI Auto."
    )]
    async fn stop_process(
        &self,
        Parameters(params): Parameters<StopProcessParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let existing = terminal::get_process(&app, &params.process_id)?;
            let workspace = approved_workspace(&app, &existing.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let record =
                terminal::stop_process(&app, &params.process_id, params.force.unwrap_or(false))?;
            activity::sync_process(&app, &record);
            let _ = activity::record_process_record(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                "stopProcess",
                &record,
            );
            Ok(record)
        })
        .await
    }

    #[tool(
        description = "Restart a previously started RepoTunnel-managed process using the same command, workspace-relative working directory, label, and environment overrides. The process keeps the same RepoTunnel process ID and increments restartCount."
    )]
    async fn restart_process(
        &self,
        Parameters(params): Parameters<ProcessIdParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let existing = terminal::get_process(&app, &params.process_id)?;
            let workspace = approved_workspace(&app, &existing.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let record = terminal::restart_process(&app, &workspace, &existing.id)?;
            activity::sync_process(&app, &record);
            let _ = activity::record_process_record(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                "restartProcess",
                &record,
            );
            Ok(record)
        })
        .await
    }

    #[tool(
        description = "List desktop applications RepoTunnel can launch directly, including detected browsers, editors, file managers, and terminals. Returns stable application IDs for launch_target.",
        annotations(read_only_hint = true)
    )]
    async fn list_launchable_applications(&self) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            Ok(launcher::list_applications())
        })
        .await
    }

    #[tool(
        description = "List the five optional deep local integrations (Android Studio, Unity, Blender, Godot, Docker) for an approved project, including whether each app is detected, whether the human enabled ChatGPT access locally, and the exact allowlisted actions available. This is read-only; MCP cannot enable its own integrations.",
        annotations(read_only_hint = true)
    )]
    async fn list_deep_integrations(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            integrations::list(&app, &workspace.id)
        })
        .await
    }

    #[tool(
        description = "Read whether the human enabled the global Gmail / Google Sign-In permission in the RepoTunnel desktop app. When enabled, AI browser workflows may reuse the persistent managed Google session and access Gmail only as needed for sign-in/verification flows. MCP cannot enable this permission itself.",
        annotations(read_only_hint = true)
    )]
    async fn get_gmail_access_status(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            let _workspace = approved_workspace(&app, &params.workspace_id)?;
            gmail_access::is_enabled(&app)
        })
        .await
    }

    #[tool(
        description = "Clean transient resources owned by the current AI session for one approved project when they are no longer needed: managed-browser tabs/session attachment and detached applications launched through RepoTunnel. AI Workspace app sessions are deliberately PRESERVED across turn/session cleanup so VS Code, Terminal, Kdenlive, and other in-progress GUI work can be resumed after reconnect. To actually close an AI Workspace app, use ai_workspace_session action=stop with its app_session_id. Do not stop AI Workspace merely because a ChatGPT turn or MCP session ended."
    )]
    async fn cleanup_ai_resources(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let client_key = request_resource_owner_key(&parts, &context).ok_or_else(|| {
            McpError::invalid_request("AI session identity is unavailable.", None)
        })?;
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ai_resources::cleanup(&app, &workspace, &client_key)
        })
        .await
    }

    #[tool(
        description = "Run one bounded action through a deep local integration that the human explicitly enabled in RepoTunnel. Call list_deep_integrations first and use only an action it returns. RepoTunnel refuses disabled/unavailable integrations, keeps targets inside the approved project, and routes commands through the project's command policy. Editing project files still uses the normal RepoTunnel file tools."
    )]
    async fn integration_action(
        &self,
        Parameters(params): Parameters<IntegrationActionParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let client_key = request_resource_owner_key(&parts, &context);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let result = integrations::run_action(
                &app,
                &workspace,
                &params.integration_id,
                &params.action,
                params.target.as_deref(),
            )?;
            if let Some(command) = result.command.as_ref() {
                let _ = activity::record_terminal_outcome(
                    &app,
                    &workspace,
                    trace_group_id.as_deref(),
                    command,
                );
            }
            if let Some(launch) = result.launch.as_ref() {
                let _ = activity::record_launch_record(
                    &app,
                    &workspace,
                    trace_group_id.as_deref(),
                    &launch.launch,
                );
                if !launch.queued {
                    if let (Some(client_key), Some(pid)) =
                        (client_key.as_deref(), launch.launch.pid)
                    {
                        ai_resources::own_launched_pid(&workspace.id, client_key, pid);
                    }
                }
            }
            Ok(result)
        })
        .await
    }

    #[tool(
        description = "Manage RepoTunnel AI Workspace, a shared isolated virtual desktop that can host multiple bounded native app sessions for multiple AIs. action=status returns the durable desktop session_id, this AI's applications[], per-app ownership diagnostics, workspaceVisibleRoot, and aggregate resource limits. action=start reuses this AI's matching app by default. Repeated or reconnect-ambiguous start calls NEVER create another implicit duplicate: pass the prior app_session_id to reattach/reclaim, or set new_instance=true only when a genuinely separate second instance is intentionally required. action=reclaim safely rebinds one matching app_session_id only after its owner lease is stale. action=stop closes only this AI's selected app session; the shared desktop remains alive while other app sessions are running. Never use an AI Workspace Terminal to launch Chrome/Chromium/Brave/Edge/Firefox; use RepoTunnel managed browser automation for browser testing. The human must enable Desktop permission first."
    )]
    async fn ai_workspace_session(
        &self,
        Parameters(params): Parameters<AiWorkspaceSessionParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let client_key = request_resource_owner_key(&parts, &context);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            if params.action != "status" {
                ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            }
            let state = app.state::<AppState>();
            match params.action.as_str() {
                "status" => {
                    let status = state.ai_workspace.status(&app, &workspace.id)?;
                    ai_workspace_session_payload(&workspace, status, client_key.as_deref())
                }
                "start" => {
                    let client_key = client_key
                        .as_deref()
                        .ok_or_else(|| "AI session identity is unavailable.".to_string())?;
                    let application_id = params.application_id.as_deref().ok_or_else(|| {
                        "AI Workspace action=start requires application_id from list_launchable_applications.".to_string()
                    })?;
                    let current = state.ai_workspace.status(&app, &workspace.id)?;

                    if let Some(requested_app_session_id) = params
                        .app_session_id
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                    {
                        let application = current
                            .applications
                            .iter()
                            .find(|application| {
                                application.app_session_id == requested_app_session_id
                            })
                            .ok_or_else(|| {
                                "That AI Workspace app_session_id is not running on this shared desktop."
                                    .to_string()
                            })?;
                        if application.application_id != application_id {
                            return Err(
                                "The requested app_session_id belongs to a different application."
                                    .to_string(),
                            );
                        }
                        let ownership = ai_resources::ai_workspace_app_ownership(
                            &workspace.id,
                            requested_app_session_id,
                            Some(client_key),
                        )?;
                        if ownership.owner_session_id.is_some()
                            && !ownership.owned_by_current_session
                            && ownership.owner_lease_active
                        {
                            return Err(format!(
                                "That AI Workspace app session is actively owned by another AI session ({}).",
                                ownership
                                    .owner_session_id
                                    .as_deref()
                                    .unwrap_or("unknown-owner")
                            ));
                        }
                        if ownership.owner_session_id.is_some()
                            && !ownership.owned_by_current_session
                        {
                            ai_resources::reclaim_ai_workspace_app_if_stale(
                                &workspace.id,
                                client_key,
                                requested_app_session_id,
                            )?;
                        } else {
                            ai_resources::claim_ai_workspace_app(
                                &workspace.id,
                                client_key,
                                requested_app_session_id,
                            )?;
                        }
                        return ai_workspace_session_payload(
                            &workspace,
                            current,
                            Some(client_key),
                        );
                    }

                    let explicit_new_instance = params.new_instance.unwrap_or(false);

                    if !explicit_new_instance {
                        let owned_sessions =
                            ai_resources::owned_ai_workspace_app_sessions(&workspace.id, client_key)?;
                        if let Some(application) = current.applications.iter().find(|application| {
                            application.application_id == application_id
                                && owned_sessions.contains(&application.app_session_id)
                        }) {
                            ai_resources::assert_ai_workspace_app_owned(
                                &workspace.id,
                                client_key,
                                &application.app_session_id,
                            )?;
                            return ai_workspace_session_payload(
                                &workspace,
                                current,
                                Some(client_key),
                            );
                        }

                        let unowned_matches = current
                            .applications
                            .iter()
                            .filter(|application| application.application_id == application_id)
                            .filter_map(|application| {
                                let ownership = ai_resources::ai_workspace_app_ownership(
                                    &workspace.id,
                                    &application.app_session_id,
                                    Some(client_key),
                                )
                                .ok()?;
                                ownership
                                    .owner_session_id
                                    .is_none()
                                    .then_some(application.app_session_id.clone())
                            })
                            .collect::<Vec<_>>();

                        if unowned_matches.len() == 1 {
                            ai_resources::claim_ai_workspace_app(
                                &workspace.id,
                                client_key,
                                &unowned_matches[0],
                            )?;
                            return ai_workspace_session_payload(
                                &workspace,
                                current,
                                Some(client_key),
                            );
                        }
                        if unowned_matches.len() > 1 {
                            return Err(format!(
                                "Multiple unowned {application_id} app sessions are already running. Pass app_session_id to reattach to the intended one. RepoTunnel will not create another implicit duplicate."
                            ));
                        }

                        let matching_sessions = current
                            .applications
                            .iter()
                            .filter(|application| application.application_id == application_id)
                            .filter_map(|application| {
                                ai_resources::ai_workspace_app_ownership(
                                    &workspace.id,
                                    &application.app_session_id,
                                    Some(client_key),
                                )
                                .ok()
                                .map(|ownership| {
                                    (application.app_session_id.clone(), ownership)
                                })
                            })
                            .collect::<Vec<_>>();

                        let stale_matches = matching_sessions
                            .iter()
                            .filter(|(_, ownership)| {
                                ownership.owner_session_id.is_some()
                                    && !ownership.owned_by_current_session
                                    && !ownership.owner_lease_active
                            })
                            .map(|(app_session_id, _)| app_session_id.clone())
                            .collect::<Vec<_>>();

                        if stale_matches.len() == 1 {
                            ai_resources::reclaim_ai_workspace_app_if_stale(
                                &workspace.id,
                                client_key,
                                &stale_matches[0],
                            )?;
                            return ai_workspace_session_payload(
                                &workspace,
                                current,
                                Some(client_key),
                            );
                        }

                        if !matching_sessions.is_empty() {
                            return Err(format!(
                                "{application_id} is already running in AI Workspace. RepoTunnel blocked another implicit instance to prevent duplicate windows. Reuse the prior app_session_id; if exactly one prior owner becomes stale RepoTunnel will reattach automatically. Pass new_instance=true only when a genuinely separate second instance is required."
                            ));
                        }
                    }

                    let reserved_app_session_id = next_ai_workspace_app_session_id();
                    ai_resources::claim_ai_workspace_app(
                        &workspace.id,
                        client_key,
                        &reserved_app_session_id,
                    )?;

                    let status = match state.ai_workspace.start_with_app_session_id(
                        &app,
                        &workspace,
                        application_id,
                        params.target.as_deref(),
                        Some(&reserved_app_session_id),
                    ) {
                        Ok(status) => status,
                        Err(error) => {
                            ai_resources::release_ai_workspace_app(
                                &workspace.id,
                                client_key,
                                &reserved_app_session_id,
                            );
                            return Err(error);
                        }
                    };

                    if status.last_started_app_session_id.as_deref()
                        != Some(reserved_app_session_id.as_str())
                    {
                        let _ = state.ai_workspace.stop_app_session(
                            &app,
                            &workspace.id,
                            &reserved_app_session_id,
                        );
                        ai_resources::release_ai_workspace_app(
                            &workspace.id,
                            client_key,
                            &reserved_app_session_id,
                        );
                        return Err(
                            "AI Workspace launched an application but returned a mismatched appSessionId; the launch was cleaned up."
                                .to_string(),
                        );
                    }

                    ai_resources::assert_ai_workspace_app_owned(
                        &workspace.id,
                        client_key,
                        &reserved_app_session_id,
                    )?;
                    ai_workspace_session_payload(&workspace, status, Some(client_key))
                }
                "reclaim" => {
                    let client_key = client_key
                        .as_deref()
                        .ok_or_else(|| "AI session identity is unavailable.".to_string())?;
                    let status = state.ai_workspace.status(&app, &workspace.id)?;
                    if params
                        .session_id
                        .as_deref()
                        .map(str::trim)
                        .is_some_and(|value| !value.is_empty())
                    {
                        require_ai_workspace_session_match(&status, params.session_id.as_deref())?;
                    }
                    let app_session_id = if let Some(app_session_id) = params
                        .app_session_id
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                    {
                        app_session_id.to_string()
                    } else if status.applications.len() == 1 {
                        status.applications[0].app_session_id.clone()
                    } else {
                        return Err(
                            "AI Workspace action=reclaim requires app_session_id when multiple applications share the desktop."
                                .to_string(),
                        );
                    };
                    if !state
                        .ai_workspace
                        .app_session_exists(&workspace.id, &app_session_id)?
                    {
                        return Err(
                            "That AI Workspace app_session_id is not running on this shared desktop."
                                .to_string(),
                        );
                    }
                    ai_resources::reclaim_ai_workspace_app_if_stale(
                        &workspace.id,
                        client_key,
                        &app_session_id,
                    )?;
                    let status = state.ai_workspace.status(&app, &workspace.id)?;
                    ai_workspace_session_payload(&workspace, status, Some(client_key))
                }
                "stop" => {
                    let client_key = client_key
                        .as_deref()
                        .ok_or_else(|| "AI session identity is unavailable.".to_string())?;
                    let current = state.ai_workspace.status(&app, &workspace.id)?;
                    if !current.running {
                        return ai_workspace_session_payload(
                            &workspace,
                            current,
                            Some(client_key),
                        );
                    }

                    let app_session_id = if params.stale_only.unwrap_or(false) {
                        if params
                            .session_id
                            .as_deref()
                            .map(str::trim)
                            .is_some_and(|value| !value.is_empty())
                        {
                            require_ai_workspace_session_match(
                                &current,
                                params.session_id.as_deref(),
                            )?;
                        }
                        let app_session_id = params
                            .app_session_id
                            .as_deref()
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .ok_or_else(|| {
                                "stale_only AI Workspace stop requires app_session_id."
                                    .to_string()
                            })?;
                        ai_resources::reclaim_ai_workspace_app_if_stale(
                            &workspace.id,
                            client_key,
                            app_session_id,
                        )?;
                        app_session_id.to_string()
                    } else {
                        resolve_ai_workspace_app_session(
                            &app,
                            &workspace,
                            Some(client_key),
                            params.app_session_id.as_deref(),
                        )?
                    };

                    let status =
                        state
                            .ai_workspace
                            .stop_app_session(&app, &workspace.id, &app_session_id)?;
                    ai_resources::release_ai_workspace_app(
                        &workspace.id,
                        client_key,
                        &app_session_id,
                    );
                    ai_workspace_session_payload(&workspace, status, Some(client_key))
                }
                _ => Err(
                    "AI Workspace session action must be status, start, reclaim, or stop."
                        .to_string(),
                ),
            }
        })
        .await
    }

    #[tool(
        description = "Inspect the isolated AI Workspace window list and exact bounds. Use the returned window IDs before visual pointer work so click/scroll coordinates can be relative to the intended dialog or application window instead of the whole virtual desktop.",
        annotations(read_only_hint = true)
    )]
    async fn ai_workspace_inspect(
        &self,
        Parameters(params): Parameters<AiWorkspaceInspectParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let app_session_id = resolve_ai_workspace_app_session(
                &app,
                &workspace,
                client_key.as_deref(),
                params.app_session_id.as_deref(),
            )?;
            let state = app.state::<AppState>();
            state
                .ai_workspace
                .inspect_app_session(&app, &workspace.id, &app_session_id, 300)
        })
        .await
    }

    #[tool(
        description = "Inspect the running isolated AI Workspace through its private AT-SPI accessibility bus. Returns the shared bounded semantic snapshot format with short-lived eN refs, normalized roles/actions/states/bounds, version/hash metadata, compact unchanged responses, and sensitive-field redaction. The private accessibility bus is isolated from the human desktop session.",
        annotations(read_only_hint = true)
    )]
    async fn ai_workspace_semantic_snapshot(
        &self,
        Parameters(params): Parameters<AiWorkspaceSemanticSnapshotParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let app_session_id = resolve_ai_workspace_app_session(
                &app,
                &workspace,
                client_key.as_deref(),
                params.app_session_id.as_deref(),
            )?;
            if !desktop_control::is_enabled(&app, &workspace.id)? {
                return Err("Desktop permission is off for this project.".to_string());
            }
            let state = app.state::<AppState>();
            state.ai_workspace.semantic_snapshot_app_session(
                &app,
                &workspace.id,
                &app_session_id,
                params.max_nodes.unwrap_or(300),
                params.known_hash,
            )
        })
        .await
    }

    #[tool(
        description = "Perform a click or type mutation through a short-lived eN ref from ai_workspace_semantic_snapshot. For type, pass either text or workspace_relative_path; RepoTunnel resolves workspace_relative_path to the exact approved host path visible inside the isolated app, which is useful for native file pickers. The ref must belong to the current isolated session; RepoTunnel rechecks the advertised action and sensitive-field policy, then the private AT-SPI helper revalidates the signed element immediately before mutation. Existing ai_workspace_action remains the visual fallback."
    )]
    async fn ai_workspace_semantic_action(
        &self,
        Parameters(params): Parameters<AiWorkspaceSemanticActionParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let app_session_id = resolve_ai_workspace_app_session(
                &app,
                &workspace,
                client_key.as_deref(),
                params.app_session_id.as_deref(),
            )?;
            if !desktop_control::is_enabled(&app, &workspace.id)? {
                return Err("Desktop permission is off for this project.".to_string());
            }
            let action = match params.action {
                UiSemanticActionParam::Click => "click",
                UiSemanticActionParam::Type => "type",
            };
            let type_text = if action == "type" {
                ai_workspace_type_text(
                    &workspace,
                    params.text.as_deref(),
                    params.workspace_relative_path.as_deref(),
                )?
            } else {
                if params.workspace_relative_path.is_some() {
                    return Err(
                        "workspace_relative_path is valid only for AI Workspace type actions."
                            .to_string(),
                    );
                }
                params.text.clone()
            };
            let state = app.state::<AppState>();
            state.ai_workspace.semantic_action_app_session(
                &app,
                &workspace.id,
                &app_session_id,
                &params.snapshot_id,
                &params.ref_id,
                action,
                type_text.as_deref(),
                params.clear_first.unwrap_or(false),
            )
        })
        .await
    }

    #[tool(
        description = "Run 1..64 already-grounded click/type/wait steps from one ai_workspace_semantic_snapshot in one bounded private helper request. Type steps accept either text or workspace_relative_path; RepoTunnel resolves project-relative paths to the exact approved host path visible inside native file pickers. This is the fast semantic path; refs/session/sensitive-field policy are checked before dispatch and signed private-AT-SPI identities are revalidated per step. Execution stops on first failure and invalidates the source snapshot. Existing ai_workspace_sequence and screenshot/coordinate actions remain available as fallback."
    )]
    async fn ai_workspace_semantic_sequence(
        &self,
        Parameters(params): Parameters<AiWorkspaceSemanticSequenceParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let app_session_id = resolve_ai_workspace_app_session(
                &app,
                &workspace,
                client_key.as_deref(),
                params.app_session_id.as_deref(),
            )?;
            if !desktop_control::is_enabled(&app, &workspace.id)? {
                return Err("Desktop permission is off for this project.".to_string());
            }
            let steps = params
                .steps
                .into_iter()
                .enumerate()
                .map(|(index, step)| match step.operation {
                    UiSemanticSequenceOperationParam::Click => {
                        let ref_id = step
                            .ref_id
                            .filter(|value| !value.trim().is_empty())
                            .ok_or_else(|| {
                                format!(
                                    "ref_id is required for AI Workspace semantic sequence step {} click.",
                                    index + 1
                                )
                            })?;
                        Ok(crate::ai_workspace::AiWorkspaceSemanticSequenceStep::Click { ref_id })
                    }
                    UiSemanticSequenceOperationParam::Type => {
                        let ref_id = step
                            .ref_id
                            .filter(|value| !value.trim().is_empty())
                            .ok_or_else(|| {
                                format!(
                                    "ref_id is required for AI Workspace semantic sequence step {} type.",
                                    index + 1
                                )
                            })?;
                        let text = ai_workspace_type_text(
                            &workspace,
                            step.text.as_deref(),
                            step.workspace_relative_path.as_deref(),
                        )?
                        .ok_or_else(|| {
                            format!(
                                "text or workspace_relative_path is required for AI Workspace semantic sequence step {} type.",
                                index + 1
                            )
                        })?;
                        Ok(crate::ai_workspace::AiWorkspaceSemanticSequenceStep::Type {
                            ref_id,
                            text,
                            clear_first: step.clear_first.unwrap_or(false),
                        })
                    }
                    UiSemanticSequenceOperationParam::Wait => {
                        Ok(crate::ai_workspace::AiWorkspaceSemanticSequenceStep::Wait {
                            wait_ms: step.wait_ms.unwrap_or(0),
                        })
                    }
                })
                .collect::<Result<Vec<_>, String>>()?;
            let state = app.state::<AppState>();
            state.ai_workspace.semantic_sequence_app_session(
                &app,
                &workspace.id,
                &app_session_id,
                &params.snapshot_id,
                &steps,
            )
        })
        .await
    }

    #[tool(
        description = "Send input only to the isolated AI Workspace display. Supported actions: activate, click, key, type, scroll. For type, pass either text or workspace_relative_path; RepoTunnel resolves project-relative paths to the exact approved host path visible inside native file pickers. Prefer a window_id from ai_workspace_inspect for click/scroll so normalized coordinates are relative to the intended dialog or application window; omit it only for full-screen fallback. Do not type commands that launch Chrome/Chromium/Brave/Edge/Firefox in an AI Workspace Terminal; use RepoTunnel managed browser automation instead. Credential/authentication-window typing remains blocked."
    )]
    async fn ai_workspace_action(
        &self,
        Parameters(params): Parameters<AiWorkspaceActionParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let client_key = request_resource_owner_key(&parts, &context);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let app_session_id = resolve_ai_workspace_app_session(
                &app,
                &workspace,
                client_key.as_deref(),
                params.app_session_id.as_deref(),
            )?;
            let type_text = if params.action == "type" {
                ai_workspace_type_text(
                    &workspace,
                    params.text.as_deref(),
                    params.workspace_relative_path.as_deref(),
                )?
            } else {
                if params.workspace_relative_path.is_some() {
                    return Err(
                        "workspace_relative_path is valid only for AI Workspace type actions."
                            .to_string(),
                    );
                }
                params.text.clone()
            };
            ensure_ai_workspace_terminal_text_allowed(
                &app,
                &workspace,
                &app_session_id,
                type_text.as_deref(),
            )?;
            let state = app.state::<AppState>();
            state.ai_workspace.action_app_session(
                &app,
                &workspace.id,
                &app_session_id,
                &params.action,
                params.window_id.as_deref(),
                params.x_ratio,
                params.y_ratio,
                params.click_count,
                params.shortcut.as_deref(),
                type_text.as_deref(),
                params.delta_x,
                params.delta_y,
            )
        })
        .await
    }

    #[tool(
        description = "Run 1..64 already-grounded AI Workspace actions in one bounded fast-path request. Supports activate, click, key, type, scroll, and wait steps; type steps accept either text or workspace_relative_path and wait can use a short delay, active-title condition, or isolated-window-count condition. Prefer this when several consecutive actions are already known because it avoids repeated MCP/helper startup round trips. Do not use Terminal type steps to launch Chrome/Chromium/Brave/Edge/Firefox; browser work belongs in RepoTunnel managed browser automation. The existing ai_workspace_action remains the reliable single-step fallback, and credential/authentication typing protections remain active."
    )]
    async fn ai_workspace_sequence(
        &self,
        Parameters(params): Parameters<AiWorkspaceSequenceParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let app_session_id = resolve_ai_workspace_app_session(
                &app,
                &workspace,
                client_key.as_deref(),
                params.app_session_id.as_deref(),
            )?;
            let steps = params
                .steps
                .into_iter()
                .map(|mut step| {
                    if step.operation == "type" {
                        step.text = ai_workspace_type_text(
                            &workspace,
                            step.text.as_deref(),
                            step.workspace_relative_path.as_deref(),
                        )?;
                        ensure_ai_workspace_terminal_text_allowed(
                            &app,
                            &workspace,
                            &app_session_id,
                            step.text.as_deref(),
                        )?;
                        step.workspace_relative_path = None;
                    } else if step.workspace_relative_path.is_some() {
                        return Err(
                            "workspace_relative_path is valid only for AI Workspace type sequence steps."
                                .to_string(),
                        );
                    }
                    serde_json::to_value(step).map_err(|error| {
                        format!("Could not encode AI Workspace sequence step: {error}")
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            let state = app.state::<AppState>();
            state.ai_workspace.sequence_app_session(
                &app,
                &workspace.id,
                &app_session_id,
                params.window_id.as_deref(),
                &steps,
            )
        })
        .await
    }

    #[tool(
        description = "Capture the current isolated AI Workspace screen for visual grounding. This screenshot comes from RepoTunnel's nested virtual display, not the human's real desktop, so the human can keep using other applications while ChatGPT works.",
        annotations(read_only_hint = true)
    )]
    async fn ai_workspace_take_screenshot(
        &self,
        Parameters(params): Parameters<AiWorkspaceFrameParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        const MAX_MCP_SCREENSHOT_BYTES: u64 = 8 * 1024 * 1024;
        let app = self.app.clone();
        let client_key = request_resource_owner_key(&parts, &context);
        let result = tokio::task::spawn_blocking(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let app_session_id = resolve_ai_workspace_app_session(
                &app,
                &workspace,
                client_key.as_deref(),
                params.app_session_id.as_deref(),
            )?;
            let state = app.state::<AppState>();
            let frame = state.ai_workspace.frame_app_session(
                &app,
                &workspace.id,
                &app_session_id,
                params.window_id.as_deref(),
                params.max_width.unwrap_or(1440),
                true,
            )?;
            Ok::<_, String>((app_session_id, frame))
        })
        .await;
        Ok(match result {
            Ok(Ok((app_session_id, frame))) if frame.size_bytes <= MAX_MCP_SCREENSHOT_BYTES && !frame.data_base64.is_empty() => {
                let metadata = serde_json::json!({
                    "ok": true,
                    "result": {
                        "sessionId": frame.session_id,
                        "appSessionId": app_session_id,
                        "mimeType": frame.mime_type.clone(),
                        "sizeBytes": frame.size_bytes,
                        "width": frame.width,
                        "height": frame.height,
                        "sourceWidth": frame.source_width,
                        "sourceHeight": frame.source_height,
                        "activeTitle": frame.active_title,
                    }
                });
                CallToolResult::success(vec![
                    ContentBlock::text(serde_json::to_string(&metadata).unwrap_or_else(|_| "{\"ok\":true}".to_string())),
                    ContentBlock::image(frame.data_base64, frame.mime_type),
                ])
            }
            Ok(Ok((_app_session_id, frame))) => error_result(format!(
                "The AI Workspace screenshot is {} bytes or empty, so RepoTunnel refused to return it through MCP.",
                frame.size_bytes
            )),
            Ok(Err(error)) => error_result(error),
            Err(error) => error_result(format!("AI Workspace screenshot task failed: {error}")),
        })
    }

    #[tool(
        description = "List currently running desktop applications available to project-scoped Desktop Control. Shows whether the single local Desktop permission is enabled for this project and whether each app exposes a Linux accessibility tree. RepoTunnel itself is never included and MCP cannot enable the Desktop permission.",
        annotations(read_only_hint = true)
    )]
    async fn list_desktop_applications(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let mut applications = desktop_control::list(&app, &workspace.id)?;
            applications.sort_by(|left, right| {
                left.name
                    .to_ascii_lowercase()
                    .cmp(&right.name.to_ascii_lowercase())
            });
            Ok(applications)
        })
        .await
    }

    #[tool(
        description = "Inspect the semantic accessibility UI of one running desktop application while the human-enabled project-level Desktop permission is on. Returns stable short-lived element IDs, roles, labels, actions, states and bounds. Sensitive password/credential values are never returned. Inspect again after the UI changes before acting.",
        annotations(read_only_hint = true)
    )]
    async fn inspect_desktop_app(
        &self,
        Parameters(params): Parameters<DesktopInspectParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            if params.application_id == "ai-workspace" {
                return Err("AI Workspace is multi-AI isolated. Use ai_workspace_inspect with your owned app_session_id.".to_string());
            }
            desktop_control::inspect(
                &app,
                &workspace.id,
                &params.application_id,
                params.limit.unwrap_or(300),
            )
        })
        .await
    }

    #[tool(
        description = "Inspect one permitted Linux desktop application through RepoTunnel's existing AT-SPI path and normalize it into the shared semantic snapshot format used by browser semantics. Returns short-lived eN refs, normalized roles/actions/states/bounds, version/hash metadata, and compact unchanged responses. Existing signed AT-SPI element IDs remain internal. Sensitive values/labels are sanitized by the shared semantic core.",
        annotations(read_only_hint = true)
    )]
    async fn desktop_semantic_snapshot(
        &self,
        Parameters(params): Parameters<DesktopSemanticSnapshotParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            if params.application_id == "ai-workspace" {
                return Err(
                    "Use ai_workspace_semantic_snapshot for the isolated AI Workspace semantic tree."
                        .to_string(),
                );
            }
            desktop_control::semantic_snapshot(
                &app,
                &workspace.id,
                &params.application_id,
                params.max_nodes.unwrap_or(300),
                params.known_hash,
            )
        })
        .await
    }

    #[tool(
        description = "Perform a click or type mutation through a short-lived eN ref from desktop_semantic_snapshot. RepoTunnel resolves the ref internally to the existing signed AT-SPI element identity, rechecks the advertised action and sensitive-field policy, and the Linux helper revalidates the signed element again immediately before mutation. Existing desktop_app_action remains the visual/raw-element fallback."
    )]
    async fn desktop_semantic_action(
        &self,
        Parameters(params): Parameters<DesktopSemanticActionParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            if params.application_id == "ai-workspace" {
                return Err(
                    "Use ai_workspace_semantic_action for the isolated AI Workspace.".to_string(),
                );
            }
            let action = match params.action {
                UiSemanticActionParam::Click => "click",
                UiSemanticActionParam::Type => "type",
            };
            desktop_control::semantic_action(
                &app,
                &workspace.id,
                &params.application_id,
                &params.snapshot_id,
                &params.ref_id,
                action,
                params.text.as_deref(),
                params.clear_first.unwrap_or(false),
            )
        })
        .await
    }

    #[tool(
        description = "Run 1..64 already-grounded click/type/wait steps from one desktop_semantic_snapshot in one bounded Linux helper process. This avoids one helper startup per step. Refs and sensitive-field rules are checked before dispatch and signed AT-SPI identities are revalidated at each step. Execution stops on the first failure; the source snapshot is invalidated after dispatch. Existing screenshot/coordinate actions remain the fallback."
    )]
    async fn desktop_semantic_sequence(
        &self,
        Parameters(params): Parameters<DesktopSemanticSequenceParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            if params.application_id == "ai-workspace" {
                return Err(
                    "Use ai_workspace_semantic_sequence for the isolated AI Workspace.".to_string(),
                );
            }
            let steps = params
                .steps
                .into_iter()
                .enumerate()
                .map(|(index, step)| match step.operation {
                    UiSemanticSequenceOperationParam::Click => {
                        let ref_id = step
                            .ref_id
                            .filter(|value| !value.trim().is_empty())
                            .ok_or_else(|| {
                                format!(
                                    "ref_id is required for desktop semantic sequence step {} click.",
                                    index + 1
                                )
                            })?;
                        Ok(desktop_control::DesktopSemanticSequenceStep::Click { ref_id })
                    }
                    UiSemanticSequenceOperationParam::Type => {
                        let ref_id = step
                            .ref_id
                            .filter(|value| !value.trim().is_empty())
                            .ok_or_else(|| {
                                format!(
                                    "ref_id is required for desktop semantic sequence step {} type.",
                                    index + 1
                                )
                            })?;
                        let text = step.text.ok_or_else(|| {
                            format!(
                                "text is required for desktop semantic sequence step {} type.",
                                index + 1
                            )
                        })?;
                        Ok(desktop_control::DesktopSemanticSequenceStep::Type {
                            ref_id,
                            text,
                            clear_first: step.clear_first.unwrap_or(false),
                        })
                    }
                    UiSemanticSequenceOperationParam::Wait => {
                        Ok(desktop_control::DesktopSemanticSequenceStep::Wait {
                            wait_ms: step.wait_ms.unwrap_or(0),
                        })
                    }
                })
                .collect::<Result<Vec<_>, String>>()?;
            desktop_control::semantic_sequence(
                &app,
                &workspace.id,
                &params.application_id,
                &params.snapshot_id,
                &steps,
            )
        })
        .await
    }

    #[tool(
        description = "Perform one bounded UI action inside a desktop application while the human-enabled project-level Desktop permission is on. Supported actions: activate, click, type, key, scroll. Use activate to raise/focus the permitted app window before pointer or keyboard work. Prefer semantic element IDs from inspect_desktop_app. Blind typing is blocked; credential/password fields are blocked; coordinate fallback is window-relative and cannot leave the target app window; RepoTunnel can never control its own UI."
    )]
    async fn desktop_app_action(
        &self,
        Parameters(params): Parameters<DesktopActionParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            if params.application_id == "ai-workspace" {
                return Err("AI Workspace is multi-AI isolated. Use ai_workspace_action with your owned app_session_id.".to_string());
            }
            desktop_control::action(
                &app,
                &workspace.id,
                &params.application_id,
                &params.action,
                params.element_id.as_deref(),
                params.window_id.as_deref(),
                params.text.as_deref(),
                params.clear_first.unwrap_or(false),
                params.shortcut.as_deref(),
                params.x_ratio,
                params.y_ratio,
                params.delta_x,
                params.delta_y,
            )
        })
        .await
    }

    #[tool(
        description = "Capture one current window belonging to a desktop application while the human-enabled project-level Desktop permission is on and return it as PNG image content for visual grounding. The capture is limited to that target application's window, not the whole desktop.",
        annotations(read_only_hint = true)
    )]
    async fn desktop_take_screenshot(
        &self,
        Parameters(params): Parameters<DesktopScreenshotParams>,
    ) -> Result<CallToolResult, McpError> {
        const MAX_MCP_SCREENSHOT_BYTES: u64 = 8 * 1024 * 1024;
        let app = self.app.clone();
        let result = tokio::task::spawn_blocking(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            if params.application_id == "ai-workspace" {
                return Err("AI Workspace is multi-AI isolated. Use ai_workspace_take_screenshot with your owned app_session_id.".to_string());
            }
            desktop_control::screenshot(
                &app,
                &workspace.id,
                &params.application_id,
                params.window_id.as_deref(),
            )
        })
        .await;
        Ok(match result {
            Ok(Ok(screenshot)) if screenshot.size_bytes <= MAX_MCP_SCREENSHOT_BYTES && !screenshot.data_base64.is_empty() => {
                let metadata = serde_json::json!({
                    "ok": true,
                    "result": {
                        "applicationId": screenshot.application_id,
                        "windowId": screenshot.window_id,
                        "mimeType": screenshot.mime_type.clone(),
                        "sizeBytes": screenshot.size_bytes,
                        "width": screenshot.width,
                        "height": screenshot.height,
                    }
                });
                CallToolResult::success(vec![
                    ContentBlock::text(serde_json::to_string(&metadata).unwrap_or_else(|_| "{\"ok\":true}".to_string())),
                    ContentBlock::image(screenshot.data_base64, screenshot.mime_type),
                ])
            }
            Ok(Ok(screenshot)) => error_result(format!(
                "The desktop screenshot is {} bytes or empty, so RepoTunnel refused to return it through MCP.",
                screenshot.size_bytes
            )),
            Ok(Err(message)) => error_result(message),
            Err(error) => error_result(format!("The desktop screenshot task could not complete: {error}")),
        })
    }

    #[tool(
        description = "Launch one structured desktop target for an approved project. kind=url opens an HTTP/HTTPS URL, kind=workspace_path opens a project-relative file/folder, and kind=application launches an allowed application ID. URL/path targets may optionally specify an application ID. In AI Auto the launch happens immediately with no confirmation; in AI Review it may queue for local Accept/Reject."
    )]
    async fn launch_target(
        &self,
        Parameters(params): Parameters<LaunchTargetParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let client_key = request_resource_owner_key(&parts, &context);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let outcome = match params.kind {
                LaunchTargetKindParam::Url => {
                    if gmail_access::is_google_identity_url(&params.target) {
                        gmail_access::require_url_access(&app, &params.target)?;
                        return Err(
                            "Use RepoTunnel managed browser automation for Gmail / Google Sign-In so the shared authenticated profile and AI-session tab isolation are preserved."
                                .to_string(),
                        );
                    }
                    launcher::request_open_url(
                        &app,
                        &workspace,
                        params.target,
                        params.application_id,
                    )
                },
                LaunchTargetKindParam::WorkspacePath => launcher::request_open_workspace_path(
                    &app,
                    &workspace,
                    params.target,
                    params.application_id,
                ),
                LaunchTargetKindParam::Application => {
                    if params.application_id.is_some() {
                        return Err("application_id must be omitted when kind=application; put the application ID in target.".to_string());
                    }
                    launcher::request_launch_application(&app, &workspace, params.target)
                }
            }?;
            if !outcome.queued {
                if let (Some(client_key), Some(pid)) = (client_key.as_deref(), outcome.launch.pid) {
                    ai_resources::own_launched_pid(&workspace.id, client_key, pid);
                }
            }
            let _ = activity::record_launch_record(&app, &workspace, trace_group_id.as_deref(), &outcome.launch);
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "List recent structured application/URL/path launch records, including pending review actions and final launched/failed/rejected state.",
        annotations(read_only_hint = true)
    )]
    async fn list_launch_history(
        &self,
        Parameters(params): Parameters<ListActivityParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            launcher::list_history(
                &app,
                params.workspace_id.as_deref(),
                params.limit.unwrap_or(30),
            )
        })
        .await
    }

    #[tool(
        description = "List installed Chromium-family browsers that support RepoTunnel's isolated Chrome DevTools automation session. Returns stable browser IDs for browser_action action=start.",
        annotations(read_only_hint = true)
    )]
    async fn list_automation_browsers(&self) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            Ok(browser::list_applications())
        })
        .await
    }

    #[tool(
        description = "Get the managed browser-automation session status for an approved project, including running state, browser, PID, session ID, and active tab.",
        annotations(read_only_hint = true)
    )]
    async fn get_browser_status(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            Ok(browser::status(&app, &scope))
        })
        .await
    }

    #[tool(
        description = "List tabs in the RepoTunnel-managed isolated browser session for an approved project, including tab IDs, titles, URLs, and active state.",
        annotations(read_only_hint = true)
    )]
    async fn list_browser_tabs(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            let gmail_enabled = gmail_access::is_enabled(&app)?;
            let mut tabs = browser::list_tabs(&app, &scope)?;
            tabs.retain(|tab| {
                if !gmail_enabled && gmail_access::is_google_identity_url(&tab.url) {
                    return false;
                }
                match client_key.as_deref() {
                    Some(client_key) => {
                        ai_resources::browser_tab_visible(&scope.id, client_key, &tab.id)
                            && ai_resources::claim_or_assert_browser_tab(
                                &scope.id, client_key, &tab.id,
                            )
                            .is_ok()
                    }
                    None => true,
                }
            });
            Ok(tabs)
        })
        .await
    }

    #[tool(
        description = "Read the persistent browser context for an approved workspace. The context contains only RepoTunnel-approved non-secret default headers plus an optional user-agent override.",
        annotations(read_only_hint = true)
    )]
    async fn get_browser_context(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            browser::get_context(&app, &workspace)
        })
        .await
    }

    #[tool(
        description = "Configure the persistent browser context before navigation. Default headers and user-agent are applied to existing tabs, restored after helper reconnects, and applied to new tabs before their first external request. Secret-bearing header names such as Authorization, Cookie, API-key/token/secret/password/credential fields are rejected rather than persisted."
    )]
    async fn configure_browser_context(
        &self,
        Parameters(params): Parameters<BrowserContextParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            browser::configure_context(
                &app,
                &scope,
                &params.name,
                params.default_headers.unwrap_or_default(),
                params.user_agent,
            )
        })
        .await
    }

    #[tool(
        description = "Control RepoTunnel's isolated browser session with one stable action contract: start, stop, open_tab, activate_tab, close_tab, navigate, click, type, scroll, or reload. In AI Auto, navigate is transactional: the same result includes final URL, HTTP status when observed, redirect chain, load state, navigation/document generation IDs, a bounded DOM snapshot only when it belongs to that navigation, request count, cookie-name changes, network failures, duration, and typed timeout/navigation errors. Click/type actions also persist a mutationReceipt with a unique mutation ID, helper acknowledgement, redacted before/after URLs, document generations, and documentChanged evidence. If helper transport is lost after dispatch, the action returns status=ambiguous with the receipt instead of replaying the mutation; inspect current page state before deciding what to do next. In AI Review mutations may queue for local Accept/Reject. Use list_browser_tabs to obtain tab IDs."
    )]
    async fn browser_action(
        &self,
        Parameters(params): Parameters<BrowserActionParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let client_key = request_resource_owner_key(&parts, &context);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            let outcome = match params.action {
                BrowserActionParam::Start => {
                    if let Some(client_key) = client_key.as_deref() {
                        ai_resources::assert_browser_access_available(&scope.id, client_key)?;
                    }
                    let application_id =
                        required_text(params.application_id, "application_id", "start")?;
                    let outcome = browser::request_start(&app, &scope, &application_id)?;
                    if !outcome.queued {
                        if let Some(client_key) = client_key.as_deref() {
                            ai_resources::own_browser_session(&scope.id, client_key);
                            let mut claimed = 0usize;
                            for tab in browser::list_tabs(&app, &scope)? {
                                if ai_resources::claim_or_assert_browser_tab(
                                    &scope.id, client_key, &tab.id,
                                )
                                .is_ok()
                                {
                                    claimed += 1;
                                }
                            }
                            if claimed == 0 {
                                let tab_id = browser::open_window_now(&app, &scope)?;
                                ai_resources::own_browser_tab(&scope.id, client_key, &tab_id);
                            }
                        }
                    }
                    Ok(outcome)
                }
                BrowserActionParam::Stop => {
                    if let Some(client_key) = client_key.as_deref() {
                        ai_resources::assert_browser_stop_available(&scope.id, client_key)?;
                    }
                    let outcome = browser::request_stop(&app, &scope)?;
                    if !outcome.queued {
                        if let Some(client_key) = client_key.as_deref() {
                            ai_resources::release_browser_download_routing(&scope.id, client_key);
                            ai_resources::release_browser_session(&scope.id, client_key);
                            ai_resources::release_all_browser_tabs(&scope.id, client_key);
                        }
                    }
                    Ok(outcome)
                }
                BrowserActionParam::OpenTab => {
                    if let Some(client_key) = client_key.as_deref() {
                        ai_resources::assert_browser_access_available(&scope.id, client_key)?;
                    }
                    let url = required_text(params.url, "url", "open_tab")?;
                    gmail_access::require_url_access(&app, &url)?;
                    let before = browser::list_tabs(&app, &scope)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|tab| tab.id)
                        .collect::<Vec<_>>();
                    let outcome = browser::request_open_tab(&app, &scope, &url)?;
                    if !outcome.queued {
                        if let Some(client_key) = client_key.as_deref() {
                            for tab in browser::list_tabs(&app, &scope)? {
                                if !before.iter().any(|tab_id| tab_id == &tab.id) {
                                    ai_resources::own_browser_tab(&scope.id, client_key, &tab.id);
                                }
                            }
                        }
                    }
                    Ok(outcome)
                }
                BrowserActionParam::ActivateTab => {
                    let tab_id = required_text(params.tab_id, "tab_id", "activate_tab")?;
                    require_ai_browser_tab_access(&scope, client_key.as_deref(), &tab_id)?;
                    require_google_tab_access(&app, &scope, &tab_id)?;
                    browser::request_activate_tab(&app, &scope, &tab_id)
                }
                BrowserActionParam::CloseTab => {
                    let tab_id = required_text(params.tab_id, "tab_id", "close_tab")?;
                    require_ai_browser_tab_access(&scope, client_key.as_deref(), &tab_id)?;
                    let outcome = browser::request_close_tab(&app, &scope, &tab_id)?;
                    if !outcome.queued {
                        if let Some(client_key) = client_key.as_deref() {
                            ai_resources::release_browser_tab(&scope.id, client_key, &tab_id);
                        }
                    }
                    Ok(outcome)
                }
                BrowserActionParam::Navigate => {
                    let tab_id = required_text(params.tab_id, "tab_id", "navigate")?;
                    let url = required_text(params.url, "url", "navigate")?;
                    require_ai_browser_tab_access(&scope, client_key.as_deref(), &tab_id)?;
                    require_google_tab_access(&app, &scope, &tab_id)?;
                    gmail_access::require_url_access(&app, &url)?;
                    browser::request_navigate_with_timeout(
                        &app,
                        &scope,
                        &tab_id,
                        &url,
                        params.timeout_ms.unwrap_or(12_000),
                    )
                }
                BrowserActionParam::Click => {
                    let tab_id = required_text(params.tab_id, "tab_id", "click")?;
                    require_ai_browser_tab_access(&scope, client_key.as_deref(), &tab_id)?;
                    require_google_tab_access(&app, &scope, &tab_id)?;
                    let selector = required_text(params.selector, "selector", "click")?;
                    browser::request_click(&app, &scope, &tab_id, &selector)
                }
                BrowserActionParam::Type => {
                    let tab_id = required_text(params.tab_id, "tab_id", "type")?;
                    require_ai_browser_tab_access(&scope, client_key.as_deref(), &tab_id)?;
                    require_google_tab_access(&app, &scope, &tab_id)?;
                    let selector = required_text(params.selector, "selector", "type")?;
                    let text = params
                        .text
                        .ok_or_else(|| "text is required for browser action type.".to_string())?;
                    browser::request_type(
                        &app,
                        &scope,
                        &tab_id,
                        &selector,
                        &text,
                        params.clear_first.unwrap_or(false),
                    )
                }
                BrowserActionParam::Scroll => {
                    let tab_id = required_text(params.tab_id, "tab_id", "scroll")?;
                    require_ai_browser_tab_access(&scope, client_key.as_deref(), &tab_id)?;
                    require_google_tab_access(&app, &scope, &tab_id)?;
                    let (delta_x, delta_y) = match (params.delta_x, params.delta_y) {
                        (None, None) => (0, 600),
                        (x, y) => (x.unwrap_or(0), y.unwrap_or(0)),
                    };
                    browser::request_scroll(&app, &scope, &tab_id, delta_x, delta_y)
                }
                BrowserActionParam::Reload => {
                    let tab_id = required_text(params.tab_id, "tab_id", "reload")?;
                    require_ai_browser_tab_access(&scope, client_key.as_deref(), &tab_id)?;
                    require_google_tab_access(&app, &scope, &tab_id)?;
                    browser::request_reload(&app, &scope, &tab_id)
                }
            }?;
            let _ = activity::record_browser_record(
                &app,
                &scope,
                trace_group_id.as_deref(),
                &outcome.action,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "Perform a browser mutation through a short-lived semantic ref from browser_semantic_snapshot. Supported actions are click and type. The ref is revalidated inside RepoTunnel; sensitive refs cannot be typed into. Completed mutations persist the same mutationReceipt evidence as selector-based click/type. If helper transport fails after dispatch, RepoTunnel returns status=ambiguous and never automatically replays the mutation; inspect the receipt/current page before continuing. Team browser locks and AI Review apply exactly as they do to selector-based browser_action."
    )]
    async fn browser_semantic_action(
        &self,
        Parameters(params): Parameters<BrowserSemanticActionParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let client_key = request_resource_owner_key(&parts, &context);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            require_ai_browser_tab_access(&scope, client_key.as_deref(), &params.tab_id)?;
            require_google_tab_access(&app, &scope, &params.tab_id)?;
            let outcome = match params.action {
                BrowserSemanticActionParam::Click => browser::request_semantic_click(
                    &app,
                    &scope,
                    &params.tab_id,
                    &params.snapshot_id,
                    &params.ref_id,
                ),
                BrowserSemanticActionParam::Type => {
                    let text = params.text.ok_or_else(|| {
                        "text is required for browser semantic action type.".to_string()
                    })?;
                    browser::request_semantic_type(
                        &app,
                        &scope,
                        &params.tab_id,
                        &params.snapshot_id,
                        &params.ref_id,
                        &text,
                        params.clear_first.unwrap_or(false),
                    )
                }
            }?;
            let _ = activity::record_browser_record(
                &app,
                &scope,
                trace_group_id.as_deref(),
                &outcome.action,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "Run 1..64 already-grounded semantic browser steps in one bounded local request. Supported steps are click, type, and wait. All click/type refs belong to the supplied semantic snapshot. The entire sequence uses one Team browser lock check and one AI Review record; refs and sensitive-field policy are revalidated again when execution actually starts. Successful dispatch persists mutationReceipt evidence. If helper transport fails after dispatch, status=ambiguous is returned and the sequence is never replayed automatically. Execution stops at the first reported failed step and the snapshot is invalidated after dispatch."
    )]
    async fn browser_semantic_sequence(
        &self,
        Parameters(params): Parameters<BrowserSemanticSequenceParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let client_key = request_resource_owner_key(&parts, &context);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            require_ai_browser_tab_access(&scope, client_key.as_deref(), &params.tab_id)?;
            require_google_tab_access(&app, &scope, &params.tab_id)?;
            let steps = params
                .steps
                .into_iter()
                .enumerate()
                .map(|(index, step)| match step.operation {
                    BrowserSemanticSequenceOperationParam::Click => {
                        let ref_id = step
                            .ref_id
                            .filter(|value| !value.trim().is_empty())
                            .ok_or_else(|| {
                                format!(
                                    "ref_id is required for browser semantic sequence step {} click.",
                                    index + 1
                                )
                            })?;
                        Ok(browser::BrowserSemanticSequenceStep::Click { ref_id })
                    }
                    BrowserSemanticSequenceOperationParam::Type => {
                        let ref_id = step
                            .ref_id
                            .filter(|value| !value.trim().is_empty())
                            .ok_or_else(|| {
                                format!(
                                    "ref_id is required for browser semantic sequence step {} type.",
                                    index + 1
                                )
                            })?;
                        let text = step.text.ok_or_else(|| {
                            format!(
                                "text is required for browser semantic sequence step {} type.",
                                index + 1
                            )
                        })?;
                        Ok(browser::BrowserSemanticSequenceStep::Type {
                            ref_id,
                            text,
                            clear_first: step.clear_first.unwrap_or(false),
                        })
                    }
                    BrowserSemanticSequenceOperationParam::Wait => {
                        Ok(browser::BrowserSemanticSequenceStep::Wait {
                            wait_ms: step.wait_ms.unwrap_or(0),
                        })
                    }
                })
                .collect::<Result<Vec<_>, String>>()?;

            let outcome = browser::request_semantic_sequence(
                &app,
                &scope,
                &params.tab_id,
                &params.snapshot_id,
                params.sequence_id.as_deref(),
                steps,
            )?;
            let _ = activity::record_browser_record(
                &app,
                &scope,
                trace_group_id.as_deref(),
                &outcome.action,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "Cancel one currently running browser semantic sequence by its caller-supplied sequence_id. Cancellation is cooperative: it interrupts waits and prevents later steps, but never replays or rolls back a click/type already in flight. Team browser ownership is enforced."
    )]
    async fn browser_semantic_sequence_cancel(
        &self,
        Parameters(params): Parameters<BrowserSemanticSequenceCancelParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            let result = browser::cancel_semantic_sequence(&app, &scope, &params.sequence_id)?;
            record_observation(
                &app,
                &scope,
                trace_group_id.as_deref(),
                ActivityKind::Browser,
                "semanticSequenceCancel",
                format!(
                    "Requested browser semantic sequence cancellation · {}",
                    params.sequence_id
                ),
                None,
            );
            Ok(result)
        })
        .await
    }

    #[tool(
        description = "Route managed-browser downloads into an existing RepoTunnel temporary task. Chrome stores each physical file by its unique download GUID under .repotunnel-tmp/<task>/downloads so the AI always has an exact collision-free path. Progress events include received/total bytes when Chrome knows them. This config is restored if the browser helper reconnects."
    )]
    async fn configure_browser_downloads(
        &self,
        Parameters(params): Parameters<BrowserDownloadConfigureParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        let trace_group_id = request_edit_group_id(&parts);
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            require_ai_browser_tab_access(&scope, client_key.as_deref(), &params.tab_id)?;
            require_google_tab_access(&app, &scope, &params.tab_id)?;
            if let Some(client_key) = client_key.as_deref() {
                ai_resources::claim_browser_download_routing(&scope.id, client_key)?;
            }
            let result =
                match browser::configure_downloads(&app, &scope, &params.tab_id, &params.task_id) {
                    Ok(result) => result,
                    Err(error) => {
                        if let Some(client_key) = client_key.as_deref() {
                            ai_resources::release_browser_download_routing(&scope.id, client_key);
                        }
                        return Err(error);
                    }
                };
            record_observation(
                &app,
                &scope,
                trace_group_id.as_deref(),
                ActivityKind::Browser,
                "configureDownloads",
                format!(
                    "Configured browser downloads · {}",
                    result.relative_directory
                ),
                Some(
                    "Physical download filenames use Chrome download GUIDs to avoid collisions."
                        .to_string(),
                ),
            );
            Ok(result)
        })
        .await
    }

    #[tool(
        description = "List browser downloads known to the current managed browser session. Returns exact workspace-relative physical path, original suggested filename, state, received/total bytes, factual percentage only when total size is known, and transfer speed derived from actual progress samples. resumable=false means RepoTunnel/Chrome does not expose a safe resume primitive.",
        annotations(read_only_hint = true)
    )]
    async fn list_browser_downloads(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            let gmail_enabled = gmail_access::is_enabled(&app)?;
            let mut downloads = browser::list_downloads(&app, &scope)?;
            downloads.retain(|download| {
                let ai_visible = client_key
                    .as_deref()
                    .map(|client_key| {
                        ai_resources::browser_tab_visible(&scope.id, client_key, &download.tab_id)
                    })
                    .unwrap_or(true);
                let google_visible =
                    gmail_enabled || !gmail_access::is_google_identity_url(&download.url);
                ai_visible && google_visible
            });
            Ok(downloads)
        })
        .await
    }

    #[tool(
        description = "Cancel one currently in-progress managed-browser download by its RepoTunnel-known Chrome GUID. This does not delete unrelated files or claim that the download is resumable."
    )]
    async fn cancel_browser_download(
        &self,
        Parameters(params): Parameters<BrowserDownloadCancelParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            let known = browser::list_downloads(&app, &scope)?
                .into_iter()
                .find(|download| download.guid == params.guid)
                .ok_or_else(|| "That browser download is not known to this project.".to_string())?;
            require_ai_browser_tab_access(&scope, client_key.as_deref(), &known.tab_id)?;
            gmail_access::require_url_access(&app, &known.url)?;
            let download = browser::cancel_download(&app, &scope, &params.guid)?;
            record_observation(
                &app,
                &scope,
                trace_group_id.as_deref(),
                ActivityKind::Browser,
                "cancelDownload",
                format!(
                    "Cancelled browser download · {}",
                    download.suggested_filename
                ),
                Some(download.relative_path.clone()),
            );
            Ok(download)
        })
        .await
    }

    #[tool(
        description = "Upload one existing approved workspace file through a web page's <input type=file> element without guessing native file-picker coordinates. RepoTunnel resolves the relative path inside the approved workspace, rejects symlink/non-file targets, and sends the exact path to Chrome through DOM.setFileInputFiles."
    )]
    async fn browser_upload_file(
        &self,
        Parameters(params): Parameters<BrowserUploadParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        let trace_group_id = request_edit_group_id(&parts);
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            require_ai_browser_tab_access(&scope, client_key.as_deref(), &params.tab_id)?;
            require_google_tab_access(&app, &scope, &params.tab_id)?;
            let result = browser::upload_file(
                &app,
                &scope,
                &params.tab_id,
                &params.selector,
                &params.relative_path,
            )?;
            record_observation(
                &app,
                &scope,
                trace_group_id.as_deref(),
                ActivityKind::Browser,
                "uploadFile",
                format!("Selected browser upload file · {}", result.file_name),
                Some(result.relative_path.clone()),
            );
            Ok(result)
        })
        .await
    }

    #[tool(
        description = "Inspect the current DOM/page content of a managed browser tab. Optionally target a CSS selector. Returns bounded text and HTML for reasoning about the rendered application.",
        annotations(read_only_hint = true)
    )]
    async fn browser_inspect_page(
        &self,
        Parameters(params): Parameters<BrowserInspectParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            require_ai_browser_tab_access(&scope, client_key.as_deref(), &params.tab_id)?;
            require_google_tab_access(&app, &scope, &params.tab_id)?;
            browser::inspect_page(
                &app,
                &scope,
                &params.tab_id,
                params.selector.as_deref(),
                params.max_chars.unwrap_or(32_000),
            )
        })
        .await
    }

    #[tool(
        description = "Inspect a managed browser tab through Chrome accessibility semantics. Returns a bounded snapshot with short-lived refs such as e1/e2, roles, labels, states and actions. Pass known_hash from the previous snapshot to receive a compact unchanged response when possible. Sensitive field values are redacted.",
        annotations(read_only_hint = true)
    )]
    async fn browser_semantic_snapshot(
        &self,
        Parameters(params): Parameters<BrowserSemanticSnapshotParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            require_ai_browser_tab_access(&scope, client_key.as_deref(), &params.tab_id)?;
            require_google_tab_access(&app, &scope, &params.tab_id)?;
            let snapshot = browser::semantic_snapshot(
                &app,
                &scope,
                &params.tab_id,
                params.max_nodes.unwrap_or(800),
                params.known_hash,
            )?;
            record_observation(
                &app,
                &scope,
                trace_group_id.as_deref(),
                ActivityKind::Browser,
                "semanticSnapshot",
                format!(
                    "Inspected browser semantics · {} nodes{}",
                    snapshot.nodes.len(),
                    if snapshot.unchanged {
                        " · unchanged"
                    } else {
                        ""
                    }
                ),
                Some(format!(
                    "tab {} · snapshot {} · version {}",
                    snapshot.target_id, snapshot.snapshot_id, snapshot.version
                )),
            );
            Ok(snapshot)
        })
        .await
    }

    #[tool(
        description = "Find elements inside a previously returned browser semantic snapshot without retransmitting the whole accessibility tree. Filters can match free text, role, accessible name, state, and supported action. Returned nodes keep the snapshot's short-lived refs.",
        annotations(read_only_hint = true)
    )]
    async fn browser_semantic_find(
        &self,
        Parameters(params): Parameters<BrowserSemanticFindParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            require_ai_browser_tab_access(&scope, client_key.as_deref(), &params.tab_id)?;
            require_google_tab_access(&app, &scope, &params.tab_id)?;
            let nodes = browser::semantic_find(
                &scope,
                &params.tab_id,
                &params.snapshot_id,
                params.query,
                params.role,
                params.name,
                params.state,
                params.action,
                params.limit.unwrap_or(20),
            )?;
            record_observation(
                &app,
                &scope,
                trace_group_id.as_deref(),
                ActivityKind::Browser,
                "semanticFind",
                format!("Searched browser semantics · {} matches", nodes.len()),
                Some(format!(
                    "tab {} · snapshot {}",
                    params.tab_id, params.snapshot_id
                )),
            );
            Ok(nodes)
        })
        .await
    }

    #[tool(
        description = "Read the most recent element selected by the human from RepoTunnel's Live Preview. Use this before editing when the human says 'change this', 'this button', or otherwise refers to a visually selected UI element.",
        annotations(read_only_hint = true)
    )]
    async fn get_visual_selection(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            browser::get_visual_selection(&scope)
        })
        .await
    }

    #[tool(
        description = "Capture the current managed browser tab as a PNG and return it as MCP image content so the assistant can visually inspect the UI. Set full_page=true only when needed; captures above 8 MiB are refused and should be retried as viewport screenshots.",
        annotations(read_only_hint = true)
    )]
    async fn browser_take_screenshot(
        &self,
        Parameters(params): Parameters<BrowserScreenshotParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let client_key = request_resource_owner_key(&parts, &context);
        run_browser_screenshot_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            require_ai_browser_tab_access(&scope, client_key.as_deref(), &params.tab_id)?;
            require_google_tab_access(&app, &scope, &params.tab_id)?;
            let screenshot = browser::screenshot(
                &app,
                &scope,
                &params.tab_id,
                params.full_page.unwrap_or(false),
            )?;
            record_observation(
                &app,
                &scope,
                trace_group_id.as_deref(),
                ActivityKind::Browser,
                "screenshot",
                format!(
                    "Captured {} screenshot",
                    if screenshot.full_page {
                        "full-page"
                    } else {
                        "viewport"
                    }
                ),
                Some(format!(
                    "{} bytes · tab {}",
                    screenshot.size_bytes, screenshot.tab_id
                )),
            );
            Ok(screenshot)
        })
        .await
    }

    #[tool(
        description = "Read bounded browser network history captured continuously from the managed browser, including successful responses and failures with request ID, URL, method, HTTP status, resource type, MIME type, and timestamp. Sensitive URL fragments are passed through RepoTunnel redaction. Raw Authorization/Cookie headers and response bodies are intentionally not exposed by this tool.",
        annotations(read_only_hint = true)
    )]
    async fn get_browser_network_history(
        &self,
        Parameters(params): Parameters<BrowserDiagnosticsParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            let limit = params.limit.unwrap_or(100).clamp(1, 200);
            if let Some(tab_id) = params.tab_id.as_deref() {
                require_ai_browser_tab_access(&scope, client_key.as_deref(), tab_id)?;
                require_google_tab_access(&app, &scope, tab_id)?;
                return browser::network_history(&app, &scope, Some(tab_id), limit);
            }

            let gmail_enabled = gmail_access::is_enabled(&app)?;
            let tabs = browser::list_tabs(&app, &scope)?;
            let mut entries = Vec::new();
            for tab in tabs {
                if client_key.as_deref().is_some_and(|client_key| {
                    !ai_resources::browser_tab_visible(&scope.id, client_key, &tab.id)
                }) {
                    continue;
                }
                if !gmail_enabled && gmail_access::is_google_identity_url(&tab.url) {
                    continue;
                }
                entries.extend(browser::network_history(
                    &app,
                    &scope,
                    Some(&tab.id),
                    limit,
                )?);
            }
            entries.sort_by_key(|entry| entry.timestamp);
            if entries.len() > limit {
                entries.drain(0..entries.len() - limit);
            }
            Ok(entries)
        })
        .await
    }

    #[tool(
        description = "Read recent console warnings/errors, JavaScript exceptions, failed network requests, and HTTP error responses captured continuously from the managed browser. Optionally filter to one tab.",
        annotations(read_only_hint = true)
    )]
    async fn get_browser_diagnostics(
        &self,
        Parameters(params): Parameters<BrowserDiagnosticsParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let client_key = request_resource_owner_key(&parts, &context);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let scope = browser_scope_for_request(&workspace, client_key.as_deref());
            let limit = params.limit.unwrap_or(50).clamp(1, 200);
            if let Some(tab_id) = params.tab_id.as_deref() {
                require_ai_browser_tab_access(&scope, client_key.as_deref(), tab_id)?;
                require_google_tab_access(&app, &scope, tab_id)?;
                return browser::diagnostics(&app, &scope, Some(tab_id), limit);
            }

            let gmail_enabled = gmail_access::is_enabled(&app)?;
            let tabs = browser::list_tabs(&app, &scope)?;
            let mut console_entries = Vec::new();
            let mut network_failures = Vec::new();
            for tab in tabs {
                if client_key.as_deref().is_some_and(|client_key| {
                    !ai_resources::browser_tab_visible(&scope.id, client_key, &tab.id)
                }) {
                    continue;
                }
                if !gmail_enabled && gmail_access::is_google_identity_url(&tab.url) {
                    continue;
                }
                let diagnostics = browser::diagnostics(&app, &scope, Some(&tab.id), limit)?;
                console_entries.extend(diagnostics.console_entries);
                network_failures.extend(diagnostics.network_failures);
            }
            console_entries.sort_by_key(|entry| entry.timestamp);
            network_failures.sort_by_key(|entry| entry.timestamp);
            if console_entries.len() > limit {
                console_entries.drain(0..console_entries.len() - limit);
            }
            if network_failures.len() > limit {
                network_failures.drain(0..network_failures.len() - limit);
            }
            Ok(crate::models::BrowserDiagnostics {
                console_entries,
                network_failures,
            })
        })
        .await
    }

    #[tool(
        description = "List recent browser-control action records, including queued Review actions and final applied/failed/rejected state. Completed type actions do not retain typed text.",
        annotations(read_only_hint = true)
    )]
    async fn list_browser_history(
        &self,
        Parameters(params): Parameters<ListActivityParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            browser::list_history(
                &app,
                params.workspace_id.as_deref(),
                params.limit.unwrap_or(40),
            )
        })
        .await
    }

    #[tool(
        description = "Inspect an existing approved workspace media file using installed FFprobe only. Returns factual file size, MIME/kind, container format, duration, dimensions, frame rate, and bounded video/audio/subtitle stream metadata. RepoTunnel does not install tools or render anything for this operation.",
        annotations(read_only_hint = true)
    )]
    async fn inspect_media_file(
        &self,
        Parameters(params): Parameters<MediaInspectParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            media_inspection::inspect(&app, &workspace, &params.relative_path)
        })
        .await
    }

    #[tool(
        description = "Extract exactly one PNG frame from an existing workspace media file at a requested timestamp into an existing RepoTunnel temp task under frames/. This uses installed FFmpeg only and is useful for visual QA without decoding/rendering an entire video."
    )]
    async fn extract_media_frame(
        &self,
        Parameters(params): Parameters<MediaFrameParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let result = media_inspection::extract_frame(
                &app,
                &workspace,
                &params.relative_path,
                &params.task_id,
                params.timestamp_seconds,
            )?;
            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Files,
                "extractMediaFrame",
                format!(
                    "Extracted media QA frame · {:.3}s",
                    result.timestamp_seconds
                ),
                Some(result.relative_path.clone()),
            );
            Ok(result)
        })
        .await
    }

    #[tool(
        description = "Decode-validate an existing workspace media file using installed FFmpeg. Low-end-safe default: when full_decode is not true, RepoTunnel validates check_seconds or 30 seconds by default and caps the sample at 600 seconds. Set full_decode=true only for deliberate final verification of the entire file. Returns pass/fail and a bounded error excerpt; it does not repair or render the file.",
        annotations(read_only_hint = true)
    )]
    async fn validate_media_decode(
        &self,
        Parameters(params): Parameters<MediaDecodeParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            if params.full_decode.unwrap_or(false) && params.check_seconds.is_some() {
                return Err("Use either full_decode=true or check_seconds, not both.".to_string());
            }
            let check_seconds = if params.full_decode.unwrap_or(false) {
                None
            } else {
                Some(params.check_seconds.unwrap_or(30.0))
            };
            media_inspection::validate_decode(
                &app,
                &workspace,
                &params.relative_path,
                check_seconds,
            )
        })
        .await
    }

    #[tool(
        description = "Get project monitoring status for an approved workspace. Monitoring tracks filtered project-file changes and feeds the unified process/terminal/port/browser observation snapshot.",
        annotations(read_only_hint = true)
    )]
    async fn get_monitoring_status(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let status = monitoring::status(&app, &workspace);
            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Monitoring,
                "status",
                format!(
                    "Checked project monitor · {}",
                    if status.running { "running" } else { "stopped" }
                ),
                None,
            );
            Ok(status)
        })
        .await
    }

    #[tool(
        description = "Persistently enable or disable read-only project monitoring for an approved workspace. Monitoring never edits project files and does not require a separate Review approval."
    )]
    async fn set_workspace_monitoring(
        &self,
        Parameters(params): Parameters<WorkspaceMonitoringParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let status = if params.enabled {
                monitoring::start_monitoring(&app, &workspace)?
            } else {
                monitoring::stop_monitoring(&app, &workspace)?
            };
            let _ = activity::record(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Monitoring,
                "setMonitoring",
                if params.enabled {
                    "Enabled project monitoring"
                } else {
                    "Disabled project monitoring"
                },
                None,
                ActivityStatus::Succeeded,
                None,
            );
            Ok(status)
        })
        .await
    }

    #[tool(
        description = "Get one combined observation snapshot for an approved workspace: monitoring state, running managed processes with output tails, listening ports/dev-server correlation, recent terminal results, browser tabs/console/network diagnostics, and recent project file changes.",
        annotations(read_only_hint = true)
    )]
    async fn get_monitoring_snapshot(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let snapshot = monitoring::snapshot(&app, &workspace)?;
            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Monitoring,
                "snapshot",
                format!(
                    "Observed {} processes · {} ports · {} file events",
                    snapshot.processes.len(),
                    snapshot.ports.len(),
                    snapshot.file_events.len()
                ),
                Some(format!(
                    "Browser console: {} · network failures: {}",
                    snapshot.browser.console_entries.len(),
                    snapshot.browser.network_failures.len()
                )),
            );
            if !snapshot.file_events.is_empty() {
                record_observation(
                    &app,
                    &workspace,
                    trace_group_id.as_deref(),
                    ActivityKind::Files,
                    "monitoredFileChanges",
                    format!(
                        "Observed {} project file changes",
                        snapshot.file_events.len()
                    ),
                    monitoring_file_detail(&snapshot.file_events),
                );
            }
            Ok(snapshot)
        })
        .await
    }

    #[tool(
        description = "List recent project-file monitoring events (created, modified, deleted). Events are filtered through RepoTunnel's existing project-index/protected-path rules and are observational only.",
        annotations(read_only_hint = true)
    )]
    async fn list_monitoring_file_events(
        &self,
        Parameters(params): Parameters<MonitoringFileEventsParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let workspace = if let Some(workspace_id) = params.workspace_id.as_deref() {
                Some(approved_workspace(&app, workspace_id)?)
            } else {
                None
            };
            let events = monitoring::list_file_events(
                &app,
                params.workspace_id.as_deref(),
                params.limit.unwrap_or(50),
            )?;
            if let Some(workspace) = workspace.as_ref() {
                record_observation(
                    &app,
                    workspace,
                    trace_group_id.as_deref(),
                    ActivityKind::Monitoring,
                    "fileEvents",
                    format!("Inspected project changes · {} events", events.len()),
                    None,
                );
                if !events.is_empty() {
                    record_observation(
                        &app,
                        workspace,
                        trace_group_id.as_deref(),
                        ActivityKind::Files,
                        "monitoredFileChanges",
                        format!("Observed {} project file changes", events.len()),
                        monitoring_file_detail(&events),
                    );
                }
            }
            Ok(events)
        })
        .await
    }

    #[tool(
        description = "Inspect Git status for an approved workspace whose .git directory is inside the workspace root. Returns branch/HEAD, ahead/behind counts, and staged/unstaged/untracked/conflicted paths. This does not modify Git.",
        annotations(read_only_hint = true)
    )]
    async fn git_status(
        &self,
        Parameters(params): Parameters<WorkspaceIdParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let status = git::repository_status(&workspace);
            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Git,
                "status",
                format!(
                    "Checked Git status · {} changed paths",
                    status.changes.len()
                ),
                status
                    .branch
                    .as_ref()
                    .map(|branch| format!("Branch: {branch}")),
            );
            Ok(status)
        })
        .await
    }

    #[tool(
        description = "Inspect a Git diff without unnecessary huge responses. Set staged=true for the index or false for working-tree changes. mode defaults to summary: compact returns counts only, summary returns counts plus up to 100 changed paths, and full additionally returns the bounded patch text. External diff and textconv execution are disabled.",
        annotations(read_only_hint = true)
    )]
    async fn git_diff(
        &self,
        Parameters(params): Parameters<GitDiffParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let mode = params.mode.unwrap_or(GitDiffOutputModeParam::Summary);
            let mut stats = git::diff_stats(&workspace, params.staged)?;
            let (content, content_truncated) = if matches!(mode, GitDiffOutputModeParam::Full) {
                let diff = git::diff(&workspace, params.staged)?;
                (Some(diff.content), diff.truncated)
            } else {
                (None, false)
            };
            if matches!(mode, GitDiffOutputModeParam::Compact) {
                stats.paths.clear();
                stats.paths_truncated = stats.changed_path_count > 0;
            }

            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Git,
                "diff",
                if params.staged {
                    "Inspected staged Git diff"
                } else {
                    "Inspected working-tree Git diff"
                },
                Some(format!(
                    "{} path(s) · +{} -{} · {:?}{}",
                    stats.changed_path_count,
                    stats.insertions,
                    stats.deletions,
                    mode,
                    if content_truncated {
                        " · content truncated"
                    } else {
                        ""
                    }
                )),
            );
            Ok(GitDiffView {
                mode,
                stats,
                content,
                content_truncated,
            })
        })
        .await
    }

    #[tool(
        description = "Run Git's native diff whitespace/conflict-marker check through RepoTunnel's trusted Git subsystem, so verification does not depend on shell access to .git. Set staged=true for the index or false for unstaged working-tree changes. Returns passed plus structured file/line issues.",
        annotations(read_only_hint = true)
    )]
    async fn git_diff_check(
        &self,
        Parameters(params): Parameters<GitDiffParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let check = git::diff_check(&workspace, params.staged)?;
            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Git,
                "diffCheck",
                if check.passed {
                    if params.staged {
                        "Staged Git diff check passed"
                    } else {
                        "Working-tree Git diff check passed"
                    }
                } else if params.staged {
                    "Staged Git diff check found issues"
                } else {
                    "Working-tree Git diff check found issues"
                },
                Some(format!(
                    "{} issue(s){}",
                    check.issue_count,
                    if check.truncated { " · truncated" } else { "" }
                )),
            );
            Ok(check)
        })
        .await
    }

    #[tool(
        description = "List recent local Git commits for an approved repository without exposing author email addresses.",
        annotations(read_only_hint = true)
    )]
    async fn git_log(
        &self,
        Parameters(params): Parameters<GitLogParams>,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let commits = git::recent_commits(&workspace, params.limit.unwrap_or(12))?;
            record_observation(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                ActivityKind::Git,
                "log",
                format!("Inspected Git log · {} commits", commits.len()),
                None,
            );
            Ok(commits)
        })
        .await
    }

    #[tool(
        description = "Stage 1 to 100 explicit workspace-relative files. RepoTunnel blocks protected paths, symlinks, directories, and files that use Git clean filters because filters may execute external programs. In AI Auto the validated staging action applies immediately. In AI Review it is queued for local Accept/Reject and MCP cannot approve it."
    )]
    async fn request_git_stage(
        &self,
        Parameters(params): Parameters<GitStageParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let action = git::request_stage(&app, &workspace, params.paths)?;
            let _ =
                activity::record_git_record(&app, &workspace, trace_group_id.as_deref(), &action);
            Ok(action)
        })
        .await
    }

    #[tool(
        description = "Commit the repository's currently staged changes. RepoTunnel records the exact staged diff and HEAD first. In AI Auto the validated commit applies immediately. In AI Review it is queued for local Accept/Reject and MCP cannot approve it. This tool never stages files automatically."
    )]
    async fn request_git_commit(
        &self,
        Parameters(params): Parameters<GitCommitParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let action = git::request_commit(&app, &workspace, params.message)?;
            let _ =
                activity::record_git_record(&app, &workspace, trace_group_id.as_deref(), &action);
            Ok(crate::mcp_git_helpers::compact_git_action_for_ai(action))
        })
        .await
    }

    #[tool(
        description = "Read RepoTunnel's actual connected GitHub account status through the trusted host-side GitHub broker. Use this instead of inferring GitHub access from a sandboxed shell or AI Workspace environment, which intentionally does not receive the GitHub credential. This tool is read-only and never returns tokens, device codes, verification URLs, or other credentials.",
        annotations(read_only_hint = true)
    )]
    async fn github_connection_status(&self) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            let status = github::status();
            Ok(serde_json::json!({
                "available": status.available,
                "connected": status.connected,
                "connecting": status.connecting,
                "username": status.username,
                "message": status.message,
                "credentialExposed": false,
                "source": "repotunnelTrustedGithubBroker",
                "guidance": if status.connected {
                    "GitHub is connected in RepoTunnel. Use RepoTunnel GitHub/Git tools or a direct approved gh command; do not treat missing sandbox credentials as a disconnected account."
                } else {
                    "GitHub is not currently connected in RepoTunnel."
                }
            }))
        })
        .await
    }

    #[tool(
        description = "Push the current or specified branch to a configured Git remote using RepoTunnel's connected GitHub credential without exposing the token. RepoTunnel runs its committed-file secret preflight before publishing. Defaults to remote=origin, current branch, set_upstream=true. force_with_lease is available for deliberate history replacement."
    )]
    async fn git_push(
        &self,
        Parameters(params): Parameters<GitPushParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let github_status = github::status();
            if !github_status.connected {
                return Err(github_status.message.unwrap_or_else(|| {
                    "GitHub is not connected in RepoTunnel. Connect GitHub and try again."
                        .to_string()
                }));
            }
            git::push(
                &workspace,
                params.remote,
                params.branch,
                params.set_upstream.unwrap_or(true),
                params.force_with_lease.unwrap_or(false),
            )
        })
        .await
    }

    #[tool(
        description = "Create a GitHub pull request for an approved RepoTunnel workspace using the connected GitHub account. The credential stays private; title/body/base/head/draft are passed as structured GitHub CLI arguments."
    )]
    async fn github_create_pr(
        &self,
        Parameters(params): Parameters<GithubCreatePrParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            github::create_pull_request(
                Path::new(&workspace.path),
                params.title,
                params.body,
                params.base,
                params.head,
                params.draft.unwrap_or(false),
            )
        })
        .await
    }

    #[tool(
        description = "Merge a GitHub pull request for an approved RepoTunnel workspace using the connected GitHub account. Supports merge, squash, or rebase plus optional branch deletion, auto-merge, and maintainer/admin merge."
    )]
    async fn github_merge_pr(
        &self,
        Parameters(params): Parameters<GithubMergePrParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        ensure_chatgpt_work_fallback(&app, &params.workspace_id, conversation_session.as_deref());
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            let method = match params.method.unwrap_or(GithubMergeMethodParam::Merge) {
                GithubMergeMethodParam::Merge => "merge",
                GithubMergeMethodParam::Squash => "squash",
                GithubMergeMethodParam::Rebase => "rebase",
            };
            github::merge_pull_request(
                Path::new(&workspace.path),
                params.number,
                method,
                params.delete_branch.unwrap_or(false),
                params.auto.unwrap_or(false),
                params.admin.unwrap_or(false),
            )
        })
        .await
    }

    #[tool(
        description = "Request restoration of one tracked UTF-8 text file to its HEAD version through RepoTunnel's normal safe-editing/history layer. In AI Auto the validated restore applies immediately; in AI Review it waits for local Accept/Reject. Staged or conflicted changes are not silently discarded."
    )]
    async fn request_git_restore_file(
        &self,
        Parameters(params): Parameters<GitRestoreParams>,
        Extension(parts): Extension<Parts>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        let trace_group_id = request_edit_group_id(&parts);
        let conversation_session = openai_conversation_session(&context).map(str::to_string);
        run_filesystem_task(move || {
            let workspace = approved_workspace(&app, &params.workspace_id)?;
            ensure_chatgpt_work_fallback(&app, &workspace.id, conversation_session.as_deref());
            let outcome = git::request_restore_file(
                &app,
                &workspace,
                params.relative_path,
                trace_group_id.as_deref(),
            )?;
            let _ = activity::record_change_outcome(
                &app,
                &workspace,
                trace_group_id.as_deref(),
                &outcome,
            );
            Ok(outcome)
        })
        .await
    }

    #[tool(
        description = "List recent Git commit requests and their pending/applied/rejected/failed status. This tool cannot approve or reject pending Git actions.",
        annotations(read_only_hint = true)
    )]
    async fn list_git_history(
        &self,
        Parameters(params): Parameters<ListGitActionsParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            git::list_actions(
                &app,
                params.workspace_id.as_deref(),
                params.limit.unwrap_or(30),
            )
            .map(|actions| {
                actions
                    .into_iter()
                    .map(compact_git_action_for_ai)
                    .collect::<Vec<_>>()
            })
        })
        .await
    }

    #[tool(
        description = "List recent RepoTunnel change records and their pending/applied/rejected/undone/failed status. This is read-only and does not approve, reject, or undo changes.",
        annotations(read_only_hint = true)
    )]
    async fn list_change_history(
        &self,
        Parameters(params): Parameters<ListChangesParams>,
    ) -> Result<CallToolResult, McpError> {
        let app = self.app.clone();
        run_filesystem_task(move || {
            ensure_ai_access(&app)?;
            changes::list_changes(
                &app,
                params.workspace_id.as_deref(),
                params.limit.unwrap_or(30),
            )
        })
        .await
    }
}

fn tool_meta(resource_uri: Option<&str>, visibility: &[&str]) -> MetaObject {
    let mut meta = MetaObject::new();
    let mut ui = serde_json::Map::new();
    ui.insert("visibility".to_string(), serde_json::json!(visibility));
    if let Some(resource_uri) = resource_uri {
        ui.insert("resourceUri".to_string(), serde_json::json!(resource_uri));
        // Keep both compatibility aliases because some MCP Apps hosts still
        // inspect the historical flat key while ChatGPT also supports
        // openai/outputTemplate.
        meta.insert(
            "ui/resourceUri".to_string(),
            serde_json::json!(resource_uri),
        );
        meta.insert(
            "openai/outputTemplate".to_string(),
            serde_json::json!(resource_uri),
        );
    }
    meta.insert("ui".to_string(), serde_json::Value::Object(ui));
    meta.insert(
        "openai/widgetAccessible".to_string(),
        serde_json::json!(visibility.contains(&"app")),
    );
    meta
}

// The primary router remains the only router used by the MCP handler.
impl RepoTunnelMcp {
    fn tool_router() -> ToolRouter<Self> {
        let mut router = Self::base_tool_router();

        // The Chrome extension bridge is now the only model-facing continuation
        // path. Keep the legacy MCP-App implementation compiled for compatibility
        // with already-mounted old widgets, but do not advertise or accept the
        // model-facing arm/mount/update/heartbeat entry points in new sessions.
        for name in [
            "arm_self_continuation",
            "mount_self_continuation_app",
            "update_self_continuation",
            "heartbeat_self_continuation",
        ] {
            router.map.remove(name);
        }

        for name in [
            "attempt_self_continuation_recovery",
            "poll_self_continuation",
            "claim_self_continuation_recovery",
            "ack_self_continuation_recovery",
        ] {
            if let Some(route) = router.map.get_mut(name) {
                route.attr.meta = Some(tool_meta(None, &["app"]));
            }
        }
        router
    }
}

fn repotunnel_server_capabilities() -> ServerCapabilities {
    let mut extensions = ExtensionCapabilities::new();
    extensions.insert(
        "io.modelcontextprotocol/ui".to_string(),
        serde_json::from_value(serde_json::json!({
            "mimeTypes": ["text/html;profile=mcp-app"]
        }))
        .expect("static MCP Apps capability must be a JSON object"),
    );

    ServerCapabilities::builder()
        .enable_extensions_with(extensions)
        .enable_tools()
        .enable_tool_list_changed()
        .enable_resources()
        .enable_resources_list_changed()
        .build()
}

#[tool_handler]
impl ServerHandler for RepoTunnelMcp {
    async fn on_initialized(&self, context: NotificationContext<RoleServer>) {
        let _ = context.peer.notify_tool_list_changed().await;
        let _ = context.peer.notify_resource_list_changed().await;
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(repotunnel_server_capabilities())
            .with_server_info(Implementation::new("repotunnel", env!("CARGO_PKG_VERSION")))
            .with_instructions(SERVER_INSTRUCTIONS)
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        SELF_CONTINUATION_RESOURCE_LIST_COUNT.fetch_add(1, Ordering::Relaxed);

        Ok(ListResourcesResult::with_all_items(vec![Resource::new(
            SELF_CONTINUATION_RESOURCE_URI,
            "repotunnel-self-continuation-v8",
        )
        .with_title("RepoTunnel self-continuation")
        .with_description(
            "Tiny background continuation bridge. Older resource URIs remain readable for cache compatibility but are no longer advertised.",
        )
        .with_mime_type("text/html;profile=mcp-app")
        .with_meta(self_continuation_resource_meta())]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        if !is_self_continuation_resource_uri(&request.uri) {
            return Err(McpError::resource_not_found(
                "RepoTunnel MCP App resource was not found.",
                None,
            ));
        }

        SELF_CONTINUATION_RESOURCE_READ_COUNT.fetch_add(1, Ordering::Relaxed);
        if request.uri == SELF_CONTINUATION_RESOURCE_URI {
            SELF_CONTINUATION_CURRENT_RESOURCE_READ_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        SELF_CONTINUATION_LAST_RESOURCE_READ_AT.store(crate::mcp_time::unix_epoch_millis(), Ordering::Relaxed);

        let response_uri = request.uri.clone();

        Ok(ReadResourceResult::new(vec![ResourceContents::text(
            SELF_CONTINUATION_APP_HTML,
            response_uri,
        )
        .with_mime_type("text/html;profile=mcp-app")
        .with_meta(self_continuation_resource_meta())])
        .into())
    }
}

#[cfg(test)]
mod lean_gateway_video_tests {
    use super::RepoTunnelMcp;

    #[test]
    fn video_tools_are_not_discoverable_or_dispatchable() {
        let router = RepoTunnelMcp::tool_router();
        let forbidden = [
            "start_video_analysis",
            "get_video_analysis",
            "cancel_video_analysis",
            "create_video_project",
            "compile_video_story_plan",
            "render_video_story_animatic",
            "start_video_project_render",
            "render_video_project_timeline",
            "synthesize_video_project_narration",
        ];

        for name in forbidden {
            assert!(
                !router.map.contains_key(name),
                "legacy video MCP route should be absent in lean mode: {name}"
            );
        }

        assert!(
            router.map.contains_key("capabilities"),
            "core MCP tools must remain available"
        );
    }
}

#[cfg(test)]
mod task_outcome_tests {
    use serde::{ser::Error as _, Serialize, Serializer};

    use super::serialize_success_payload;

    struct CannotSerialize;

    impl Serialize for CannotSerialize {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(S::Error::custom("intentional serialization failure"))
        }
    }

    #[test]
    fn successful_payload_preserves_result_envelope() {
        let serialized = serialize_success_payload(42_u32).expect("serializable result");
        let parsed: serde_json::Value =
            serde_json::from_str(&serialized).expect("valid JSON");
        assert_eq!(parsed["ok"], true);
        assert_eq!(parsed["result"], 42);
    }

    #[test]
    fn serialization_failure_is_reported_to_caller() {
        assert!(serialize_success_payload(CannotSerialize).is_err());
    }
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderMap;

    use crate::models::{GitActionKind, GitActionRecord, GitActionStatus};
    use crate::mcp_request_identity::trace_edit_group_id;
    use crate::mcp_ai_workspace_helpers::ai_workspace_terminal_browser_launch;

    use super::{
        compact_git_action_for_ai,
        continuation_identity_keys_from, is_self_continuation_resource_uri,
        repotunnel_server_capabilities, self_continuation_resource_meta,
        RepoTunnelMcp, LEGACY_SELF_CONTINUATION_RESOURCE_URI,
        LEGACY_V1_SELF_CONTINUATION_RESOURCE_URI, LEGACY_V2_SELF_CONTINUATION_RESOURCE_URI,
        LEGACY_V5_SELF_CONTINUATION_RESOURCE_URI, LEGACY_V6_SELF_CONTINUATION_RESOURCE_URI,
        LEGACY_V7_SELF_CONTINUATION_RESOURCE_URI, PREVIOUS_SELF_CONTINUATION_RESOURCE_URI,
        SELF_CONTINUATION_APP_HTML, SELF_CONTINUATION_RESOURCE_URI,
    };

    #[test]
    fn continuation_identity_requires_openai_conversation_and_never_uses_shared_mcp_session() {
        let keys = continuation_identity_keys_from(Some("conversation-123"));
        assert_eq!(keys, vec!["openai-session:conversation-123".to_string()]);

        let no_conversation = continuation_identity_keys_from(None);
        assert!(no_conversation.is_empty());
    }

    #[test]
    fn derives_edit_group_from_traceparent() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "traceparent",
            "00-6a834f2e00000000a27259e7454a1be9-0123456789abcdef-00"
                .parse()
                .expect("valid header"),
        );

        assert_eq!(
            trace_edit_group_id(&headers).as_deref(),
            Some("trace-6a834f2e00000000a27259e7454a1be9")
        );
    }

    #[test]
    fn rejects_malformed_traceparent_for_grouping() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "traceparent",
            "00-00000000000000000000000000000000-0123456789abcdef-00"
                .parse()
                .expect("valid header"),
        );

        assert_eq!(trace_edit_group_id(&headers), None);
    }

    #[test]
    fn ai_workspace_terminal_blocks_direct_browser_launches_but_not_normal_commands() {
        assert!(ai_workspace_terminal_browser_launch(
            "/tmp/test/chrome --user-data-dir=/tmp/profile"
        ));
        assert!(ai_workspace_terminal_browser_launch(
            "nohup google-chrome-stable --headless &"
        ));
        assert!(ai_workspace_terminal_browser_launch(
            "echo done && chromium --version"
        ));
        assert!(!ai_workspace_terminal_browser_launch("npm test"));
        assert!(!ai_workspace_terminal_browser_launch("cargo test --locked"));
        assert!(!ai_workspace_terminal_browser_launch(
            "echo chrome should stay plain text"
        ));
    }

    #[test]
    fn commit_git_action_details_are_omitted_only_from_ai_responses() {
        let base = GitActionRecord {
            id: "git-test".to_string(),
            workspace_id: "workspace-test".to_string(),
            workspace_name: "Test".to_string(),
            kind: GitActionKind::Commit,
            summary: "Commit staged changes".to_string(),
            detail: Some("very large staged diff".to_string()),
            status: GitActionStatus::Pending,
            created_at: 1,
            updated_at: 1,
            commit_hash: None,
            error: None,
        };

        let compact = compact_git_action_for_ai(base.clone());
        assert_eq!(compact.detail, None);
        assert_eq!(base.detail.as_deref(), Some("very large staged diff"));

        let mut stage = base;
        stage.kind = GitActionKind::Stage;
        stage.detail = Some("src/main.rs".to_string());
        assert_eq!(
            compact_git_action_for_ai(stage).detail.as_deref(),
            Some("src/main.rs")
        );
    }

    #[test]
    fn runtime_and_browser_v2_tools_are_exposed() {
        let tools = RepoTunnelMcp::tool_router().list_all();
        for expected in [
            "capabilities",
            "get_workspace_runtime_status",
            "get_browser_context",
            "configure_browser_context",
            "get_browser_network_history",
            "browser_action",
            "github_connection_status",
            "git_push",
            "github_create_pr",
            "github_merge_pr",
            "list_chatgpt_extension_targets",
            "queue_chatgpt_extension_message",
            "list_chatgpt_extension_jobs",
            "cancel_chatgpt_extension_job",
            "complete_chatgpt_extension_work",
        ] {
            assert!(
                tools.iter().any(|tool| tool.name.as_ref() == expected),
                "missing MCP tool: {expected}"
            );
        }
    }

    #[test]
    fn android_phone_tools_are_not_exposed() {
        let tools = RepoTunnelMcp::tool_router().list_all();
        assert!(tools.iter().all(|tool| !tool.name.starts_with("phone_")));
    }

    #[test]
    fn ai_workspace_schema_and_tool_refresh_are_advertised() {
        let capabilities = repotunnel_server_capabilities();
        assert_eq!(
            capabilities
                .tools
                .as_ref()
                .and_then(|tools| tools.list_changed),
            Some(true)
        );
        assert_eq!(
            capabilities
                .resources
                .as_ref()
                .and_then(|resources| resources.list_changed),
            Some(true)
        );

        let tools = RepoTunnelMcp::tool_router().list_all();
        let tool = tools
            .iter()
            .find(|tool| tool.name.as_ref() == "ai_workspace_session")
            .expect("ai_workspace_session tool");
        let value = serde_json::to_value(tool).expect("serialize ai_workspace_session schema");
        let properties = value
            .pointer("/inputSchema/properties")
            .or_else(|| value.pointer("/input_schema/properties"))
            .and_then(serde_json::Value::as_object)
            .expect("ai_workspace_session input properties");
        assert!(
            properties.contains_key("new_instance"),
            "ai_workspace_session must expose new_instance to MCP clients"
        );
        assert!(
            properties.contains_key("app_session_id"),
            "ai_workspace_session must expose app_session_id to MCP clients"
        );
    }

    #[test]
    fn server_negotiates_standard_mcp_apps_ui_extension() {
        let capabilities = repotunnel_server_capabilities();
        let extensions = capabilities.extensions.expect("MCP extension capabilities");
        let ui = extensions
            .get("io.modelcontextprotocol/ui")
            .expect("MCP Apps UI extension");
        assert_eq!(
            ui.get("mimeTypes"),
            Some(&serde_json::json!(["text/html;profile=mcp-app"]))
        );
    }

    #[test]
    fn self_continuation_tools_use_minimal_mcp_app_visibility() {
        assert_eq!(
            SELF_CONTINUATION_RESOURCE_URI,
            "ui://widget/repotunnel-self-continuation-v8.html"
        );
        assert!(is_self_continuation_resource_uri(
            SELF_CONTINUATION_RESOURCE_URI
        ));
        assert!(is_self_continuation_resource_uri(
            LEGACY_V7_SELF_CONTINUATION_RESOURCE_URI
        ));
        assert!(is_self_continuation_resource_uri(
            LEGACY_V6_SELF_CONTINUATION_RESOURCE_URI
        ));
        assert!(is_self_continuation_resource_uri(
            LEGACY_V5_SELF_CONTINUATION_RESOURCE_URI
        ));
        assert!(is_self_continuation_resource_uri(
            PREVIOUS_SELF_CONTINUATION_RESOURCE_URI
        ));
        assert!(is_self_continuation_resource_uri(
            LEGACY_SELF_CONTINUATION_RESOURCE_URI
        ));
        assert!(is_self_continuation_resource_uri(
            LEGACY_V2_SELF_CONTINUATION_RESOURCE_URI
        ));
        assert!(is_self_continuation_resource_uri(
            LEGACY_V1_SELF_CONTINUATION_RESOURCE_URI
        ));
        assert!(!is_self_continuation_resource_uri(
            "ui://repotunnel/self-continuation/unexpected.html"
        ));
        let tools = RepoTunnelMcp::tool_router().list_all();
        let find = |name: &str| tools.iter().find(|tool| tool.name.as_ref() == name);

        for name in [
            "arm_self_continuation",
            "mount_self_continuation_app",
            "update_self_continuation",
            "heartbeat_self_continuation",
        ] {
            assert!(
                find(name).is_none(),
                "legacy model-facing continuation tool must stay hidden: {name}"
            );
        }

        for name in [
            "list_chatgpt_extension_targets",
            "queue_chatgpt_extension_message",
            "list_chatgpt_extension_jobs",
            "cancel_chatgpt_extension_job",
            "complete_chatgpt_extension_work",
        ] {
            assert!(
                find(name).is_some(),
                "extension continuation tool must remain advertised: {name}"
            );
        }

        for name in [
            "attempt_self_continuation_recovery",
            "poll_self_continuation",
            "claim_self_continuation_recovery",
            "ack_self_continuation_recovery",
        ] {
            let tool = find(name).unwrap_or_else(|| panic!("missing app-only MCP tool: {name}"));
            let meta = tool.meta.as_ref().expect("app-only metadata");
            let ui = meta
                .get("ui")
                .and_then(serde_json::Value::as_object)
                .expect("app-only ui metadata");
            assert_eq!(ui.get("visibility"), Some(&serde_json::json!(["app"])));
            assert!(!ui.contains_key("resourceUri"));
        }

        assert!(SELF_CONTINUATION_APP_HTML.contains("window.openai"));
        assert!(SELF_CONTINUATION_APP_HTML.contains("callTool"));
        assert!(SELF_CONTINUATION_APP_HTML.contains("sendFollowUpMessage"));
        assert!(SELF_CONTINUATION_APP_HTML.contains("\"ui/initialize\""));
        assert!(SELF_CONTINUATION_APP_HTML.contains("\"ui/notifications/size-changed\""));
        assert!(SELF_CONTINUATION_APP_HTML.contains("\"ui/message\""));
        assert!(SELF_CONTINUATION_APP_HTML.contains("\"tools/call\""));
        assert!(SELF_CONTINUATION_APP_HTML.contains("width: 1"));
        assert!(SELF_CONTINUATION_APP_HTML.contains("height: 1"));
        assert!(SELF_CONTINUATION_APP_HTML.contains("attempt_self_continuation_recovery"));
        assert!(!SELF_CONTINUATION_APP_HTML.contains("poll_self_continuation"));
        assert!(!SELF_CONTINUATION_APP_HTML.contains("claim_self_continuation_recovery"));
        assert!(SELF_CONTINUATION_APP_HTML.contains("ack_self_continuation_recovery"));
        assert!(!SELF_CONTINUATION_APP_HTML.contains("setInterval("));
        assert!(!SELF_CONTINUATION_APP_HTML.contains("POLL_MS"));
        assert!(!SELF_CONTINUATION_APP_HTML.contains("prompt: \"continue\""));

        let resource_meta = self_continuation_resource_meta();
        let standard_ui = resource_meta
            .get("ui")
            .and_then(serde_json::Value::as_object)
            .expect("standard MCP Apps resource metadata");
        assert_eq!(
            standard_ui.get("prefersBorder"),
            Some(&serde_json::json!(false))
        );
        let standard_csp = standard_ui
            .get("csp")
            .and_then(serde_json::Value::as_object)
            .expect("standard MCP Apps CSP");
        assert_eq!(
            standard_csp.get("connectDomains"),
            Some(&serde_json::json!([]))
        );
        assert_eq!(
            standard_csp.get("resourceDomains"),
            Some(&serde_json::json!([]))
        );
        assert_eq!(
            resource_meta.get("openai/widgetPrefersBorder"),
            Some(&serde_json::json!(false))
        );
        let legacy_csp = resource_meta
            .get("openai/widgetCSP")
            .and_then(serde_json::Value::as_object)
            .expect("legacy ChatGPT CSP compatibility metadata");
        assert_eq!(
            legacy_csp.get("connect_domains"),
            Some(&serde_json::json!([]))
        );
        assert_eq!(
            legacy_csp.get("resource_domains"),
            Some(&serde_json::json!([]))
        );

        for name in [
            "attempt_self_continuation_recovery",
            "poll_self_continuation",
            "claim_self_continuation_recovery",
            "ack_self_continuation_recovery",
        ] {
            let tool = find(name).unwrap_or_else(|| panic!("missing app-only MCP tool: {name}"));
            let annotations = tool.annotations.as_ref().expect("continuation annotations");
            assert_eq!(annotations.read_only_hint, Some(false), "{name}");
            assert_eq!(annotations.destructive_hint, Some(false), "{name}");
            assert_eq!(annotations.open_world_hint, Some(false), "{name}");
        }
        assert_eq!(
            find("poll_self_continuation")
                .and_then(|tool| tool.annotations.as_ref())
                .and_then(|value| value.idempotent_hint),
            Some(true)
        );
        assert_eq!(
            find("ack_self_continuation_recovery")
                .and_then(|tool| tool.annotations.as_ref())
                .and_then(|value| value.idempotent_hint),
            Some(true)
        );
    }

    #[test]
    fn large_project_fast_read_tools_are_exposed() {
        let tools = RepoTunnelMcp::tool_router().list_all();
        for expected in [
            "inspect_project_page",
            "fast_search_files",
            "read_file_range",
            "list_directory_page",
            "inspect_project",
            "search_files",
            "read_file",
            "list_directory",
        ] {
            let tool = tools
                .iter()
                .find(|tool| tool.name.as_ref() == expected)
                .unwrap_or_else(|| panic!("missing MCP tool: {expected}"));
            let annotations = tool.annotations.as_ref().expect("read-only annotations");
            assert_eq!(annotations.read_only_hint, Some(true), "{expected}");
        }
    }

    #[test]
    fn project_memory_update_declares_closed_world_safety_metadata() {
        let tools = RepoTunnelMcp::tool_router().list_all();
        let tool = tools
            .iter()
            .find(|tool| tool.name.as_ref() == "update_project_memory")
            .expect("project-memory update tool");
        let annotations = tool.annotations.as_ref().expect("tool annotations");

        assert_eq!(annotations.read_only_hint, Some(false));
        assert_eq!(annotations.destructive_hint, Some(true));
        assert_eq!(annotations.idempotent_hint, Some(false));
        assert_eq!(annotations.open_world_hint, Some(false));
    }
}
