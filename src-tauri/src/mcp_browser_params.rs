use std::collections::BTreeMap;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BrowserActionParam {
    Start,
    Stop,
    OpenTab,
    ActivateTab,
    CloseTab,
    Navigate,
    Click,
    Type,
    Scroll,
    Reload,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserActionParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Browser mutation to perform. Fields not used by the selected action are ignored.
    pub(crate) action: BrowserActionParam,
    /// Browser application ID returned by list_automation_browsers. Required for action=start.
    pub(crate) application_id: Option<String>,
    /// Browser tab ID returned by list_browser_tabs. Required for activate_tab, close_tab, navigate, click, type, scroll, and reload.
    pub(crate) tab_id: Option<String>,
    /// HTTP/HTTPS URL. Required for open_tab and navigate.
    pub(crate) url: Option<String>,
    /// CSS selector. Required for click and type.
    pub(crate) selector: Option<String>,
    /// Text to enter. Required for type. Completed browser history records do not retain this text.
    pub(crate) text: Option<String>,
    /// For type, clear the target field before entering text. Defaults to false.
    pub(crate) clear_first: Option<bool>,
    /// Horizontal scroll delta in CSS pixels. Used only by scroll. Defaults to 0.
    pub(crate) delta_x: Option<i32>,
    /// Vertical scroll delta in CSS pixels. Used only by scroll. Defaults to 600 when both deltas are omitted.
    pub(crate) delta_y: Option<i32>,
    /// For navigate, maximum time to wait for the new document before returning an atomic receipt. Defaults to 12000 ms and is clamped to 1000..30000.
    pub(crate) timeout_ms: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserContextParams {
    /// Approved workspace whose managed browser should use this persistent context.
    pub(crate) workspace_id: String,
    /// Human-readable context name such as normal-user or security-research.
    pub(crate) name: String,
    /// Non-secret default HTTP headers applied before navigation. Authorization, Cookie, token/secret/password/credential-like names are rejected.
    pub(crate) default_headers: Option<BTreeMap<String, String>>,
    /// Optional user-agent override for this workspace browser context.
    pub(crate) user_agent: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BrowserSemanticActionParam {
    Click,
    Type,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserSemanticActionParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Semantic mutation to perform.
    pub(crate) action: BrowserSemanticActionParam,
    /// Browser tab ID that owns the semantic snapshot/ref.
    pub(crate) tab_id: String,
    /// Snapshot ID returned by browser_semantic_snapshot.
    pub(crate) snapshot_id: String,
    /// Short-lived semantic ref such as e1.
    pub(crate) ref_id: String,
    /// Text to enter. Required only for action=type.
    pub(crate) text: Option<String>,
    /// For action=type, clear the current field before entering text. Defaults to false.
    pub(crate) clear_first: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BrowserSemanticSequenceOperationParam {
    Click,
    Type,
    Wait,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserSemanticSequenceStepParam {
    /// One of click, type, or wait.
    pub(crate) operation: BrowserSemanticSequenceOperationParam,
    /// Short-lived semantic ref such as e1. Required for click and type.
    pub(crate) ref_id: Option<String>,
    /// Text to enter. Required only for type and never included in completed browser-history summaries.
    pub(crate) text: Option<String>,
    /// For type, clear the current field before entering text. Defaults to false.
    pub(crate) clear_first: Option<bool>,
    /// Bounded delay for wait. Maximum 2000 ms per step and 10000 ms total.
    pub(crate) wait_ms: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserSemanticSequenceParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Browser tab ID that owns the semantic snapshot/refs.
    pub(crate) tab_id: String,
    /// Snapshot ID returned by browser_semantic_snapshot.
    pub(crate) snapshot_id: String,
    /// Optional caller-chosen sequence ID (letters, digits, '-' or '_', max 128 chars). Provide one when another concurrent request may need to cancel this sequence. RepoTunnel generates an ID when omitted.
    pub(crate) sequence_id: Option<String>,
    /// Ordered already-grounded semantic steps. RepoTunnel accepts 1..64 steps.
    pub(crate) steps: Vec<BrowserSemanticSequenceStepParam>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserSemanticSequenceCancelParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Sequence ID supplied to browser_semantic_sequence.
    pub(crate) sequence_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserInspectParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Browser tab ID returned by list_browser_tabs.
    pub(crate) tab_id: String,
    /// Optional CSS selector. Omit to inspect the page document/body.
    pub(crate) selector: Option<String>,
    /// Maximum text/HTML characters to return. RepoTunnel clamps this internally.
    pub(crate) max_chars: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserDownloadConfigureParams {
    pub(crate) workspace_id: String,
    /// Any currently open managed browser tab. Download tracking itself is browser-wide.
    pub(crate) tab_id: String,
    /// Existing RepoTunnel temporary task. Downloads are stored under its downloads/ directory.
    pub(crate) task_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserDownloadCancelParams {
    pub(crate) workspace_id: String,
    /// Download GUID returned by list_browser_downloads.
    pub(crate) guid: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserUploadParams {
    pub(crate) workspace_id: String,
    pub(crate) tab_id: String,
    /// CSS selector for an <input type=file> control.
    pub(crate) selector: String,
    /// Existing approved workspace-relative file path, including a file inside .repotunnel-tmp.
    pub(crate) relative_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserSemanticSnapshotParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Browser tab ID returned by list_browser_tabs.
    pub(crate) tab_id: String,
    /// Maximum accessibility nodes to return. RepoTunnel clamps this to 20..2000.
    pub(crate) max_nodes: Option<usize>,
    /// Hash from a previous semantic snapshot. When the state is unchanged, RepoTunnel can return a compact unchanged response.
    pub(crate) known_hash: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserSemanticFindParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Browser tab ID whose semantic snapshot is being searched.
    pub(crate) tab_id: String,
    /// Snapshot ID returned by browser_semantic_snapshot.
    pub(crate) snapshot_id: String,
    /// Optional free-text match across safe role/name/description/text/value fields.
    pub(crate) query: Option<String>,
    /// Optional accessibility role filter.
    pub(crate) role: Option<String>,
    /// Optional accessible-name substring filter.
    pub(crate) name: Option<String>,
    /// Optional exact semantic state filter such as enabled, focused, or checked.
    pub(crate) state: Option<String>,
    /// Optional exact supported-action filter such as click, type, or focus.
    pub(crate) action: Option<String>,
    /// Maximum matches to return. RepoTunnel clamps this to 1..100.
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserScreenshotParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Browser tab ID returned by list_browser_tabs.
    pub(crate) tab_id: String,
    /// True for a full-page capture, false for the current viewport. Defaults to false.
    pub(crate) full_page: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct BrowserDiagnosticsParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Optional tab ID. Omit to return diagnostics across the managed browser session.
    pub(crate) tab_id: Option<String>,
    /// Maximum console entries and network failures to return per category. RepoTunnel clamps this internally.
    pub(crate) limit: Option<usize>,
}

