use tauri::AppHandle;
use crate::{changes, execution, git, project_index, workflow, projects::approved_workspace, models::{SafetyScanCheck, SafetyScanResult, WorkspaceChangePolicy}};

#[tauri::command]
pub fn run_safety_scan(app: AppHandle, workspace_id: String) -> Result<SafetyScanResult, String> {
    let workspace = approved_workspace(&app, &workspace_id)?;
    let snapshot = project_index::project_snapshot(&workspace, 1600)?;
    let readiness = workflow::readiness(&workspace);
    let sandbox = execution::execution_status();
    let git_status = git::repository_status(&workspace);
    let pending_reviews = changes::list_changes(&app, Some(&workspace.id), 100)?
        .into_iter()
        .filter(|change| change.status == crate::models::ChangeStatus::Pending)
        .count();

    let mut ignored_items = vec![
        ".gitignore and .ignore rules are respected.".to_string(),
        "Generated dependency/build folders are excluded from AI discovery and checkpoints."
            .to_string(),
    ];
    ignored_items.extend(
        snapshot
            .overview
            .ignored_entries
            .iter()
            .map(|entry| format!("Excluded: {entry}")),
    );
    let unlisted_ignored = snapshot
        .overview
        .ignored_entry_count
        .saturating_sub(snapshot.overview.ignored_entries.len());
    if unlisted_ignored > 0 {
        ignored_items.push(format!(
            "{unlisted_ignored} additional ignored or protected entries were excluded without listing sensitive path names."
        ));
    }
    ignored_items.push("Ignored content is not copied into RepoTunnel checkpoints.".to_string());

    let mut checks = vec![
        SafetyScanCheck {
            key: "workspace-boundary".to_string(),
            title: "Workspace boundary".to_string(),
            status: "pass".to_string(),
            detail: "AI file access is restricted to this approved project root; parent traversal and symlink escapes are blocked.".to_string(),
            items: vec![format!("Approved root: {}", workspace.path), "Parent-directory traversal blocked".to_string(), "Symlink escapes outside the approved root blocked".to_string()],
        },
        SafetyScanCheck {
            key: "secret-protection".to_string(),
            title: "Secret protection".to_string(),
            status: "pass".to_string(),
            detail: "Common credential files, private keys, and .env secrets are blocked from AI file access.".to_string(),
            items: vec![".env and .env.* protected".to_string(), "Private-key formats (.pem, .key, .p12, .pfx, keystores) protected".to_string(), "Credential/config files such as .npmrc and service-account files protected".to_string()],
        },
        SafetyScanCheck {
            key: "version-protection".to_string(),
            title: "Version protection".to_string(),
            status: "pass".to_string(),
            detail: if workspace.access_mode == crate::models::WorkspaceAccessMode::ReadOnly {
                "This project is read-only; AI edits are blocked.".to_string()
            } else if workspace.change_policy == WorkspaceChangePolicy::Review {
                "AI file proposals wait for local approval; approved edits are saved into reversible local version history.".to_string()
            } else {
                "Compatible AI file edits apply automatically and RepoTunnel saves reversible local versions.".to_string()
            },
            items: vec![format!("Access mode: {:?}", workspace.access_mode), "Automatic local version history enabled".to_string(), "Previous and later saved versions remain available after restore".to_string()],
        },
        SafetyScanCheck {
            key: "ignored-content".to_string(),
            title: "Ignored & generated content".to_string(),
            status: "pass".to_string(),
            detail: format!(
                "{} ignored entries were excluded from the project scan; safe examples are listed below when available.",
                snapshot.overview.ignored_entry_count
            ),
            items: ignored_items,
        },
        SafetyScanCheck {
            key: "sandbox".to_string(),
            title: "Command sandbox".to_string(),
            status: if sandbox.sandbox_available { "pass" } else { "warning" }.to_string(),
            detail: sandbox.message.clone().unwrap_or_else(|| if sandbox.sandbox_available {
                "The native OS sandbox is available for isolated project checks.".to_string()
            } else {
                "The native OS sandbox is unavailable, so project commands will be refused.".to_string()
            }),
            items: vec![sandbox.sandbox_version.clone().unwrap_or_else(|| "Native OS sandbox not detected".to_string()), "Network disabled for disposable project checks".to_string(), "Checks run in a disposable project copy".to_string()],
        },
        SafetyScanCheck {
            key: "git".to_string(),
            title: "Git protection".to_string(),
            status: if git_status.available { "pass" } else { "warning" }.to_string(),
            detail: if git_status.available {
                format!("Git is available{}; staging and commits require local approval.", git_status.branch.as_ref().map(|branch| format!(" on {branch}")).unwrap_or_default())
            } else {
                git_status.message.clone().unwrap_or_else(|| "This project is not currently a Git repository.".to_string())
            },
            items: vec![git_status.branch.as_ref().map(|branch| format!("Branch: {branch}")).unwrap_or_else(|| "No Git branch detected".to_string()), format!("Staged: {} · Unstaged: {} · Untracked: {}", git_status.staged_count, git_status.unstaged_count, git_status.untracked_count), "Staging and commits require local approval".to_string()],
        },
    ];

    if snapshot.overview.truncated {
        checks.push(SafetyScanCheck {
            key: "index-limit".to_string(),
            title: "Project size".to_string(),
            status: "warning".to_string(),
            detail: "The safety scan reached its project-index display limit; core access protections remain active.".to_string(),
            items: vec![format!("Accessible files counted: {}", snapshot.overview.file_count), format!("Ignored entries: {}", snapshot.overview.ignored_entry_count)],
        });
    }

    let level = if checks.iter().any(|check| check.status == "warning")
        || readiness.level != crate::models::WorkflowReadinessLevel::Ready
    {
        "attention"
    } else {
        "protected"
    };

    Ok(SafetyScanResult {
        workspace_id: workspace.id,
        workspace_name: workspace.name,
        level: level.to_string(),
        file_count: snapshot.overview.file_count,
        ignored_entry_count: snapshot.overview.ignored_entry_count,
        pending_reviews,
        checks,
    })
}

