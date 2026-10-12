use std::collections::BTreeMap;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct WorkspaceCommandParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Exact preset ID returned by list_command_presets. Arbitrary shell commands are not accepted.
    pub(crate) preset_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct ListCommandsParams {
    /// Optional workspace ID. Omit to list recent command records across approved projects.
    pub(crate) workspace_id: Option<String>,
    /// Maximum records to return. RepoTunnel clamps this to 1..100.
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct TerminalCommandParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Arbitrary shell command to run in the real approved workspace.
    pub(crate) command: String,
    /// Optional workspace-relative working directory. Omit or use an empty string for the project root.
    pub(crate) cwd: Option<String>,
    /// Optional timeout in seconds for this one-shot command. RepoTunnel clamps it to 1..43200 (12 hours); persistent processes use start_process and do not inherit this one-shot timeout.
    pub(crate) timeout_seconds: Option<u64>,
    /// Optional environment-variable overrides applied only to this command. Credential-like variable names are rejected for AI commands.
    pub(crate) env: Option<BTreeMap<String, String>>,
    /// Legacy compatibility flag for explicit push intent. When RepoTunnel GitHub is verified connected, publishing access is already granted without this flag.
    pub(crate) user_requested_push: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct ManagedProcessStartParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Arbitrary shell command for the persistent process, such as a development server.
    pub(crate) command: String,
    /// Optional workspace-relative working directory. Omit or use an empty string for the project root.
    pub(crate) cwd: Option<String>,
    /// Optional human-readable label for the process.
    pub(crate) label: Option<String>,
    /// Optional environment-variable overrides applied only to this process.
    pub(crate) env: Option<BTreeMap<String, String>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct ListProcessesParams {
    /// Optional workspace ID. Omit to inspect managed processes across approved projects.
    pub(crate) workspace_id: Option<String>,
    /// Maximum records to return. RepoTunnel clamps this to 1..100.
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct WorkspaceRuntimeStatusParams {
    /// Approved workspace whose lightweight Git + managed-process state should be returned.
    pub(crate) workspace_id: String,
    /// Maximum managed-process records to return. RepoTunnel clamps this to 1..100.
    pub(crate) process_limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct ProcessIdParams {
    /// Managed process ID returned by start_process or list_processes.
    pub(crate) process_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct ProcessOutputParams {
    /// Managed process ID returned by start_process or list_processes.
    pub(crate) process_id: String,
    /// Byte offset for incremental stdout reads. Omit for the beginning.
    pub(crate) stdout_offset: Option<u64>,
    /// Byte offset for incremental stderr reads. Omit for the beginning.
    pub(crate) stderr_offset: Option<u64>,
    /// Maximum bytes to return from each stream. RepoTunnel clamps this to 1..65536.
    pub(crate) max_bytes: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct ProcessWaitParams {
    /// Managed process ID returned by start_process or list_processes.
    pub(crate) process_id: String,
    /// Literal output strings that immediately count as success. Maximum 32.
    #[serde(default)]
    pub(crate) success_patterns: Vec<String>,
    /// Literal output strings that immediately count as failure. Maximum 32.
    #[serde(default)]
    pub(crate) failure_patterns: Vec<String>,
    /// Maximum time to wait. RepoTunnel clamps this to 1..600 seconds.
    pub(crate) timeout_seconds: Option<u64>,
    /// Start scanning stdout at this byte offset. Omit for the beginning.
    pub(crate) stdout_offset: Option<u64>,
    /// Start scanning stderr at this byte offset. Omit for the beginning.
    pub(crate) stderr_offset: Option<u64>,
    /// Maximum bytes returned from each stream when the wait finishes.
    pub(crate) max_bytes: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct StopProcessParams {
    /// Managed process ID returned by start_process or list_processes.
    pub(crate) process_id: String,
    /// When true, stop immediately; otherwise RepoTunnel first attempts a graceful process-group stop.
    pub(crate) force: Option<bool>,
}

