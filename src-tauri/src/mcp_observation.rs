use tauri::AppHandle;
use crate::{activity, models::{ActivityKind, ActivityStatus, Workspace}};

pub(crate) fn record_observation(
    app: &AppHandle,
    workspace: &Workspace,
    trace_group_id: Option<&str>,
    kind: ActivityKind,
    action: &str,
    summary: impl Into<String>,
    detail: Option<String>,
) {
    let _ = activity::record(
        app,
        workspace,
        trace_group_id,
        kind,
        action,
        summary.into(),
        detail,
        ActivityStatus::Observed,
        None,
    );
}

pub(crate) fn monitoring_file_detail(events: &[crate::models::MonitoringFileEvent]) -> Option<String> {
    if events.is_empty() {
        return None;
    }
    let mut lines = events
        .iter()
        .take(20)
        .map(|event| format!("{:?}: {}", event.kind, event.path))
        .collect::<Vec<_>>();
    if events.len() > lines.len() {
        lines.push(format!("…and {} more", events.len() - lines.len()));
    }
    Some(lines.join("\n"))
}

