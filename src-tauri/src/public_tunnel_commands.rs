use tauri::{AppHandle, Manager, State};
use crate::{app_state::AppState, hardening, mcp_auth, models::PublicTunnelStatus, public_tunnel::PublicTunnelProvider};

#[tauri::command]
pub async fn get_public_tunnel_status(app: AppHandle) -> Result<PublicTunnelStatus, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_for_task.state::<AppState>();
        state.public_tunnel_status(&app_for_task)
    })
    .await
    .map_err(|error| format!("Public connection status task could not complete: {error}"))?
}

#[tauri::command]
pub fn configure_public_tunnel(
    app: AppHandle,
    state: State<'_, AppState>,
    provider: PublicTunnelProvider,
    credential: String,
    public_url: Option<String>,
) -> Result<PublicTunnelStatus, String> {
    let status = state.configure_public_tunnel(app.clone(), provider, credential, public_url)?;
    hardening::log_event(
        &app,
        "INFO",
        "public_tunnel.configure",
        &format!(
            "User-specific {} public connection configured and started.",
            status.provider
        ),
    );
    Ok(status)
}

#[tauri::command]
pub async fn restart_public_tunnel(app: AppHandle) -> Result<PublicTunnelStatus, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_for_task.state::<AppState>();
        state.stop_public_tunnel()?;
        state.start_public_tunnel(app_for_task.clone())?;
        hardening::log_event(
            &app_for_task,
            "INFO",
            "public_tunnel.restart",
            "Public connection restarted.",
        );
        state.public_tunnel_status(&app_for_task)
    })
    .await
    .map_err(|error| format!("Public connection restart task could not complete: {error}"))?
}

#[tauri::command]
pub async fn provision_direct_https_certificate(
    app: AppHandle,
    staging: bool,
) -> Result<PublicTunnelStatus, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_for_task.state::<AppState>();
        let status = state.provision_direct_certificate(app_for_task.clone(), staging)?;
        hardening::log_event(
            &app_for_task,
            "INFO",
            "public_tunnel.direct_certificate",
            if staging {
                "Direct HTTPS Let's Encrypt staging certificate provisioned."
            } else {
                "Direct HTTPS Let's Encrypt trusted certificate provisioned."
            },
        );
        Ok::<PublicTunnelStatus, String>(status)
    })
    .await
    .map_err(|error| format!("Direct HTTPS certificate task could not complete: {error}"))?
}

#[tauri::command]
pub fn revoke_mcp_access(app: AppHandle) -> Result<(), String> {
    mcp_auth::revoke_tokens(&app)?;
    hardening::log_event(
        &app,
        "INFO",
        "mcp_auth.revoke",
        "Current MCP OAuth access and refresh credentials revoked.",
    );
    Ok(())
}

#[tauri::command]
pub async fn clear_public_tunnel(app: AppHandle) -> Result<PublicTunnelStatus, String> {
    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_for_task.state::<AppState>();
        state.clear_public_tunnel(&app_for_task)?;
        hardening::log_event(
            &app_for_task,
            "INFO",
            "public_tunnel.clear",
            "Saved public connection credentials and endpoint identity removed.",
        );
        state.public_tunnel_status(&app_for_task)
    })
    .await
    .map_err(|error| format!("Public connection reset task could not complete: {error}"))?
}

