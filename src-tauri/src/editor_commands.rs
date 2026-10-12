use std::{fs, path::Path, time::{SystemTime, UNIX_EPOCH}};
use tauri::AppHandle;
use crate::{access::{resolve_workspace_path, AccessOperation}, activity, changes, filesystem, models::{ActivityKind, ActivityStatus, ChangeOutcome, CommandPolicy, ImagePreview, Workspace, WorkspaceChangePolicy}, projects::approved_workspace};

fn manual_edit_group_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("manual-{nanos:x}")
}

pub(crate) fn local_user_workspace(mut workspace: Workspace) -> Workspace {
    // These actions come from the user pressing controls in the desktop editor/explorer,
    // so they apply immediately even when AI-originated edits are configured for Review.
    // They still pass through normal workspace validation, backups, versioning and History.
    workspace.change_policy = WorkspaceChangePolicy::Automatic;
    workspace.command_policy = CommandPolicy::Automatic;
    workspace
}

fn record_manual_file_action(
    app: &AppHandle,
    workspace: &Workspace,
    group_id: &str,
    action: &str,
    summary: String,
    outcome: &ChangeOutcome,
) {
    let _ = activity::record(
        app,
        workspace,
        Some(group_id),
        ActivityKind::Files,
        action,
        summary,
        outcome.change.diff.clone(),
        ActivityStatus::Succeeded,
        Some(outcome.change.id.clone()),
    );
}

#[tauri::command]
pub fn editor_save_file(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    content: String,
    expected_content: String,
) -> Result<ChangeOutcome, String> {
    let workspace = local_user_workspace(approved_workspace(&app, &workspace_id)?);
    let current = filesystem::read_file(&workspace, &relative_path)?;
    if current.content != expected_content {
        return Err("This file changed externally after you opened it. Compare or reload the latest version before saving.".to_string());
    }
    let group_id = manual_edit_group_id();
    let outcome = changes::write_file(
        &app,
        &workspace,
        relative_path.clone(),
        content,
        Some(&group_id),
    )?;
    record_manual_file_action(
        &app,
        &workspace,
        &group_id,
        "manualEdit",
        format!("Manual edit {relative_path}"),
        &outcome,
    );
    Ok(outcome)
}

#[tauri::command]
pub fn editor_create_file(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    content: String,
) -> Result<ChangeOutcome, String> {
    let workspace = local_user_workspace(approved_workspace(&app, &workspace_id)?);
    let group_id = manual_edit_group_id();
    let outcome = changes::create_file(
        &app,
        &workspace,
        relative_path.clone(),
        content,
        Some(&group_id),
    )?;
    record_manual_file_action(
        &app,
        &workspace,
        &group_id,
        "manualCreateFile",
        format!("Created {relative_path} manually"),
        &outcome,
    );
    Ok(outcome)
}

#[tauri::command]
pub fn editor_create_directory(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
) -> Result<ChangeOutcome, String> {
    let workspace = local_user_workspace(approved_workspace(&app, &workspace_id)?);
    let group_id = manual_edit_group_id();
    let outcome = changes::create_directory(
        &app,
        &workspace,
        relative_path.clone(),
        true,
        Some(&group_id),
    )?;
    record_manual_file_action(
        &app,
        &workspace,
        &group_id,
        "manualCreateDirectory",
        format!("Created folder {relative_path} manually"),
        &outcome,
    );
    Ok(outcome)
}

#[tauri::command]
pub fn editor_rename_entry(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    new_name: String,
) -> Result<ChangeOutcome, String> {
    let workspace = local_user_workspace(approved_workspace(&app, &workspace_id)?);
    let group_id = manual_edit_group_id();
    let outcome = changes::rename_entry(
        &app,
        &workspace,
        relative_path.clone(),
        new_name.clone(),
        Some(&group_id),
    )?;
    record_manual_file_action(
        &app,
        &workspace,
        &group_id,
        "manualRename",
        format!("Renamed {relative_path} to {new_name}"),
        &outcome,
    );
    Ok(outcome)
}

#[tauri::command]
pub fn editor_delete_entry(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
    recursive: bool,
) -> Result<ChangeOutcome, String> {
    let workspace = local_user_workspace(approved_workspace(&app, &workspace_id)?);
    let group_id = manual_edit_group_id();
    let outcome = changes::delete_entry(
        &app,
        &workspace,
        relative_path.clone(),
        recursive,
        Some(&group_id),
    )?;
    record_manual_file_action(
        &app,
        &workspace,
        &group_id,
        "manualDelete",
        format!("Deleted {relative_path} manually"),
        &outcome,
    );
    Ok(outcome)
}

fn image_mime_type(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "bmp" => Some("image/bmp"),
        _ => None,
    }
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut index = 0usize;
    while index < bytes.len() {
        let a = bytes[index];
        let b = bytes.get(index + 1).copied();
        let c = bytes.get(index + 2).copied();
        output.push(TABLE[(a >> 2) as usize] as char);
        output.push(TABLE[(((a & 0x03) << 4) | (b.unwrap_or(0) >> 4)) as usize] as char);
        output.push(match b {
            Some(value) => TABLE[(((value & 0x0f) << 2) | (c.unwrap_or(0) >> 6)) as usize] as char,
            None => '=',
        });
        output.push(match c {
            Some(value) => TABLE[(value & 0x3f) as usize] as char,
            None => '=',
        });
        index += 3;
    }
    output
}

#[tauri::command]
pub fn preview_workspace_image(
    app: AppHandle,
    workspace_id: String,
    relative_path: String,
) -> Result<ImagePreview, String> {
    const MAX_PREVIEW_BYTES: u64 = 12 * 1024 * 1024;
    let workspace = approved_workspace(&app, &workspace_id)?;
    let path = resolve_workspace_path(&workspace, &relative_path, AccessOperation::Read, true)?;
    let mime_type = image_mime_type(&path).ok_or_else(|| {
        "That file type is not supported by the built-in image preview.".to_string()
    })?;
    let metadata =
        fs::metadata(&path).map_err(|error| format!("Could not inspect image: {error}"))?;
    if !metadata.is_file() {
        return Err("The requested preview target is not a file.".to_string());
    }
    if metadata.len() > MAX_PREVIEW_BYTES {
        return Err("That image is larger than the 12 MB built-in preview limit. Open it externally instead.".to_string());
    }
    let bytes = fs::read(&path).map_err(|error| format!("Could not read image: {error}"))?;
    Ok(ImagePreview {
        path: relative_path,
        mime_type: mime_type.to_string(),
        size: metadata.len(),
        data_base64: encode_base64(&bytes),
    })
}

