mod access;
mod activity;
mod activity_commands;
mod ai_resources;
mod ai_workspace;
mod app_state;
mod workspace_management_commands;
mod workspace_policy_commands;
mod chat_connection_commands;
mod gateway_commands;
mod ai_access_commands;
mod browser;
mod changes;
mod chatgpt_bridge;
mod connection;
mod continuity;
mod desktop_control;
mod visual_access_commands;
mod direct_https;
mod environment;
mod execution;
mod external_access;
mod filesystem;
mod gateway;
mod git;
mod github;
mod gmail_access;
mod hardening;
mod integrations;
mod large_project_read;
mod launcher;
#[cfg(any(target_os = "macos", test))]
mod macos_ax;
mod mcp_auth;
mod mcp_server;
mod mcp_task_metrics;
mod mcp_app_session_ids;
mod mcp_git_helpers;
mod mcp_time;
mod mcp_workspace_params;
mod mcp_continuation_params;
mod mcp_filesystem_params;
mod mcp_execution_params;
mod mcp_launch_params;
mod mcp_ai_workspace_params;
mod mcp_desktop_params;
mod mcp_browser_params;
mod mcp_request_identity;
mod mcp_continuation_fallback;
mod mcp_browser_access;
mod mcp_ai_workspace_helpers;
mod mcp_observation;
mod media_inspection;
mod media_tools;
mod models;
mod monitoring;
mod platform_sandbox;
mod projects;
mod project_index;
mod project_memory;
mod project_setup;
mod public_tunnel;
mod public_tunnel_commands;
mod repository;
mod secret_guard;
mod self_continuation;
mod semantic;
mod storage;
mod system_tray;
mod system_resources;
mod temp_workspace;
mod terminal;
mod updates;
mod runtime_commands;
mod versioning;
#[cfg(any(windows, test))]
mod windows_uia;
#[cfg(windows)]
mod windows_capture;
mod workflow;

use app_state::AppState;
use tauri::Manager;

pub fn maybe_run_platform_sandbox_helper() -> Option<i32> {
    platform_sandbox::maybe_run_helper()
}

pub fn maybe_run_managed_process_supervisor() -> Option<i32> {
    terminal::maybe_run_managed_process_supervisor()
}

pub fn maybe_run_ai_workspace_github_proxy() -> Option<i32> {
    terminal::maybe_run_ai_workspace_github_proxy()
}

fn install_rustls_crypto_provider() {
    // RepoTunnel's dependency graph intentionally contains both AWS-LC (ngrok/axum-server)
    // and ring (reqwest/updater). Rustls 0.23 refuses to guess when both are enabled, so
    // choose one process-wide provider before any TLS client/server is constructed.
    // install_default is idempotent for our startup purpose: if a provider was already
    // installed by an earlier library initializer, keep that valid provider instead.
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
}

fn initialize_local_runtime(app: &tauri::AppHandle) {
    // Legacy background integrations are excluded from the lean gateway by default.
    // Re-enable temporarily with --features legacy_background_services.
    #[cfg(feature = "legacy_background_services")]
    {
        // GitHub CLI authentication is global RepoTunnel state, not a Commands-page
        // feature. Prewarm it at application startup using the existing persistent
        // human gh profile. This is read-only: it never logs out, refreshes, replaces,
        // or copies the user's credential.
        if let Err(error) = std::thread::Builder::new()
            .name("repotunnel-github-status".to_string())
            .spawn(|| {
                let _ = github::status();
            })
        {
            eprintln!("RepoTunnel GitHub status prewarm warning: {error}");
        }

        let state = app.state::<AppState>();
        match state.ai_workspace.initialize(app) {
            Ok(true) => hardening::log_event(
                app,
                "INFO",
                "ai_workspace.recover",
                "Reattached the durable RepoTunnel AI Workspace session.",
            ),
            Ok(false) => {}
            Err(error) => hardening::log_event(app, "WARN", "ai_workspace.recover", &error),
        }
    }

    if let Err(error) = terminal::initialize(app) {
        hardening::log_event(app, "WARN", "terminal.initialize", &error);
    }
    #[cfg(feature = "legacy_background_services")]
    if let Err(error) = chatgpt_bridge::initialize(app) {
        hardening::log_event(app, "WARN", "chatgpt_bridge.initialize", &error);
    }
    if let Err(error) = monitoring::initialize(app) {
        hardening::log_event(app, "WARN", "monitoring.initialize", &error);
    }
}

