//! Read-only and deny-only local UI surface. No grant/issue endpoint.
use tauri::State;
use crate::{app_state::AppState, desktop_approvals::ApprovalRequest};

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
    Ok(())
}
