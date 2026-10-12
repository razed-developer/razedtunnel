use tauri::{AppHandle, State};
use crate::{app_state::AppState, browser, hardening, models::AiAccessStatus, storage::save_ai_access_paused, terminal};

#[tauri::command]
pub fn get_ai_access_status(state: State<'_, AppState>) -> Result<AiAccessStatus, String> {
    Ok(AiAccessStatus {
        paused: state.ai_access_paused(),
    })
}

#[tauri::command]
pub fn set_ai_access_paused(
    app: AppHandle,
    state: State<'_, AppState>,
    paused: bool,
) -> Result<AiAccessStatus, String> {
    if paused {
        // Emergency pause must take effect even if persisting the setting fails.
        state.set_ai_access_paused(true);
        terminal::stop_all_activity(&app);
        browser::stop_all_activity();
        save_ai_access_paused(&app, true).map_err(|error| {
            format!(
                "AI access is paused for this session, but RepoTunnel could not save the paused state: {error}"
            )
        })?;
    } else {
        // Resume remains persistence-first so a storage failure cannot fail open.
        save_ai_access_paused(&app, false)?;
        state.set_ai_access_paused(false);
    }
    hardening::log_event(
        &app,
        "INFO",
        if paused {
            "ai_access.pause"
        } else {
            "ai_access.resume"
        },
        if paused {
            "MCP workspace access paused by the user."
        } else {
            "MCP workspace access resumed by the user."
        },
    );
    Ok(AiAccessStatus { paused })
}

