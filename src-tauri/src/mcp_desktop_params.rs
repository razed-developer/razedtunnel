#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct DesktopInspectParams {
    /// Approved project whose local desktop permission grants apply.
    pub(crate) workspace_id: String,
    /// Running desktop application ID returned by list_desktop_applications.
    pub(crate) application_id: String,
    /// Maximum semantic UI elements to return. RepoTunnel clamps this to 20..800.
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct DesktopSemanticSnapshotParams {
    /// Approved project whose local Desktop permission grants apply.
    pub(crate) workspace_id: String,
    /// Running desktop application ID returned by list_desktop_applications.
    pub(crate) application_id: String,
    /// Maximum AT-SPI elements to normalize into the shared semantic snapshot. Clamped to 20..800.
    pub(crate) max_nodes: Option<usize>,
    /// Hash from a previous semantic snapshot. When unchanged, RepoTunnel can return a compact response.
    pub(crate) known_hash: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum UiSemanticActionParam {
    Click,
    Type,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct DesktopSemanticActionParams {
    /// Approved project whose local Desktop permission grants apply.
    pub(crate) workspace_id: String,
    /// Running desktop application ID that owns the semantic snapshot/ref.
    pub(crate) application_id: String,
    /// Snapshot ID returned by desktop_semantic_snapshot.
    pub(crate) snapshot_id: String,
    /// Short-lived semantic ref such as e1.
    pub(crate) ref_id: String,
    /// Semantic mutation to perform.
    pub(crate) action: UiSemanticActionParam,
    /// Text to enter. Required only for action=type.
    pub(crate) text: Option<String>,
    /// For action=type, replace the existing field contents. Defaults to false.
    pub(crate) clear_first: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct AiWorkspaceSemanticActionParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Per-application appSessionId. Required when multiple app sessions share the desktop.
    pub(crate) app_session_id: Option<String>,
    /// Snapshot ID returned by ai_workspace_semantic_snapshot.
    pub(crate) snapshot_id: String,
    /// Short-lived semantic ref such as e1.
    pub(crate) ref_id: String,
    /// Semantic mutation to perform.
    pub(crate) action: UiSemanticActionParam,
    /// Text to enter. Required only for action=type.
    pub(crate) text: Option<String>,
    /// For action=type, resolve this workspace-relative file/folder to the exact host path visible inside the isolated app. Mutually exclusive with text.
    pub(crate) workspace_relative_path: Option<String>,
    /// For action=type, replace the existing field contents. Defaults to false.
    pub(crate) clear_first: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum UiSemanticSequenceOperationParam {
    Click,
    Type,
    Wait,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct UiSemanticSequenceStepParam {
    /// One of click, type, or wait.
    pub(crate) operation: UiSemanticSequenceOperationParam,
    /// Short-lived semantic ref. Required for click and type.
    pub(crate) ref_id: Option<String>,
    /// Text to enter. Required only for type.
    pub(crate) text: Option<String>,
    /// For AI Workspace type steps, resolve this workspace-relative file/folder to the exact host path visible inside the isolated app. Mutually exclusive with text.
    pub(crate) workspace_relative_path: Option<String>,
    /// For type, replace the existing field contents. Defaults to false.
    pub(crate) clear_first: Option<bool>,
    /// Bounded delay for wait. Maximum 2000 ms per step and 10000 ms total.
    pub(crate) wait_ms: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct DesktopSemanticSequenceParams {
    /// Approved project whose local Desktop permission grants apply.
    pub(crate) workspace_id: String,
    /// Running desktop application ID that owns the semantic snapshot/refs.
    pub(crate) application_id: String,
    /// Snapshot ID returned by desktop_semantic_snapshot.
    pub(crate) snapshot_id: String,
    /// Ordered already-grounded semantic steps. RepoTunnel accepts 1..64 steps.
    pub(crate) steps: Vec<UiSemanticSequenceStepParam>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct AiWorkspaceSemanticSequenceParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Per-application appSessionId. Required when multiple app sessions share the desktop.
    pub(crate) app_session_id: Option<String>,
    /// Snapshot ID returned by ai_workspace_semantic_snapshot.
    pub(crate) snapshot_id: String,
    /// Ordered already-grounded semantic steps. RepoTunnel accepts 1..64 steps.
    pub(crate) steps: Vec<UiSemanticSequenceStepParam>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct DesktopActionParams {
    /// Approved project whose local desktop permission grants apply.
    pub(crate) workspace_id: String,
    /// Enabled application ID returned by list_desktop_applications.
    pub(crate) application_id: String,
    /// One of activate, click, type, key, or scroll.
    pub(crate) action: String,
    /// Semantic element ID returned by inspect_desktop_app. Required for type; preferred for click.
    pub(crate) element_id: Option<String>,
    /// Optional app-owned window ID returned by inspect_desktop_app.
    pub(crate) window_id: Option<String>,
    /// Text for action=type. Password/credential fields are blocked by RepoTunnel.
    pub(crate) text: Option<String>,
    /// Replace existing field contents for action=type. Defaults to false.
    pub(crate) clear_first: Option<bool>,
    /// Safe keyboard shortcut for action=key, such as Ctrl+S, Escape, Enter, or F5.
    pub(crate) shortcut: Option<String>,
    /// Window-relative horizontal ratio 0..1 for screenshot-grounded fallback click/scroll.
    pub(crate) x_ratio: Option<f64>,
    /// Window-relative vertical ratio 0..1 for screenshot-grounded fallback click/scroll.
    pub(crate) y_ratio: Option<f64>,
    /// Horizontal wheel delta for action=scroll.
    pub(crate) delta_x: Option<i32>,
    /// Vertical wheel delta for action=scroll.
    pub(crate) delta_y: Option<i32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct DesktopScreenshotParams {
    /// Approved project whose local desktop permission grants apply.
    pub(crate) workspace_id: String,
    /// Enabled application ID returned by list_desktop_applications.
    pub(crate) application_id: String,
    /// Optional app-owned window ID; omit to capture the first current window.
    pub(crate) window_id: Option<String>,
}

