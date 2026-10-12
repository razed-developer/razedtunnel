#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct ProjectSnapshotParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Maximum filtered tree entries to return. Values are clamped to 100..25000.
    pub(crate) entry_limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct WorkspacePathParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Path relative to the workspace root. Use an empty string for the workspace root.
    pub(crate) relative_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct SearchFilesParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// File or folder path relative to the workspace root. Use an empty string to search the whole workspace.
    pub(crate) relative_path: String,
    /// Case-insensitive text to find. The query cannot be empty.
    pub(crate) query: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct FastSearchFilesParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// File or folder path relative to the workspace root. Use an empty string to search the whole workspace.
    pub(crate) relative_path: String,
    /// Case-insensitive text to find. The query cannot be empty.
    pub(crate) query: String,
    /// Opaque continuation cursor returned by a previous fast_search_files call.
    #[serde(default)]
    pub(crate) cursor: Option<String>,
    /// Maximum matches to return in this page. Defaults to 40 and is clamped to 1..100.
    #[serde(default)]
    pub(crate) max_results: Option<usize>,
    /// Maximum filesystem work for this page in milliseconds. Defaults to 900 and is clamped to 100..1500.
    #[serde(default)]
    pub(crate) budget_ms: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct ReadFileRangeParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// File path relative to the workspace root.
    pub(crate) relative_path: String,
    /// First 1-based line to return when starting a range read. Defaults to 1.
    #[serde(default)]
    pub(crate) start_line: Option<usize>,
    /// Maximum lines to return in one bounded page. Defaults to 240 and is clamped to 1..1000.
    #[serde(default)]
    pub(crate) max_lines: Option<usize>,
    /// Opaque continuation cursor returned by a previous read_file_range call.
    #[serde(default)]
    pub(crate) cursor: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct ListDirectoryPageParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Folder path relative to the workspace root. Use an empty string for the workspace root.
    pub(crate) relative_path: String,
    /// Opaque continuation cursor returned by a previous list_directory_page call.
    #[serde(default)]
    pub(crate) cursor: Option<String>,
    /// Maximum accessible entries to return in one page. Defaults to 200 and is clamped to 1..500.
    #[serde(default)]
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct InspectProjectPageParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Folder path relative to the workspace root. Use an empty string for the whole project.
    pub(crate) relative_path: String,
    /// Opaque continuation cursor returned by a previous inspect_project_page call.
    #[serde(default)]
    pub(crate) cursor: Option<String>,
    /// Maximum tree entries to return in one bounded page. Defaults to 300 and is clamped to 1..800.
    #[serde(default)]
    pub(crate) page_size: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct FileContentParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// File path relative to the workspace root.
    pub(crate) relative_path: String,
    /// Complete UTF-8 text content for the file.
    pub(crate) content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct PatchFileParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Existing file path relative to the workspace root.
    pub(crate) relative_path: String,
    /// Exact text expected to appear exactly once in the current file.
    pub(crate) expected: String,
    /// Text that replaces the expected text. May be empty to remove the expected text.
    pub(crate) replacement: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct CreateDirectoryParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// New folder path relative to the workspace root.
    pub(crate) relative_path: String,
    /// When true, create missing parent folders as needed.
    pub(crate) recursive: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct RenameEntryParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Existing file or folder path relative to the workspace root.
    pub(crate) relative_path: String,
    /// New basename only. Do not include path separators.
    pub(crate) new_name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct MoveEntryParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Existing file or folder path relative to the workspace root.
    pub(crate) source_path: String,
    /// New full path relative to the same workspace root.
    pub(crate) destination_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct DeleteEntryParams {
    /// ID returned by list_workspaces for the approved project.
    pub(crate) workspace_id: String,
    /// Existing file or folder path relative to the workspace root. The workspace root itself cannot be deleted.
    pub(crate) relative_path: String,
    /// Required for deleting non-empty folders. Has no effect for files.
    pub(crate) recursive: bool,
}

