mod access;
mod activity;
mod ai_resources;
mod ai_workspace;
mod app_state;
mod browser;
mod changes;
mod change_commands;
mod chatgpt_bridge;
mod checkpoint;
mod commands;
mod connection;
mod continuity;
mod conversation;
mod desktop_control;
mod direct_https;
mod environment;
mod editor_commands;
mod execution;
mod execution_commands;
mod external_access;
mod filesystem;
mod filesystem_commands;
mod gateway;
mod git;
mod github;
mod github_commands;
mod gmail_access;
mod hardening;
mod history_commands;
mod https_setup;
mod integrations;
mod large_project_read;
mod launcher;
#[cfg(any(target_os = "macos", test))]
mod macos_ax;
mod mcp_auth;
mod mcp_server;
mod media_inspection;
mod media_tools;
mod model_hub;
mod model_trial;
mod models;
mod monitoring;
mod phone;
mod phone_commands;
mod platform_sandbox;
mod projects;
mod project_context;
mod project_index;
mod project_memory;
mod project_setup;
mod public_tunnel;
mod repository;
mod secret_guard;
mod self_continuation;
mod semantic;
mod storage;
mod system_resources;
mod team;
mod temp_workspace;
mod temp_workspace_commands;
mod terminal;
mod updates;
mod versioning;
#[cfg(feature = "legacy_video")]
mod video;
#[cfg(feature = "legacy_video")]
mod video_assets;
#[cfg(feature = "legacy_video")]
mod video_director;
#[cfg(feature = "legacy_video")]
mod video_html;
#[cfg(feature = "legacy_video")]
mod video_narration;
#[cfg(feature = "legacy_video")]
mod video_narration_managed;
#[cfg(feature = "legacy_video")]
mod video_preview;
#[cfg(feature = "legacy_video")]
mod video_production;
#[cfg(feature = "legacy_video")]
mod video_qa;
#[cfg(feature = "legacy_video")]
mod video_render;
#[cfg(feature = "legacy_video")]
mod video_scene;
#[cfg(feature = "legacy_video")]
mod video_story_render;
#[cfg(any(windows, test))]
mod windows_uia;
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
    if public_tunnel::load_config(app)
        .ok()
        .flatten()
        .is_some_and(|config| config.auto_start)
    {
        let app_handle = app.clone();
        if let Err(error) = std::thread::Builder::new()
            .name("repotunnel-auto-connect".to_string())
            .spawn(move || {
                let state = app_handle.state::<AppState>();
                if let Err(error) = state.start_public_tunnel(app_handle.clone()) {
                    hardening::log_event(&app_handle, "WARN", "public_tunnel.auto_start", &error);
                }
            })
        {
            eprintln!("RepoTunnel auto-connect worker warning: {error}");
        }
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
                commands::select_workspace,
                commands::list_workspaces,
                commands::get_workspace_health,
                commands::relocate_workspace,
                commands::add_workspace,
                commands::clone_git_workspace,
                commands::create_project,
                commands::remove_workspace,
                commands::update_workspace_access,
                commands::update_workspace_change_policy,
                commands::update_workspace_command_policy,
                commands::check_workspace_access,
                filesystem_commands::list_directory,
                filesystem_commands::read_file,
                filesystem_commands::search_files,
                editor_commands::editor_save_file,
                editor_commands::editor_create_file,
                editor_commands::editor_create_directory,
                editor_commands::editor_rename_entry,
                editor_commands::editor_delete_entry,
                editor_commands::preview_workspace_image,
                commands::open_workspace_path_local,
                commands::get_project_setup,
                commands::prepare_project,
                commands::get_project_memory,
                commands::get_resume_snapshot,
                commands::update_project_memory,
                commands::inspect_project,
                commands::get_workflow_readiness,
                commands::get_environment_diagnostics,
                commands::list_tool_capabilities,
                phone_commands::get_phone_discovery,
                phone_commands::pair_phone_wirelessly,
                phone_commands::get_phone_access_status,
                phone_commands::select_phone_device,
                phone_commands::set_phone_access_mode,
                phone_commands::set_phone_access_paused,
                phone_commands::get_phone_runtime_status,
                phone_commands::ensure_phone_runtime,
                phone_commands::clear_phone_runtime,
                phone_commands::probe_phone_runtime,
                phone_commands::get_phone_screen_frame,
                phone_commands::phone_tap,
                phone_commands::phone_swipe,
                phone_commands::phone_key_event,
                phone_commands::phone_type_text,
                commands::get_system_resource_snapshot,
                temp_workspace_commands::create_temp_workspace,
                temp_workspace_commands::list_temp_workspaces,
                temp_workspace_commands::inspect_temp_workspace,
                temp_workspace_commands::set_temp_workspace_preserved,
                temp_workspace_commands::cleanup_temp_workspace,
                temp_workspace_commands::temp_workspace_file_action,
                change_commands::create_file,
                change_commands::write_file,
                change_commands::patch_file,
                change_commands::create_directory,
                change_commands::rename_entry,
                change_commands::move_entry,
                change_commands::delete_entry,
                change_commands::list_changes,
                change_commands::approve_change,
                change_commands::reject_change,
                change_commands::undo_change,
                history_commands::get_version_timeline,
                history_commands::get_activity_timeline,
                history_commands::clear_version_history,
                history_commands::get_history_settings,
                history_commands::update_history_settings,
                history_commands::restore_version,
                execution_commands::get_execution_status,
                execution_commands::list_command_presets,
                execution_commands::run_workspace_command,
                execution_commands::list_command_history,
                execution_commands::approve_command,
                execution_commands::reject_command,
                commands::run_terminal_command,
                commands::run_local_terminal_command,
                commands::list_terminal_history,
                commands::approve_terminal_command,
                commands::reject_terminal_command,
                commands::start_managed_process,
                commands::start_local_managed_process,
                commands::list_managed_processes,
                commands::read_managed_process_output,
                commands::wait_managed_process,
                commands::approve_managed_process,
                commands::reject_managed_process,
                commands::stop_managed_process,
                commands::restart_managed_process,
                commands::list_launchable_applications,
                github_commands::get_github_connection_status,
                github_commands::connect_github,
                github_commands::cancel_github_connection,
                github_commands::disconnect_github,
                commands::list_deep_integrations,
                commands::set_deep_integration_enabled,
                commands::list_desktop_control_applications,
                commands::get_desktop_control_enabled,
                commands::set_desktop_control_enabled,
                commands::get_gmail_access_enabled,
                commands::set_gmail_access_enabled,
                commands::get_ai_workspace_status,
                commands::start_ai_workspace,
                commands::stop_ai_workspace,
                commands::get_ai_workspace_frame,
                commands::ai_workspace_action,
                commands::ai_workspace_sequence,
                commands::open_url,
                commands::open_workspace_path,
                commands::launch_application,
                commands::list_launch_history,
                commands::approve_launch_action,
                commands::reject_launch_action,
                commands::list_automation_browsers,
                commands::get_browser_automation_status,
                commands::start_browser_automation,
                commands::stop_browser_automation,
                commands::list_browser_tabs,
                commands::browser_open_tab,
                commands::browser_activate_tab,
                commands::browser_close_tab,
                commands::browser_navigate,
                commands::browser_click,
                commands::browser_type,
                commands::browser_scroll,
                commands::browser_reload,
                commands::browser_configure_downloads,
                commands::list_browser_downloads,
                commands::cancel_browser_download,
                commands::browser_upload_file,
                commands::browser_inspect_page,
                commands::browser_pick_element,
                commands::get_browser_visual_selection,
                commands::browser_take_screenshot,
                commands::get_browser_diagnostics,
                commands::list_browser_history,
                commands::approve_browser_action,
                commands::reject_browser_action,
                commands::inspect_media_file,
                commands::extract_media_frame,
                commands::validate_media_decode,
                // Legacy video Tauri commands intentionally not registered in lean gateway.
                // Their implementations remain for now; restore from Git history if needed.
                commands::get_monitoring_status,
                commands::start_workspace_monitoring,
                commands::stop_workspace_monitoring,
                commands::get_monitoring_snapshot,
                commands::list_monitoring_file_events,
                commands::get_git_status,
                commands::get_git_diff,
                commands::get_git_diff_check,
                commands::get_git_log,
                commands::request_git_stage,
                commands::request_git_commit,
                commands::list_git_actions,
                commands::approve_git_action,
                commands::reject_git_action,
                commands::request_git_restore_file,
                commands::get_file_info,
                commands::create_checkpoint,
                commands::list_checkpoints,
                commands::compare_checkpoint,
                commands::restore_checkpoint,
                commands::delete_checkpoint,
                commands::rename_checkpoint,
                commands::set_checkpoint_pinned,
                commands::clear_checkpoints,
                commands::run_safety_scan,
                commands::get_ai_access_status,
                commands::set_ai_access_paused,
                commands::get_gateway_status,
                commands::start_gateway,
                commands::stop_gateway,
                commands::get_public_tunnel_status,
                commands::configure_public_tunnel,
                commands::restart_public_tunnel,
                commands::provision_direct_https_certificate,
                https_setup::get_https_setup_readiness,
                https_setup::install_https_setup_wireguard_config,
                https_setup::open_https_setup_resource,
                https_setup::verify_https_setup_hostname,
                commands::clear_public_tunnel,
                commands::revoke_mcp_access,
                commands::get_chat_connection_status,
                commands::start_chat_connection,
                commands::stop_chat_connection,
                commands::get_model_hub,
                commands::refresh_model_runtime,
                commands::update_model_runtime_endpoint,
                commands::set_selected_local_model,
                commands::test_local_model,
                commands::get_model_trial,
                commands::run_model_trial,
                commands::cancel_model_trial,
                commands::list_home_conversations,
                commands::get_home_conversation,
                commands::create_home_conversation,
                commands::delete_home_conversation,
                commands::list_home_context_files,
                commands::begin_home_chat,
                commands::cancel_home_chat,
                commands::get_runtime_diagnostics,
                commands::set_launch_at_login,
                commands::get_update_status,
                commands::check_for_updates,
                commands::set_auto_update_checks,
                commands::defer_update,
                commands::install_update_and_restart,
                commands::list_team_sessions,
                commands::get_team_session,
                commands::create_team_session,
                commands::post_team_user_message,
                commands::pause_team_session,
                commands::resume_team_session,
                commands::cancel_team_session,
                commands::complete_team_session,
                commands::delete_team_session,
            ];
            handler
        })
        .setup(|app| {
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
                monitoring::stop_all_activity();
                terminal::stop_transient_activity_for_shutdown();
                browser::stop_all_activity();
                // Legacy video lifecycle is opt-in during gateway simplification.
                #[cfg(feature = "legacy_video")]
                {
                    video::stop_all_activity();
                    video_production::stop_all_activity();
                    video_preview::clear_all(app_handle);
                }
                model_hub::stop_owned_local_runtimes();
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
