//! Read-only and deny-only local UI surface. No grant/issue endpoint.
use tauri::State;
use crate::{app_state::AppState, desktop_approvals::ApprovalRequest, models::{ActivityKind, ActivityStatus, BrowserActionStatus}};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PendingApprovalView {
    id: String,
    workspace_id: String,
    path: String,
    operation: String,
}

impl From<ApprovalRequest> for PendingApprovalView {
    fn from(request: ApprovalRequest) -> Self {
        Self {
            id: request.id,
            workspace_id: request.approval.workspace_id,
            path: request.approval.path.to_string_lossy().into_owned(),
            operation: format!("{:?}", request.approval.operation),
        }
    }
}

#[tauri::command]
pub(crate) fn list_pending_approval_requests(state: State<'_, AppState>) -> Result<Vec<PendingApprovalView>, String> {
    Ok(state.approval_requests.pending()?.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub(crate) fn deny_pending_approval_request(state: State<'_, AppState>, request_id: String) -> Result<bool, String> {
    state.approval_requests.deny(&request_id)
}


/// Rejection cannot execute an action or grant permissions.
#[tauri::command]
pub(crate) fn reject_pending_browser_action(app: tauri::AppHandle, action_id: String) -> Result<(), String> {
    crate::browser::reject_action(&app, &action_id)?;
    crate::activity::update_operation_status(&app, ActivityKind::Browser, &action_id, ActivityStatus::Rejected)?;
    Ok(())
}

#[tauri::command]
pub(crate) fn reject_pending_file_change(app: tauri::AppHandle, change_id: String) -> Result<(), String> {
    crate::changes::reject_change(&app, &change_id)?;
    crate::activity::update_operation_status(&app, ActivityKind::Files, &change_id, ActivityStatus::Rejected)?;
    Ok(())
}

use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri::Manager;

/// Only the main desktop window may request a local, explicit native confirmation.
#[tauri::command]
pub(crate) fn approve_pending_browser_action(
    window: tauri::Window,
    app: tauri::AppHandle,
    action_id: String,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Browser approval requires the main desktop window.".to_string());
    }
    let record = crate::browser::get_action(&app, &action_id)?;
    if record.status != crate::models::BrowserActionStatus::Pending {
        return Err("Only pending browser actions can be approved.".to_string());
    }
    let workspace = crate::storage::load_workspaces(&app)?
        .into_iter()
        .find(|workspace| workspace.id == record.workspace_id)
        .ok_or_else(|| "The project is no longer approved.".to_string())?;
    let summary = format!(
        "Approve this queued browser operation?\n\nProject: {}\nOperation: {:?}\nTarget: {}\nDetails: {}\n\nThis may change the browser or website. Choose No to leave it pending.",
        record.workspace_name,
        record.kind,
        record.target,
        record.detail.as_deref().unwrap_or("(none)")
    );
    if !app.dialog().message(summary).buttons(MessageDialogButtons::YesNo).blocking_show() {
        return Err("Approval cancelled; the request remains pending.".to_string());
    }
    // approve_action revalidates pending status and project identity under its execution lock.
    let result = crate::browser::approve_action(&app, &workspace, &action_id)?;
    let status = match result.status {
        BrowserActionStatus::Applied => ActivityStatus::Succeeded,
        BrowserActionStatus::Rejected => ActivityStatus::Rejected,
        BrowserActionStatus::Pending => ActivityStatus::Pending,
        BrowserActionStatus::Ambiguous | BrowserActionStatus::Failed => ActivityStatus::Failed,
    };
    crate::activity::update_operation_status(&app, ActivityKind::Browser, &action_id, status)?;
    Ok(())
}

#[tauri::command]
pub(crate) fn approve_pending_file_change(
    window: tauri::Window,
    app: tauri::AppHandle,
    change_id: String,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("File approval requires the main desktop window.".to_string());
    }
    let record = crate::changes::review_pending_patch(&app, &change_id)?;
    if record.status != crate::models::ChangeStatus::Pending {
        return Err("Only pending file changes can be approved.".to_string());
    }
    let workspace = crate::storage::load_workspaces(&app)?
        .into_iter()
        .find(|workspace| workspace.id == record.workspace_id)
        .ok_or_else(|| "The project is no longer approved.".to_string())?;
    let preview = record.diff.as_deref().unwrap_or("(No text diff available)");
    // The stored diff is a bounded summary, not a complete patch. Never authorize
    // a change if the preview is truncated or too large for a readable native dialog.
    if record.diff.is_none()
        || preview.contains("… diff preview truncated …")
        || preview.len() > 12_000
    {
        return Err("A complete, readable change preview is unavailable. Approval is blocked; do not apply this change from the dashboard.".to_string());
    }
    // Non-patch operations can have additional filesystem effects not represented
    // by a text diff. Keep them outside this first approval surface.
    if !matches!(record.operation, crate::models::ChangeOperation::PatchFile) {
        return Err("Only file patches with a readable diff can be approved here.".to_string());
    }
    let message = format!(
        "Apply this queued file change?\n\nProject: {}\nOperation: {:?}\nFile: {}\nSummary: {}\n\nProposed diff:\n{}\n\nYes applies the change; No leaves it pending.",
        workspace.name,
        record.operation,
        record.primary_path,
        record.summary,
        preview
    );
    if !app.dialog().message(message).buttons(MessageDialogButtons::YesNo).blocking_show() {
        return Err("Approval cancelled; the file change remains pending.".to_string());
    }
    let outcome = crate::changes::approve_change(&app, &change_id)?;
    let status = match outcome.change.status {
        crate::models::ChangeStatus::Applied => ActivityStatus::Succeeded,
        crate::models::ChangeStatus::Rejected => ActivityStatus::Rejected,
        crate::models::ChangeStatus::Pending => ActivityStatus::Pending,
        crate::models::ChangeStatus::Failed => ActivityStatus::Failed,
        crate::models::ChangeStatus::Undone => ActivityStatus::Stopped,
    };
    crate::activity::update_operation_status(&app, ActivityKind::Files, &change_id, status)?;
    Ok(())
}

