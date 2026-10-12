//! Approved-project path validation and collision-resistant registry IDs.
use std::{path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};
use crate::models::Workspace;

pub(crate) fn canonical_workspace_path(path: &str) -> Result<PathBuf, String> {
    let requested = Path::new(path);

    if !requested.exists() {
        return Err("The selected project folder no longer exists.".to_string());
    }

    if !requested.is_dir() {
        return Err("The selected path is not a folder.".to_string());
    }

    requested
        .canonicalize()
        .map_err(|error| format!("Could not resolve the selected project folder: {error}"))
}

pub(crate) fn workspace_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Project")
        .to_string()
}

pub(crate) fn allocate_unique_workspace_id_with<F>(
    existing: &[Workspace],
    mut fill_random: F,
) -> Result<String, String>
where
    F: FnMut(&mut [u8; 16]) -> Result<(), String>,
{
    for _ in 0..16 {
        let mut random = [0u8; 16];
        fill_random(&mut random)?;
        let id = format!(
            "workspace-{}",
            random
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        if existing.iter().all(|workspace| workspace.id != id) {
            return Ok(id);
        }
    }

    Err("Could not allocate a unique workspace ID.".to_string())
}

pub(crate) fn new_workspace_id(existing: &[Workspace]) -> Result<(String, u64), String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "System time is unavailable.".to_string())?
        .as_millis();
    let added_at = u64::try_from(timestamp).unwrap_or(u64::MAX);
    let id = allocate_unique_workspace_id_with(existing, |random| {
        getrandom::fill(random)
            .map_err(|error| format!("Could not generate a workspace ID: {error}"))
    })?;
    Ok((id, added_at))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{WorkspaceAccessMode, WorkspaceChangePolicy, CommandPolicy};
    fn workspace_with_id(id: String) -> Workspace {
        Workspace { id, name: "Test".to_string(), path: ".".to_string(), added_at: 0,
            access_mode: WorkspaceAccessMode::ReadWrite,
            change_policy: WorkspaceChangePolicy::Automatic,
            command_policy: CommandPolicy::Automatic }
    }
    #[test]
    fn workspace_id_allocator_retries_a_collision() {
        let colliding_id = format!("workspace-{}", "00".repeat(16));
        let existing = vec![workspace_with_id(colliding_id.clone())];
        let mut attempt = 0usize;
        let id = allocate_unique_workspace_id_with(&existing, |random| {
            *random = if attempt == 0 { [0u8; 16] } else { [1u8; 16] };
            attempt += 1;
            Ok(())
        }).unwrap();
        assert_eq!(attempt, 2);
        assert_ne!(id, colliding_id);
        assert_eq!(id, format!("workspace-{}", "01".repeat(16)));
    }
}
