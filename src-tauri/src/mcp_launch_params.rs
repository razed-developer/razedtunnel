#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LaunchTargetKindParam {
    Url,
    WorkspacePath,
    Application,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct LaunchTargetParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Target type: url, workspace_path, or application.
    pub(crate) kind: LaunchTargetKindParam,
    /// URL, workspace-relative path, or application ID depending on kind. Use an empty string for the project root when kind=workspace_path.
    pub(crate) target: String,
    /// Optional application ID returned by list_launchable_applications when opening a URL or workspace path with a specific app. Omit to use the desktop default.
    pub(crate) application_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct IntegrationActionParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// One of: android-studio, unity, blender, godot, docker.
    pub(crate) integration_id: String,
    /// Exact allowlisted action returned by list_deep_integrations for this integration.
    pub(crate) action: String,
    /// Optional workspace-relative target. Android Studio accepts a project folder; Blender run_script/render accepts the required file.
    pub(crate) target: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct AiWorkspaceSessionParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// One of status, start, reclaim, or stop.
    pub(crate) action: String,
    /// Application ID returned by list_launchable_applications. Required only for action=start.
    pub(crate) application_id: Option<String>,
    /// Optional workspace-relative project file or folder opened inside the isolated app session. Productivity files can be opened directly in a detected Word/Excel/PowerPoint, Writer/Calc/Impress, or Pages/Numbers/Keynote app.
    pub(crate) target: Option<String>,
    /// Durable shared-desktop session_id returned by start/status. Retained for backward-compatible stale desktop recovery.
    pub(crate) session_id: Option<String>,
    /// Per-application appSessionId returned in status.applications and as lastStartedAppSessionId. Use this to address one AI's app without affecting other apps on the shared desktop.
    pub(crate) app_session_id: Option<String>,
    /// For action=start only: explicitly request a separate additional instance even when the same application is already running. Defaults false so repeated/reconnected AI calls reuse or stop safely instead of spawning duplicate Terminals/editors.
    pub(crate) new_instance: Option<bool>,
    /// For action=stop, allow cleanup only when a different owner is stale. Requires the matching app_session_id for multi-app sessions.
    pub(crate) stale_only: Option<bool>,
}

