#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct AiWorkspaceFrameParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Per-application appSessionId. Omit only when this AI owns exactly one app session.
    pub(crate) app_session_id: Option<String>,
    /// Optional app-owned window ID. When omitted RepoTunnel selects the active/first window owned by the app session.
    pub(crate) window_id: Option<String>,
    /// Maximum returned image width. RepoTunnel clamps this to the virtual screen width.
    pub(crate) max_width: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct AiWorkspaceInspectParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Per-application appSessionId. Omit only when this AI owns exactly one app session.
    pub(crate) app_session_id: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct AiWorkspaceSemanticSnapshotParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Per-application appSessionId. Required when multiple app sessions share the desktop.
    pub(crate) app_session_id: Option<String>,
    /// Maximum private AT-SPI nodes to normalize into the shared semantic snapshot. Clamped to 20..800.
    pub(crate) max_nodes: Option<usize>,
    /// Hash from a previous AI Workspace semantic snapshot. When unchanged, RepoTunnel can return a compact response.
    pub(crate) known_hash: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct AiWorkspaceActionParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Per-application appSessionId. Omit only when this AI owns exactly one app session.
    pub(crate) app_session_id: Option<String>,
    /// One of activate, click, key, type, or scroll.
    pub(crate) action: String,
    /// Optional AI Workspace window ID returned by ai_workspace_inspect. When supplied, click/scroll coordinates are relative to that exact isolated window.
    pub(crate) window_id: Option<String>,
    /// Horizontal position from 0..1 for click/scroll inside the isolated virtual display or selected window.
    pub(crate) x_ratio: Option<f64>,
    /// Vertical position from 0..1 for click/scroll inside the isolated virtual display.
    pub(crate) y_ratio: Option<f64>,
    /// Click count 1..3 for action=click.
    pub(crate) click_count: Option<u8>,
    /// Safe shortcut for action=key, such as Ctrl+S, Enter, Escape, or F5.
    pub(crate) shortcut: Option<String>,
    /// Text for action=type. Credential/authentication windows are blocked.
    pub(crate) text: Option<String>,
    /// For action=type, resolve this workspace-relative file/folder to the exact host path visible inside the isolated app. Mutually exclusive with text.
    pub(crate) workspace_relative_path: Option<String>,
    /// Horizontal scroll delta.
    pub(crate) delta_x: Option<i32>,
    /// Vertical scroll delta.
    pub(crate) delta_y: Option<i32>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AiWorkspaceSequenceStep {
    /// One of activate, click, key, type, scroll, or wait.
    pub(crate) operation: String,
    /// Optional per-step window override. Omit to inherit the sequence window_id.
    pub(crate) window_id: Option<String>,
    /// Horizontal position from 0..1 for click/scroll.
    pub(crate) x_ratio: Option<f64>,
    /// Vertical position from 0..1 for click/scroll.
    pub(crate) y_ratio: Option<f64>,
    /// Click count 1..3 for click.
    pub(crate) count: Option<u8>,
    /// Safe shortcut for key.
    pub(crate) shortcut: Option<String>,
    /// Text for type. Text is never echoed in completed sequence results.
    pub(crate) text: Option<String>,
    /// For type, resolve this workspace-relative file/folder to the exact host path visible inside the isolated app. Mutually exclusive with text.
    pub(crate) workspace_relative_path: Option<String>,
    /// Horizontal scroll delta.
    pub(crate) delta_x: Option<i32>,
    /// Vertical scroll delta.
    pub(crate) delta_y: Option<i32>,
    /// Optional bounded delay for wait, clamped to 0..2000 ms.
    pub(crate) wait_ms: Option<u64>,
    /// Optional wait-condition timeout, clamped to 0..5000 ms.
    pub(crate) timeout_ms: Option<u64>,
    /// Wait until the active isolated window title contains this text.
    pub(crate) title_contains: Option<String>,
    /// Wait until at least this many isolated windows exist.
    pub(crate) window_count_at_least: Option<usize>,
    /// Wait until no more than this many isolated windows exist.
    pub(crate) window_count_at_most: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct AiWorkspaceSequenceParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Per-application appSessionId. Omit only when this AI owns exactly one app session.
    pub(crate) app_session_id: Option<String>,
    /// Optional default isolated window ID inherited by steps that omit windowId.
    pub(crate) window_id: Option<String>,
    /// Ordered fast-path actions. RepoTunnel accepts 1..64 bounded steps per request.
    pub(crate) steps: Vec<AiWorkspaceSequenceStep>,
}

