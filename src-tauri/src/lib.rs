mod access;
mod activity;
mod ai_resources;
mod ai_workspace;
mod ai_workspace_commands;
mod app_state;
mod workspace_management_commands;
mod workspace_policy_commands;
mod safety_commands;
mod workspace_diagnostics_commands;
mod project_lifecycle_commands;
mod chat_connection_commands;
mod gateway_commands;
mod ai_access_commands;
mod browser;
mod browser_commands;
mod changes;
mod change_commands;
mod chatgpt_bridge;
mod checkpoint;
mod checkpoint_commands;
#[cfg(feature = "legacy_video")]
mod legacy_video_commands;
mod connection;
mod continuity;
mod conversation;
mod conversation_commands;
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
mod git_commands;
mod github;
mod github_commands;
mod gmail_access;
mod hardening;
mod history_commands;
mod https_setup;
mod integrations;
mod integration_commands;
mod large_project_read;
mod launcher;
mod launcher_commands;
#[cfg(any(target_os = "macos", test))]
mod macos_ax;
mod mcp_auth;
mod mcp_server;
mod mcp_git_helpers;
mod mcp_time;
mod mcp_workspace_params;
mod mcp_continuation_params;
mod mcp_filesystem_params;
mod mcp_execution_params;
mod mcp_launch_params;
mod mcp_phone_params;
mod mcp_ai_workspace_params;
mod mcp_desktop_params;
mod mcp_browser_params;
mod mcp_phone_helpers;
mod mcp_phone_transactions;
mod mcp_request_identity;
mod mcp_continuation_fallback;
mod mcp_browser_access;
mod media_inspection;
mod media_commands;
mod media_tools;
mod model_hub;
mod model_hub_commands;
mod model_trial;
mod model_trial_commands;
mod models;
mod monitoring;
mod monitoring_commands;
mod phone;
mod phone_commands;
mod platform_sandbox;
mod projects;
mod project_context;
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
mod system_resources;
mod team;
mod team_commands;
mod temp_workspace;
mod temp_workspace_commands;
mod terminal;
mod terminal_commands;
mod updates;
mod runtime_commands;
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
                workspace_management_commands::select_workspace,
                workspace_management_commands::list_workspaces,
                workspace_management_commands::get_workspace_health,
                workspace_management_commands::relocate_workspace,
                workspace_management_commands::add_workspace,
                workspace_management_commands::clone_git_workspace,
                workspace_management_commands::create_project,
                workspace_management_commands::remove_workspace,
                workspace_policy_commands::update_workspace_access,
                workspace_policy_commands::update_workspace_change_policy,
                workspace_policy_commands::update_workspace_command_policy,
                workspace_policy_commands::check_workspace_access,
                filesystem_commands::list_directory,
                filesystem_commands::read_file,
                filesystem_commands::search_files,
                editor_commands::editor_save_file,
                editor_commands::editor_create_file,
                editor_commands::editor_create_directory,
                editor_commands::editor_rename_entry,
                editor_commands::editor_delete_entry,
                editor_commands::preview_workspace_image,
                workspace_policy_commands::open_workspace_path_local,
                project_lifecycle_commands::get_project_setup,
                project_lifecycle_commands::prepare_project,
                project_lifecycle_commands::get_project_memory,
                project_lifecycle_commands::get_resume_snapshot,
                project_lifecycle_commands::update_project_memory,
                workspace_diagnostics_commands::inspect_project,
                workspace_diagnostics_commands::get_workflow_readiness,
                workspace_diagnostics_commands::get_environment_diagnostics,
                workspace_diagnostics_commands::list_tool_capabilities,
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
                workspace_diagnostics_commands::get_system_resource_snapshot,
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
                terminal_commands::run_terminal_command,
                terminal_commands::run_local_terminal_command,
                terminal_commands::list_terminal_history,
                terminal_commands::approve_terminal_command,
                terminal_commands::reject_terminal_command,
                terminal_commands::start_managed_process,
                terminal_commands::start_local_managed_process,
                terminal_commands::list_managed_processes,
                terminal_commands::read_managed_process_output,
                terminal_commands::wait_managed_process,
                terminal_commands::approve_managed_process,
                terminal_commands::reject_managed_process,
                terminal_commands::stop_managed_process,
                terminal_commands::restart_managed_process,
                workspace_diagnostics_commands::list_launchable_applications,
                github_commands::get_github_connection_status,
                github_commands::connect_github,
                github_commands::cancel_github_connection,
                github_commands::disconnect_github,
                integration_commands::list_deep_integrations,
                integration_commands::set_deep_integration_enabled,
                integration_commands::list_desktop_control_applications,
                integration_commands::get_desktop_control_enabled,
                integration_commands::set_desktop_control_enabled,
                integration_commands::get_gmail_access_enabled,
                integration_commands::set_gmail_access_enabled,
                ai_workspace_commands::get_ai_workspace_status,
                ai_workspace_commands::start_ai_workspace,
                ai_workspace_commands::stop_ai_workspace,
                ai_workspace_commands::get_ai_workspace_frame,
                ai_workspace_commands::ai_workspace_action,
                ai_workspace_commands::ai_workspace_sequence,
                launcher_commands::open_url,
                launcher_commands::open_workspace_path,
                launcher_commands::launch_application,
                launcher_commands::list_launch_history,
                launcher_commands::approve_launch_action,
                launcher_commands::reject_launch_action,
                browser_commands::list_automation_browsers,
                browser_commands::get_browser_automation_status,
                browser_commands::start_browser_automation,
                browser_commands::stop_browser_automation,
                browser_commands::list_browser_tabs,
                browser_commands::browser_open_tab,
                browser_commands::browser_activate_tab,
                browser_commands::browser_close_tab,
                browser_commands::browser_navigate,
                browser_commands::browser_click,
                browser_commands::browser_type,
                browser_commands::browser_scroll,
                browser_commands::browser_reload,
                browser_commands::browser_configure_downloads,
                browser_commands::list_browser_downloads,
                browser_commands::cancel_browser_download,
                browser_commands::browser_upload_file,
                browser_commands::browser_inspect_page,
                browser_commands::browser_pick_element,
                browser_commands::get_browser_visual_selection,
                browser_commands::browser_take_screenshot,
                browser_commands::get_browser_diagnostics,
                browser_commands::list_browser_history,
                browser_commands::approve_browser_action,
                browser_commands::reject_browser_action,
                media_commands::inspect_media_file,
                media_commands::extract_media_frame,
                media_commands::validate_media_decode,
                // Legacy video Tauri commands intentionally not registered in lean gateway.
                // Their implementations remain for now; restore from Git history if needed.
                monitoring_commands::get_monitoring_status,
                monitoring_commands::start_workspace_monitoring,
                monitoring_commands::stop_workspace_monitoring,
                monitoring_commands::get_monitoring_snapshot,
                monitoring_commands::list_monitoring_file_events,
                git_commands::get_git_status,
                git_commands::get_git_diff,
                git_commands::get_git_diff_check,
                git_commands::get_git_log,
                git_commands::request_git_stage,
                git_commands::request_git_commit,
                git_commands::list_git_actions,
                git_commands::approve_git_action,
                git_commands::reject_git_action,
                git_commands::request_git_restore_file,
                workspace_diagnostics_commands::get_file_info,
                checkpoint_commands::create_checkpoint,
                checkpoint_commands::list_checkpoints,
                checkpoint_commands::compare_checkpoint,
                checkpoint_commands::restore_checkpoint,
                checkpoint_commands::delete_checkpoint,
                checkpoint_commands::rename_checkpoint,
                checkpoint_commands::set_checkpoint_pinned,
                checkpoint_commands::clear_checkpoints,
                safety_commands::run_safety_scan,
                ai_access_commands::get_ai_access_status,
                ai_access_commands::set_ai_access_paused,
                gateway_commands::get_gateway_status,
                gateway_commands::start_gateway,
                gateway_commands::stop_gateway,
                public_tunnel_commands::get_public_tunnel_status,
                public_tunnel_commands::configure_public_tunnel,
                public_tunnel_commands::restart_public_tunnel,
                public_tunnel_commands::provision_direct_https_certificate,
                https_setup::get_https_setup_readiness,
                https_setup::install_https_setup_wireguard_config,
                https_setup::open_https_setup_resource,
                https_setup::verify_https_setup_hostname,
                public_tunnel_commands::clear_public_tunnel,
                public_tunnel_commands::revoke_mcp_access,
                chat_connection_commands::get_chat_connection_status,
                chat_connection_commands::start_chat_connection,
                chat_connection_commands::stop_chat_connection,
                model_hub_commands::get_model_hub,
                model_hub_commands::refresh_model_runtime,
                model_hub_commands::update_model_runtime_endpoint,
                model_hub_commands::set_selected_local_model,
                model_hub_commands::test_local_model,
                model_trial_commands::get_model_trial,
                model_trial_commands::run_model_trial,
                model_trial_commands::cancel_model_trial,
                conversation_commands::list_home_conversations,
                conversation_commands::get_home_conversation,
                conversation_commands::create_home_conversation,
                conversation_commands::delete_home_conversation,
                conversation_commands::list_home_context_files,
                conversation_commands::begin_home_chat,
                conversation_commands::cancel_home_chat,
                runtime_commands::get_runtime_diagnostics,
                runtime_commands::set_launch_at_login,
                runtime_commands::get_update_status,
                runtime_commands::check_for_updates,
                runtime_commands::set_auto_update_checks,
                runtime_commands::defer_update,
                runtime_commands::install_update_and_restart,
                team_commands::list_team_sessions,
                team_commands::get_team_session,
                team_commands::create_team_session,
                team_commands::post_team_user_message,
                team_commands::pause_team_session,
                team_commands::resume_team_session,
                team_commands::cancel_team_session,
                team_commands::complete_team_session,
                team_commands::delete_team_session,
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
