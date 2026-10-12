use tauri::AppHandle;
use crate::{filesystem, models::{DirectoryEntry, FileContent, SearchMatch}, projects::approved_workspace};

#[tauri::command]
pub fn list_directory(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
) -> Result<Vec<DirectoryEntry>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    filesystem::list_directory(&workspace, &relative_path)
}

#[tauri::command]
pub fn read_file(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
) -> Result<FileContent, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    filesystem::read_file(&workspace, &relative_path)
}

#[tauri::command]
pub fn search_files(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    query: String,
) -> Result<Vec<SearchMatch>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    filesystem::search_files(&workspace, &relative_path, &query)
}

