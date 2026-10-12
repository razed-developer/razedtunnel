use tauri::{AppHandle, Manager};
use crate::{access::{resolve_workspace_path, AccessOperation}, ai_resources, app_state::AppState, launcher, models::Workspace};

pub(crate) fn resolve_ai_workspace_app_session(
    app: &AppHandle,
    workspace: &Workspace,
    client_key: Option<&str>,
    requested_app_session_id: Option<&str>,
) -> Result<String, String> {
    let client_key = client_key.ok_or_else(|| "AI session identity is unavailable.".to_string())?;
    let state = app.state::<AppState>();

    if let Some(app_session_id) = requested_app_session_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        if !state
            .ai_workspace
            .app_session_exists(&workspace.id, app_session_id)?
        {
            return Err(
                "That AI Workspace app_session_id is not running on this shared desktop."
                    .to_string(),
            );
        }
        let ownership = ai_resources::ai_workspace_app_ownership(
            &workspace.id,
            app_session_id,
            Some(client_key),
        )?;
        if ownership.owner_session_id.is_none() || ownership.owned_by_current_session {
            ai_resources::claim_ai_workspace_app(&workspace.id, client_key, app_session_id)?;
            return Ok(app_session_id.to_string());
        }
        if ownership.owner_lease_active {
            return Err(format!(
                "AI Workspace app session {app_session_id} is actively owned by another AI session ({}).",
                ownership
                    .owner_session_id
                    .as_deref()
                    .unwrap_or("unknown-owner")
            ));
        }
        ai_resources::reclaim_ai_workspace_app_if_stale(&workspace.id, client_key, app_session_id)?;
        return Ok(app_session_id.to_string());
    }

    let mut owned = ai_resources::owned_ai_workspace_app_sessions(&workspace.id, client_key)?;
    owned.retain(|app_session_id| {
        state
            .ai_workspace
            .app_session_exists(&workspace.id, app_session_id)
            .unwrap_or(false)
    });
    match owned.as_slice() {
        [only] => {
            ai_resources::assert_ai_workspace_app_owned(&workspace.id, client_key, only)?;
            return Ok(only.clone());
        }
        sessions if sessions.len() > 1 => {
            return Err(format!(
                "This AI owns multiple AI Workspace app sessions ({}). Specify app_session_id so input cannot be routed to the wrong application.",
                sessions.join(", ")
            ));
        }
        _ => {}
    }

    let status = state.ai_workspace.status(app, &workspace.id)?;
    if status.applications.len() == 1 {
        let app_session_id = status.applications[0].app_session_id.clone();
        let ownership = ai_resources::ai_workspace_app_ownership(
            &workspace.id,
            &app_session_id,
            Some(client_key),
        )?;
        if ownership.owner_session_id.is_none() || ownership.owned_by_current_session {
            ai_resources::claim_ai_workspace_app(&workspace.id, client_key, &app_session_id)?;
            return Ok(app_session_id);
        }
        if !ownership.owner_lease_active {
            ai_resources::reclaim_ai_workspace_app_if_stale(
                &workspace.id,
                client_key,
                &app_session_id,
            )?;
            return Ok(app_session_id);
        }
        return Err(format!(
            "The only running AI Workspace app is still actively owned by another AI session ({}). Pass new_instance=true to ai_workspace_session action=start only if a genuinely separate app instance is required.",
            ownership
                .owner_session_id
                .as_deref()
                .unwrap_or("unknown-owner")
        ));
    }

    Err(
        "Multiple AI Workspace applications are running. Specify the app_session_id owned by this AI."
            .to_string(),
    )
}

pub(crate) fn resolve_ai_workspace_visible_path(
    workspace: &Workspace,
    relative_path: &str,
) -> Result<String, String> {
    let path = resolve_workspace_path(workspace, relative_path, AccessOperation::Read, true)?;
    Ok(path.to_string_lossy().into_owned())
}

pub(crate) fn ai_workspace_type_text(
    workspace: &Workspace,
    text: Option<&str>,
    workspace_relative_path: Option<&str>,
) -> Result<Option<String>, String> {
    match (text, workspace_relative_path) {
        (Some(_), Some(_)) => Err(
            "AI Workspace type input accepts either text or workspace_relative_path, not both."
                .to_string(),
        ),
        (Some(text), None) => Ok(Some(text.to_string())),
        (None, Some(relative_path)) => {
            resolve_ai_workspace_visible_path(workspace, relative_path).map(Some)
        }
        (None, None) => Ok(None),
    }
}

