use tauri::AppHandle;
use crate::{activity, models::ActivityTimeline};

/// Read-only desktop activity feed. Approval mutations remain MCP-managed
/// until their authorization and audit paths are explicitly reviewed.
#[tauri::command]
pub fn get_activity_timeline(app: AppHandle) -> Result<ActivityTimeline, String> {
    activity::timeline(&app, None)
}