#[tauri::command]
pub(crate) fn approve_pending_command(
    window: tauri::Window,
    app: tauri::AppHandle,
    command_id: String,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Command approval requires the main desktop window.".to_string());
    }
    let record = crate::execution::list_history(&app, None, 100)?
        .into_iter()
        .find(|record| record.id == command_id)
        .ok_or_else(|| "The command request no longer exists in recent history.".to_string())?;
    if record.status != crate::models::CommandStatus::Pending {
        return Err("Only pending commands can be approved.".to_string());
    }
    let workspace = crate::storage::load_workspaces(&app)?
        .into_iter()
        .find(|workspace| workspace.id == record.workspace_id)
        .ok_or_else(|| "The project is no longer approved.".to_string())?;
    let message = format!(
        "Run this queued validation command?\n\nProject: {}\nPreset: {}\nCommand: {}\n\nThe command runs in a disposable, network-disabled sandbox. Yes executes it; No leaves it pending.",
        workspace.name, record.preset_id, record.command
    );
    if !app.dialog().message(message).buttons(MessageDialogButtons::YesNo).blocking_show() {
        return Err("Approval cancelled; the command remains pending.".to_string());
    }
    let outcome = match crate::execution::approve_command(&app, &workspace, &command_id) {
        Ok(outcome) => outcome,
        Err(error) => {
            // Execution may fail after the command has already transitioned to
            // Failed. Keep the activity feed synchronized with durable history.
            if let Ok(records) = crate::execution::list_history(&app, Some(&workspace.id), 100) {
                if let Some(current) = records.iter().find(|item| item.id == command_id) {
                    let status = match current.status {
                        crate::models::CommandStatus::Pending => ActivityStatus::Pending,
                        crate::models::CommandStatus::Running => ActivityStatus::Running,
                        crate::models::CommandStatus::Completed => ActivityStatus::Succeeded,
                        crate::models::CommandStatus::Rejected => ActivityStatus::Rejected,
                        crate::models::CommandStatus::Cancelled => ActivityStatus::Stopped,
                        _ => ActivityStatus::Failed,
                    };
                    let _ = crate::activity::update_operation_status(
                        &app, ActivityKind::Verification, &command_id, status
                    );
                }
            }
            return Err(error);
        }
    };
    let status = match outcome.status {
        crate::models::CommandStatus::Completed => ActivityStatus::Succeeded,
        crate::models::CommandStatus::Rejected => ActivityStatus::Rejected,
        crate::models::CommandStatus::Pending => ActivityStatus::Pending,
        crate::models::CommandStatus::Running => ActivityStatus::Running,
        _ => ActivityStatus::Failed,
    };
    crate::activity::update_operation_status(&app, ActivityKind::Verification, &command_id, status)?;
    Ok(())
}

#[tauri::command]
pub(crate) fn reject_pending_command(
    window: tauri::Window,
    app: tauri::AppHandle,
    command_id: String,
) -> Result<(), String> {
    if window.label() != "main" {
        return Err("Command rejection requires the main desktop window.".to_string());
    }
    crate::execution::reject_command(&app, &command_id)?;
    crate::activity::update_operation_status(&app, ActivityKind::Verification, &command_id, ActivityStatus::Rejected)?;
    Ok(())
}

#[tauri::command]
pub(crate) fn reconcile_pending_verification_activity(app: tauri::AppHandle) -> Result<usize, String> {
    let timeline = crate::activity::timeline(&app, None)?;
    let history = crate::execution::list_history(&app, None, 100)?;
    let mut corrected = 0usize;
    for group in timeline.groups {
        for event in group.events {
            if event.kind != ActivityKind::Verification || event.status != ActivityStatus::Pending {
                continue;
            }
            let Some(id) = event.source_id.as_deref() else { continue };
            let status = match history.iter().find(|record| record.id == id) {
                Some(record) => match record.status {
                    crate::models::CommandStatus::Pending => continue,
                    crate::models::CommandStatus::Running => ActivityStatus::Running,
                    crate::models::CommandStatus::Completed => ActivityStatus::Succeeded,
                    crate::models::CommandStatus::Rejected => ActivityStatus::Rejected,
                    crate::models::CommandStatus::Cancelled => ActivityStatus::Stopped,
                    _ => ActivityStatus::Failed,
                },
                // History is capped; never infer a terminal status from absence.
                None => continue,
            };
            crate::activity::update_operation_status(&app, ActivityKind::Verification, id, status)?;
            corrected += 1;
        }
    }
    Ok(corrected)
}
