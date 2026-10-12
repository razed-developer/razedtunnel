use tauri::AppHandle;
use crate::{chatgpt_bridge, mcp_request_identity::continuation_identity_keys_from, mcp_server::approved_workspace};

pub(crate) fn ensure_chatgpt_work_fallback_with_identities(
    app: &AppHandle,
    workspace_id: &str,
    identity_keys: &[String],
) {
    if approved_workspace(app, workspace_id).is_err() {
        return;
    }
    if let Err(error) =
        chatgpt_bridge::ensure_automatic_fallback_for_identities(workspace_id, identity_keys)
    {
        eprintln!("RepoTunnel automatic ChatGPT continuation fallback failed: {error}");
    }
}

pub(crate) fn ensure_chatgpt_work_fallback(
    app: &AppHandle,
    workspace_id: &str,
    conversation_session: Option<&str>,
) {
    let identity_keys = continuation_identity_keys_from(conversation_session);
    ensure_chatgpt_work_fallback_with_identities(app, workspace_id, &identity_keys);
}

