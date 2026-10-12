use tauri::AppHandle;
use crate::{activity, terminal, models::{TerminalCommandOutcome, TerminalCommandRecord, ManagedProcessOutcome, ManagedProcessRecord, ManagedProcessOutput, ManagedProcessWaitResult}, projects::approved_workspace};

#[tauri::command]
pub async fn run_terminal_command(
    app: AppHandle,
    workspace_id: String,
    command: String,
    cwd: Option<String>,
    timeout_seconds: Option<u64>,
    env: Option<std::collections::BTreeMap<String, String>>,
) -> Result<TerminalCommandOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let workspace = approved_workspace(&app, &workspace_id)?;
        terminal::request_terminal_command(
            &app,
            &workspace,
            command,
            cwd,
            timeout_seconds,
            env.unwrap_or_default(),
            false,
            false,
        )
    })
    .await
    .map_err(|error| format!("The terminal command task could not complete: {error}"))?
}

#[tauri::command]
pub async fn run_local_terminal_command(
    app: AppHandle,
    workspace_id: String,
    command: String,
    cwd: Option<String>,
    timeout_seconds: Option<u64>,
) -> Result<TerminalCommandOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let workspace = approved_workspace(&app, &workspace_id)?;
        let outcome = terminal::run_local_terminal_command(
            &app,
            &workspace,
            command,
            cwd,
            timeout_seconds,
            std::collections::BTreeMap::new(),
        )?;
        activity::sync_terminal(&app, &outcome.command);
        Ok(outcome)
    })
    .await
    .map_err(|error| format!("The local terminal task could not complete: {error}"))?
}

#[tauri::command]
pub fn list_terminal_history(
    app: AppHandle,
    workspace_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<TerminalCommandRecord>, String> {
    terminal::list_terminal_history(&app, workspace_id.as_deref(), limit.unwrap_or(40))
}

#[tauri::command]
pub async fn approve_terminal_command(
    app: AppHandle,
    command_id: String,
) -> Result<TerminalCommandRecord, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let record = terminal::get_terminal_command(&app, &command_id)?;
        let workspace = approved_workspace(&app, &record.workspace_id)?;
        let approved = terminal::approve_terminal_command(&app, &workspace, &command_id)?;
        activity::sync_terminal(&app, &approved);
        Ok(approved)
    })
    .await
    .map_err(|error| format!("The terminal approval task could not complete: {error}"))?
}

#[tauri::command]
pub fn reject_terminal_command(
    app: AppHandle,
    command_id: String,
) -> Result<TerminalCommandRecord, String> {
    let record = terminal::reject_terminal_command(&app, &command_id)?;
    activity::sync_terminal(&app, &record);
    Ok(record)
}

#[tauri::command]
pub fn start_managed_process(
    app: AppHandle,
    workspace_id: String,
    command: String,
    cwd: Option<String>,
    label: Option<String>,
    env: Option<std::collections::BTreeMap<String, String>>,
) -> Result<ManagedProcessOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    terminal::request_process_start(
        &app,
        &workspace,
        command,
        cwd,
        label,
        env.unwrap_or_default(),
        false,
    )
}

#[tauri::command]
pub fn start_local_managed_process(
    app: AppHandle,
    workspace_id: String,
    command: String,
    cwd: Option<String>,
    label: Option<String>,
) -> Result<ManagedProcessOutcome, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let outcome = terminal::start_local_process(
        &app,
        &workspace,
        command,
        cwd,
        label,
        std::collections::BTreeMap::new(),
    )?;
    activity::sync_process(&app, &outcome.process);
    Ok(outcome)
}

#[tauri::command]
pub fn list_managed_processes(
    app: AppHandle,
    workspace_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<ManagedProcessRecord>, String> {
    terminal::list_processes(&app, workspace_id.as_deref(), limit.unwrap_or(60))
}

#[tauri::command]
pub fn read_managed_process_output(
    app: AppHandle,
    process_id: String,
    stdout_offset: Option<u64>,
    stderr_offset: Option<u64>,
    max_bytes: Option<usize>,
) -> Result<ManagedProcessOutput, String> {
    terminal::read_process_output(
        &app,
        &process_id,
        stdout_offset.unwrap_or(0),
        stderr_offset.unwrap_or(0),
        max_bytes.unwrap_or(64 * 1024),
    )
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn wait_managed_process(
    app: AppHandle,
    process_id: String,
    success_patterns: Vec<String>,
    failure_patterns: Vec<String>,
    timeout_seconds: Option<u64>,
    stdout_offset: Option<u64>,
    stderr_offset: Option<u64>,
    max_bytes: Option<usize>,
) -> Result<ManagedProcessWaitResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        terminal::wait_process(
            &app,
            &process_id,
            success_patterns,
            failure_patterns,
            timeout_seconds.unwrap_or(120),
            stdout_offset.unwrap_or(0),
            stderr_offset.unwrap_or(0),
            max_bytes.unwrap_or(64 * 1024),
        )
    })
    .await
    .map_err(|error| format!("The managed-process wait task could not complete: {error}"))?
}

#[tauri::command]
pub fn approve_managed_process(
    app: AppHandle,
    process_id: String,
) -> Result<ManagedProcessRecord, String> {
    let record = terminal::get_process(&app, &process_id)?;
    let workspace = approved_workspace(&app, &record.workspace_id)?;
    let approved = terminal::approve_process_start(&app, &workspace, &process_id)?;
    activity::sync_process(&app, &approved);
    Ok(approved)
}

#[tauri::command]
pub fn reject_managed_process(
    app: AppHandle,
    process_id: String,
) -> Result<ManagedProcessRecord, String> {
    let record = terminal::reject_process_start(&app, &process_id)?;
    activity::sync_process(&app, &record);
    Ok(record)
}

#[tauri::command]
pub async fn stop_managed_process(
    app: AppHandle,
    process_id: String,
    force: Option<bool>,
) -> Result<ManagedProcessRecord, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let record = terminal::stop_process(&app, &process_id, force.unwrap_or(false))?;
        activity::sync_process(&app, &record);
        Ok(record)
    })
    .await
    .map_err(|error| format!("The process stop task could not complete: {error}"))?
}

#[tauri::command]
pub async fn restart_managed_process(
    app: AppHandle,
    process_id: String,
) -> Result<ManagedProcessRecord, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let record = terminal::get_process(&app, &process_id)?;
        let workspace = approved_workspace(&app, &record.workspace_id)?;
        let restarted = terminal::restart_process(&app, &workspace, &process_id)?;
        activity::sync_process(&app, &restarted);
        Ok(restarted)
    })
    .await
    .map_err(|error| format!("The process restart task could not complete: {error}"))?
}

