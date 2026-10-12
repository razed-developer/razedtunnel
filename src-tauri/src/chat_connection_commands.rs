use tauri::{AppHandle, State};
use crate::{app_state::AppState, hardening, models::ChatConnectionStatus};

#[tauri::command]
pub fn get_chat_connection_status(
    state: State<'_, AppState>,
) -> Result<ChatConnectionStatus, String> {
    state.chat_connection_status()
}

#[tauri::command]
pub fn start_chat_connection(
    app: AppHandle,
    state: State<'_, AppState>,
    tunnel_id: String,
    api_key: String,
) -> Result<ChatConnectionStatus, String> {
    let result = state.start_chat_connection(app.clone(), tunnel_id, api_key);
    if result.is_ok() {
        hardening::log_event(
            &app,
            "INFO",
            "chat.connect",
            "Secure MCP Tunnel process started.",
        );
    }
    result
}

#[tauri::command]
pub fn stop_chat_connection(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ChatConnectionStatus, String> {
    state.stop_chat_connection()?;
    hardening::log_event(
        &app,
        "INFO",
        "chat.disconnect",
        "Secure MCP Tunnel stopped.",
    );
    state.chat_connection_status()
}