pub(crate) fn ai_workspace_terminal_browser_launch(text: &str) -> bool {
    const BROWSER_EXECUTABLES: &[&str] = &[
        "chrome",
        "chromium",
        "chromium-browser",
        "google-chrome",
        "google-chrome-stable",
        "brave",
        "brave-browser",
        "microsoft-edge",
        "microsoft-edge-stable",
        "firefox",
    ];
    const SHELL_PREFIXES: &[&str] = &["env", "nohup", "exec", "command", "setsid", "sudo"];

    let normalized = text
        .replace("&&", "\n")
        .replace("||", "\n")
        .replace(';', "\n");

    normalized.lines().any(|line| {
        let mut tokens = line.split_whitespace().peekable();
        while let Some(token) = tokens.peek().copied() {
            let cleaned = token.trim_matches(|ch: char| {
                matches!(ch, '\'' | '"' | '(' | ')' | '{' | '}' | '[' | ']')
            });
            if cleaned.contains('=') && !cleaned.starts_with('=') {
                tokens.next();
                continue;
            }
            if SHELL_PREFIXES.contains(&cleaned) {
                tokens.next();
                continue;
            }
            break;
        }

        let Some(command) = tokens.next() else {
            return false;
        };
        let command = command
            .trim_matches(|ch: char| matches!(ch, '\'' | '"' | '(' | ')' | '{' | '}' | '[' | ']'));
        let basename = std::path::Path::new(command)
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(command)
            .to_ascii_lowercase();
        BROWSER_EXECUTABLES.contains(&basename.as_str())
    })
}

pub(crate) fn ensure_ai_workspace_terminal_text_allowed(
    app: &AppHandle,
    workspace: &Workspace,
    app_session_id: &str,
    text: Option<&str>,
) -> Result<(), String> {
    let Some(text) = text else {
        return Ok(());
    };
    if !ai_workspace_terminal_browser_launch(text) {
        return Ok(());
    }

    let state = app.state::<AppState>();
    let status = state.ai_workspace.status(app, &workspace.id)?;
    let Some(application) = status
        .applications
        .iter()
        .find(|application| application.app_session_id == app_session_id)
    else {
        return Ok(());
    };
    let is_terminal = launcher::list_applications()
        .into_iter()
        .find(|candidate| candidate.id == application.application_id)
        .is_some_and(|candidate| candidate.category == "terminal");
    if !is_terminal {
        return Ok(());
    }

    Err(
        "RepoTunnel blocked launching a browser from AI Workspace Terminal. Use RepoTunnel managed browser automation for Chrome/Chromium/Brave/Edge/Firefox testing instead."
            .to_string(),
    )
}

pub(crate) fn ai_workspace_session_payload(
    workspace: &Workspace,
    mut status: crate::ai_workspace::AiWorkspaceStatus,
    client_key: Option<&str>,
) -> Result<serde_json::Value, String> {
    let mut owned_applications = Vec::new();
    let mut app_ownership = Vec::new();
    for application in &status.applications {
        let ownership = ai_resources::ai_workspace_app_ownership(
            &workspace.id,
            &application.app_session_id,
            client_key,
        )?;
        if ownership.owned_by_current_session {
            owned_applications.push(application.clone());
            app_ownership.push(ownership);
        }
    }

    let owned_ids = owned_applications
        .iter()
        .map(|application| application.app_session_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    if status
        .last_started_app_session_id
        .as_deref()
        .is_some_and(|id| !owned_ids.contains(id))
    {
        status.last_started_app_session_id = None;
    }

    if let Some(first_owned) = owned_applications.first() {
        status.application_id = Some(first_owned.application_id.clone());
        status.application_name = Some(first_owned.application_name.clone());
        status.started_at = Some(first_owned.started_at);
    } else {
        status.application_id = None;
        status.application_name = None;
        status.started_at = None;
    }
    status.applications = owned_applications;

    let mut value = serde_json::to_value(status)
        .map_err(|error| format!("Could not encode AI Workspace status: {error}"))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "AI Workspace status was not a JSON object.".to_string())?;
    object.insert(
        "workspaceVisibleRoot".to_string(),
        serde_json::json!(workspace.path),
    );
    object.insert(
        "ownershipMode".to_string(),
        serde_json::json!("perApplication"),
    );
    object.insert("sharedDesktop".to_string(), serde_json::json!(true));
    object.insert(
        "appOwnership".to_string(),
        serde_json::to_value(app_ownership)
            .map_err(|error| format!("Could not encode AI Workspace app ownership: {error}"))?,
    );
    Ok(value)
}

pub(crate) fn require_ai_workspace_session_match(
    status: &crate::ai_workspace::AiWorkspaceStatus,
    session_id: Option<&str>,
) -> Result<(), String> {
    let expected = status
        .session_id
        .as_deref()
        .ok_or_else(|| "No AI Workspace is running for this project.".to_string())?;
    let supplied = session_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "The running AI Workspace session_id is required for stale recovery.".to_string()
        })?;
    if supplied != expected {
        return Err(
            "The supplied AI Workspace session_id does not match the running isolated session."
                .to_string(),
        );
    }
    Ok(())
}

