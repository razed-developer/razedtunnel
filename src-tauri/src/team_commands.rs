use tauri::AppHandle;
use crate::{activity, team, projects::approved_workspace, models::{TeamSessionSummary, TeamSnapshot, ActivityKind, ActivityStatus}};

#[tauri::command]
pub fn list_team_sessions(
    app: AppHandle,
    workspace_id: Option<String>,
) -> Result<Vec<TeamSessionSummary>, String> {
    team::list_sessions(&app, workspace_id.as_deref())
}

#[tauri::command]
pub fn get_team_session(app: AppHandle, session_id: String) -> Result<TeamSnapshot, String> {
    team::get_snapshot(&app, &session_id, None)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn create_team_session(
    app: AppHandle,
    workspace_id: String,
    goal: String,
    success_criteria: Vec<String>,
    agent_a_name: String,
    agent_a_role: String,
    agent_b_name: String,
    agent_b_role: String,
) -> Result<TeamSnapshot, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let snapshot = team::create_session(
        &app,
        &workspace,
        goal,
        success_criteria,
        agent_a_name,
        agent_a_role,
        agent_b_name,
        agent_b_role,
    )?;
    let _ = activity::record(
        &app,
        &workspace,
        None,
        ActivityKind::Team,
        "teamCreate",
        "Created a two-agent Team Mode session",
        Some(snapshot.session.goal.clone()),
        ActivityStatus::Succeeded,
        Some(snapshot.session.id.clone()),
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn post_team_user_message(
    app: AppHandle,
    session_id: String,
    text: String,
) -> Result<TeamSnapshot, String> {
    let before = team::get_snapshot(&app, &session_id, None)?;
    let workspace = approved_workspace(&app, &before.session.workspace_id)?;
    let snapshot = team::post_user_message(&app, &session_id, text)?;
    let _ = activity::record(
        &app,
        &workspace,
        None,
        ActivityKind::Team,
        "teamUserMessage",
        "Posted guidance to the AI team",
        None,
        ActivityStatus::Succeeded,
        Some(session_id),
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn pause_team_session(app: AppHandle, session_id: String) -> Result<TeamSnapshot, String> {
    let before = team::get_snapshot(&app, &session_id, None)?;
    let workspace = approved_workspace(&app, &before.session.workspace_id)?;
    let snapshot = team::pause_session(&app, &session_id)?;
    let _ = activity::record(
        &app,
        &workspace,
        None,
        ActivityKind::Team,
        "teamPause",
        "Paused AI Team Mode",
        None,
        ActivityStatus::Stopped,
        Some(session_id),
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn resume_team_session(app: AppHandle, session_id: String) -> Result<TeamSnapshot, String> {
    let before = team::get_snapshot(&app, &session_id, None)?;
    let workspace = approved_workspace(&app, &before.session.workspace_id)?;
    let snapshot = team::resume_session(&app, &session_id)?;
    let _ = activity::record(
        &app,
        &workspace,
        None,
        ActivityKind::Team,
        "teamResume",
        "Resumed AI Team Mode",
        None,
        ActivityStatus::Succeeded,
        Some(session_id),
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn cancel_team_session(
    app: AppHandle,
    session_id: String,
    summary: Option<String>,
) -> Result<TeamSnapshot, String> {
    let before = team::get_snapshot(&app, &session_id, None)?;
    let workspace = approved_workspace(&app, &before.session.workspace_id)?;
    let snapshot = team::cancel_session(&app, &session_id, summary)?;
    let _ = activity::record(
        &app,
        &workspace,
        None,
        ActivityKind::Team,
        "teamCancel",
        "Cancelled AI Team Mode",
        snapshot.session.completion_summary.clone(),
        ActivityStatus::Stopped,
        Some(session_id),
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn complete_team_session(
    app: AppHandle,
    session_id: String,
    summary: String,
) -> Result<TeamSnapshot, String> {
    let before = team::get_snapshot(&app, &session_id, None)?;
    let workspace = approved_workspace(&app, &before.session.workspace_id)?;
    let snapshot = team::complete_session(&app, &session_id, None, summary)?;
    let _ = activity::record(
        &app,
        &workspace,
        None,
        ActivityKind::Team,
        "teamEnd",
        "Ended persistent AI Team",
        snapshot.session.completion_summary.clone(),
        ActivityStatus::Succeeded,
        Some(session_id),
    );
    Ok(snapshot)
}

#[tauri::command]
pub fn delete_team_session(app: AppHandle, session_id: String) -> Result<(), String> {
    team::delete_session(&app, &session_id)
}

