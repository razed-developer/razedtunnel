use tauri::AppHandle;
use crate::{conversation, project_context, projects::approved_workspace};

#[tauri::command]
pub fn list_home_conversations(
    app: AppHandle,
    workspace_id: Option<String>,
) -> Result<Vec<conversation::ConversationSummary>, String> {
    if let Some(workspace_id) = workspace_id {
        let _workspace = approved_workspace(&app, &workspace_id)?;
        conversation::list(&app, Some(&workspace_id))
    } else {
        conversation::list(&app, Some(conversation::GENERAL_WORKSPACE_ID))
    }
}

#[tauri::command]
pub fn get_home_conversation(
    app: AppHandle,
    conversation_id: String,
) -> Result<conversation::HomeConversation, String> {
    let conversation = conversation::get(&app, &conversation_id)?;
    if conversation.workspace_id != conversation::GENERAL_WORKSPACE_ID {
        let _workspace = approved_workspace(&app, &conversation.workspace_id)?;
    }
    Ok(conversation)
}

#[tauri::command]
pub fn create_home_conversation(
    app: AppHandle,
    workspace_id: Option<String>,
) -> Result<conversation::HomeConversation, String> {
    let workspace = workspace_id
        .as_deref()
        .map(|workspace_id| approved_workspace(&app, workspace_id))
        .transpose()?;
    conversation::create(&app, workspace.as_ref())
}

#[tauri::command]
pub fn delete_home_conversation(app: AppHandle, conversation_id: String) -> Result<(), String> {
    let existing = conversation::get(&app, &conversation_id)?;
    if existing.workspace_id != conversation::GENERAL_WORKSPACE_ID {
        let _workspace = approved_workspace(&app, &existing.workspace_id)?;
    }
    conversation::delete(&app, &conversation_id)
}

#[tauri::command]
pub fn list_home_context_files(
    app: AppHandle,
    workspace_id: String,
    limit: Option<usize>,
) -> Result<Vec<crate::models::ProjectEntry>, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    project_context::safe_attachment_candidates(&workspace, limit.unwrap_or(250))
}

#[tauri::command]
pub async fn begin_home_chat(
    app: AppHandle,
    workspace_id: Option<String>,
    conversation_id: String,
    question: String,
    context: project_context::ProjectContextRequest,
    allow_changes: bool,
) -> Result<conversation::HomeChatStartResult, String> {
    let workspace = workspace_id
        .as_deref()
        .map(|workspace_id| approved_workspace(&app, workspace_id))
        .transpose()?;
    conversation::begin(
        app,
        workspace,
        conversation_id,
        question,
        context,
        allow_changes,
    )
    .await
}

#[tauri::command]
pub fn cancel_home_chat(generation_id: String) -> Result<bool, String> {
    conversation::cancel(&generation_id)
}

