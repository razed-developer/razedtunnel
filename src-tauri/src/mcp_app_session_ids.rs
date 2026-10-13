//! Unique reservation IDs for AI Workspace app sessions started through MCP.
use std::sync::atomic::{AtomicU64, Ordering};

static APP_SESSION_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(crate) fn next_app_session_id() -> String {
    let sequence = APP_SESSION_SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1;
    format!(
        "aiwapp-mcp-{}-{}-{sequence}",
        std::process::id(),
        crate::mcp_time::unix_epoch_millis()
    )
}
