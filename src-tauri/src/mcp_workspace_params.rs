#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct CloneRepositoryParams {
    /// GitHub repository explicitly provided by the human, as owner/repository or https://github.com/owner/repository.
    pub(crate) repository: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct CreateProjectParams {
    /// New local project name explicitly requested by the human. RepoTunnel creates it inside ~/Projects.
    pub(crate) name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct ProjectMemoryUpdateParams {
    /// Approved workspace whose persistent project memory should be updated.
    pub(crate) workspace_id: String,
    /// Concise current project description/context.
    pub(crate) summary: String,
    /// Current user/product goals worth carrying into later AI sessions.
    pub(crate) goals: Vec<String>,
    /// Important architecture/product decisions already made.
    pub(crate) decisions: Vec<String>,
    /// Stable user preferences or constraints for this project.
    pub(crate) preferences: Vec<String>,
    /// Useful unfinished work or next steps.
    pub(crate) next_steps: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExternalFileActionParam {
    Read,
    Import,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct ExternalFileAccessParams {
    /// Approved project that needs the external file.
    pub(crate) workspace_id: String,
    /// read opens a user picker and returns UTF-8 content once; import copies the selected file into destination_path.
    pub(crate) action: ExternalFileActionParam,
    /// Short explanation shown in the native approval picker.
    pub(crate) reason: Option<String>,
    /// Workspace-relative destination required for action=import. RepoTunnel never overwrites an existing entry.
    pub(crate) destination_path: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct WorkspaceIdParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct CreateTempWorkspaceParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Stable task identifier containing only letters, digits, '-' or '_'.
    pub(crate) task_id: String,
    /// Human-readable purpose of this temporary workspace.
    pub(crate) label: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct TempWorkspaceParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Task identifier returned by create_temp_workspace/list_temp_workspaces.
    pub(crate) task_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PreserveTempWorkspaceParams {
    pub(crate) workspace_id: String,
    pub(crate) task_id: String,
    /// True to preserve this task across cleanup decisions; false to make it disposable again.
    pub(crate) preserved: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct CleanupTempWorkspaceParams {
    pub(crate) workspace_id: String,
    pub(crate) task_id: String,
    /// Explicitly allow deletion of a task currently marked preserved.
    pub(crate) force_preserved: Option<bool>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TempWorkspaceFileActionParam {
    CopyToWorkspace,
    MoveToWorkspace,
    Rename,
    Delete,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct TempWorkspaceFileActionParams {
    pub(crate) workspace_id: String,
    pub(crate) task_id: String,
    pub(crate) action: TempWorkspaceFileActionParam,
    /// File path relative to the temporary task root.
    pub(crate) source_relative: String,
    /// For copy/move: normal approved project path outside .repotunnel-tmp. For rename: path relative to the same temporary task. Omit for delete.
    pub(crate) destination_relative: Option<String>,
    /// Replace an existing regular destination file when true. Defaults to false.
    pub(crate) overwrite: Option<bool>,
}

