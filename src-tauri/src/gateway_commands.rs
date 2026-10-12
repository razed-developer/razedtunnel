use tauri::{AppHandle, Manager, State};
use crate::{app_state::AppState, hardening, mcp_server, models::GatewayStatus, storage::load_workspaces};

fn status(app: &AppHandle, state: &AppState) -> Result<GatewayStatus, String> {
    let (running, port) = state.gateway_status()?;

    let (task_tool_successes, task_tool_failures) = mcp_server::task_tool_outcomes();
    Ok(GatewayStatus {
        task_tool_successes,
        task_tool_failures,
        running,
        port,
        workspace_count: load_workspaces(app)?.len(),
    })
}

#[tauri::command]
pub fn get_gateway_status(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<GatewayStatus, String> {
    status(&app, state.inner())
}

#[tauri::command]
pub fn start_gateway(app: AppHandle, state: State<'_, AppState>) -> Result<GatewayStatus, String> {
    state.start_gateway(app.clone())?;
    hardening::log_event(
        &app,
        "INFO",
        "gateway.start",
        "Loopback MCP gateway started.",
    );
    if state.public_tunnel_status(&app)?.configured {
        if let Err(error) = state.start_public_tunnel(app.clone()) {
            hardening::log_event(&app, "WARN", "public_tunnel.start", &error);
        }
    }
    status(&app, state.inner())
}

#[tauri::command]
pub async fn stop_gateway(app: AppHandle) -> Result<GatewayStatus, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_for_task.state::<AppState>();
        state.stop_gateway()?;
        hardening::log_event(
            &app_for_task,
            "INFO",
            "gateway.stop",
            "Loopback MCP gateway and managed remote connections stopped cleanly.",
        );
        status(&app_for_task, state.inner())
    })
    .await
    .map_err(|error| format!("Gateway stop task could not complete: {error}"))?
}