fn initialize_connection_runtime(app: &tauri::AppHandle) {
    // The MCP/Direct HTTPS transport is RepoTunnel's recovery/control channel.
    // Keep it independent from optional product features so connection recovery cannot
    // strand the application with Direct HTTPS and MCP unavailable.
    let configured = public_tunnel::load_config(app)
        .ok()
        .flatten()
        .is_some_and(|config| config.auto_start);
    app.state::<AppState>().set_public_connection_desired(configured);
    if configured {
        let app_handle = app.clone();
        if let Err(error) = std::thread::Builder::new()
            .name("repotunnel-auto-connect".to_string())
            .spawn(move || {
                let state = app_handle.state::<AppState>();
                // A login-time network delay can make the first connection attempt fail.
                // Retry a few times without requiring the configuration window to be open.
                for attempt in 1..=3 {
                    match state.start_public_tunnel(app_handle.clone()) {
                        Ok(()) => break,
                        Err(error) => {
                            hardening::log_event(
                                &app_handle,
                                "WARN",
                                "public_tunnel.auto_start",
                                &format!("Attempt {attempt}/3 failed: {error}"),
                            );
                            if attempt < 3 {
                                std::thread::sleep(std::time::Duration::from_secs(10));
                            }
                        }
                    }
                }
            })
        {
            eprintln!("RepoTunnel auto-connect worker warning: {error}");
        }
    }
    // Keep the transport available after longer outages, even when the window is hidden.
    // An intentional gateway stop clears the desired flag; exit cancels the loop.
    let app_handle = app.clone();
    if let Err(error) = std::thread::Builder::new()
        .name("repotunnel-connection-watch".to_string())
        .spawn(move || {
            let state = app_handle.state::<AppState>();
            while !state.shutting_down() {
                std::thread::sleep(std::time::Duration::from_secs(15));
                if state.shutting_down() || !state.public_connection_desired() {
                    continue;
                }
                match state.public_tunnel_status(&app_handle) {
                    Ok(status) if !status.running => {
                        if let Err(error) = state.start_public_tunnel(app_handle.clone()) {
                            hardening::log_event(&app_handle, "WARN", "public_tunnel.recover", &error);
                        }
                    }
                    Err(error) => {
                        hardening::log_event(&app_handle, "WARN", "public_tunnel.status", &error);
                    }
                    _ => {}
                }
            }
        })
    {
        eprintln!("RepoTunnel connection recovery worker warning: {error}");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    install_rustls_crypto_provider();
    platform_sandbox::recover_stale_state();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler({
            let handler: fn(tauri::ipc::Invoke<tauri::Wry>) -> bool = tauri::generate_handler![
                workspace_management_commands::select_workspace,
                workspace_management_commands::list_workspaces,
                workspace_management_commands::open_workspace_folder,
                workspace_management_commands::get_workspace_health,
                workspace_management_commands::relocate_workspace,
                workspace_management_commands::add_workspace,
                workspace_management_commands::clone_git_workspace,
                workspace_management_commands::remove_workspace,
                workspace_policy_commands::update_workspace_access,
                workspace_policy_commands::update_workspace_change_policy,
                workspace_policy_commands::update_workspace_command_policy,
                // File explorer and editor IPC belong to the removed desktop workspace UI.
                // Filesystem and editor Rust modules remain for MCP dependencies.
                // Project lifecycle desktop IPC belonged to the removed workspace UI.
                // Preserve its Rust implementation for MCP tool dependencies.
                // Legacy desktop diagnostics IPC removed; workspace health remains
                // registered above for Projects, and MCP diagnostics stay compiled.
                // Temporary workspace desktop controls are unmounted; preserve
                // temporary workspace implementation for MCP project tooling.
                // History, checkpoints and safety scan have no mounted desktop UI.
                // Their underlying modules and MCP dependencies remain compiled.
                // Terminal desktop IPC was retired with the terminal UI.
                // Terminal services and MCP command execution remain compiled.
                // Legacy GitHub account-management desktop IPC is unmounted.
                // GitHub and git MCP implementations remain available.
                // Deep integration / desktop control / Gmail settings IPC belonged
                // to removed desktop pages; preserve MCP-side access controls.
                // AI Workspace frame/action desktop IPC belonged to the removed editor.
                // Preserve AI workspace implementation and MCP tool dependencies.
                // Browser desktop IPC was retired with the browser UI.
                // Browser engine and MCP operations remain compiled and available.
                // Legacy video Tauri commands intentionally not registered in lean gateway.
                // Their implementations remain for now; restore from Git history if needed.
                visual_access_commands::get_visual_access_status,
                visual_access_commands::set_visual_access_enabled,
                visual_access_commands::list_visual_access_applications,
                visual_access_commands::capture_visual_access_window,
                visual_access_commands::get_visual_access_application_identity,
                visual_access_commands::list_ignored_visual_applications,
                visual_access_commands::list_visual_access_windows,
                visual_access_commands::set_visual_application_ignored,
                ai_access_commands::get_ai_access_status,
                ai_access_commands::set_ai_access_paused,
                activity_commands::get_activity_timeline,
                gateway_commands::get_gateway_status,
                gateway_commands::start_gateway,
                gateway_commands::stop_gateway,
                public_tunnel_commands::get_public_tunnel_status,
                public_tunnel_commands::configure_public_tunnel,
                public_tunnel_commands::restart_public_tunnel,
                public_tunnel_commands::provision_direct_https_certificate,
                public_tunnel_commands::clear_public_tunnel,
                public_tunnel_commands::revoke_mcp_access,
                chat_connection_commands::get_chat_connection_status,
                chat_connection_commands::start_chat_connection,
                chat_connection_commands::stop_chat_connection,
                runtime_commands::get_runtime_diagnostics,
                runtime_commands::set_launch_at_login,
            ];
            handler
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            system_tray::install(app)?;
            if let Err(error) = hardening::initialize(app.handle()) {
                eprintln!("RepoTunnel production initialization warning: {error}");
            }
            let paused = match storage::load_ai_access_paused(app.handle()) {
                Ok(paused) => paused,
                Err(error) => {
                    eprintln!(
                        "RepoTunnel AI access settings could not be loaded; AI access remains paused: {error}"
                    );
                    true
                }
            };
            app.state::<AppState>().set_ai_access_paused(paused);
            updates::complete_post_update_health_check(app.handle());
            initialize_connection_runtime(app.handle());
            initialize_local_runtime(app.handle());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build RepoTunnel")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                app_handle.state::<AppState>().mark_shutting_down();
                monitoring::stop_all_activity();
                terminal::stop_transient_activity_for_shutdown();
                browser::stop_all_activity();
                #[cfg(feature = "legacy_background_services")]
                {
                    chatgpt_bridge::shutdown();
                    app_handle.state::<AppState>().ai_workspace.shutdown(app_handle);
                }
                let _ = app_handle.state::<AppState>().stop_gateway();
                hardening::shutdown(app_handle);
            }
        });
}
