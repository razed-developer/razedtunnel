//! Counters for the two instrumented MCP task wrappers, not all tool dispatches.
use std::sync::atomic::{AtomicU64, Ordering};

static TASK_TOOL_SUCCESSES: AtomicU64 = AtomicU64::new(0);
static TASK_TOOL_FAILURES: AtomicU64 = AtomicU64::new(0);

pub(crate) fn task_tool_outcomes() -> (u64, u64) {
    (
        TASK_TOOL_SUCCESSES.load(Ordering::Relaxed),
        TASK_TOOL_FAILURES.load(Ordering::Relaxed),
    )
}

pub(crate) fn record_task_outcome(succeeded: bool) {
    if succeeded {
        TASK_TOOL_SUCCESSES.fetch_add(1, Ordering::Relaxed);
    } else {
        TASK_TOOL_FAILURES.fetch_add(1, Ordering::Relaxed);
    }
}
