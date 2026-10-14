//! In-memory, single-use approval foundation. Not exposed to desktop actions yet.
//! A desktop click must never be treated as proof of which filesystem path it affects.
use std::{collections::HashMap, path::{Path, PathBuf}, sync::Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FileOperation { Create, Write, Move, Delete }

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Approval {
    pub(crate) workspace_id: String,
    pub(crate) path: PathBuf,
    pub(crate) operation: FileOperation,
}

#[derive(Default)]
pub(crate) struct ApprovalRegistry {
    pending: Mutex<HashMap<String, Approval>>,
}

fn canonical_existing(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize().map_err(|_| "Approval path must exist and be verifiable.".to_string())
}

impl ApprovalRegistry {
    /// Caller must use an unpredictable, collision-resistant token; this registry
    /// intentionally does not generate credentials.
    pub(crate) fn register(&self, token: String, approval: Approval, workspace_root: &Path) -> Result<(), String> {
        if token.len() < 32 || !token.bytes().all(|b| b.is_ascii_alphanumeric()) {
            return Err("Approval token must be an opaque 32+ character alphanumeric value.".into());
        }
        if approval.workspace_id.trim().is_empty() {
            return Err("Approval requires a workspace identity.".into());
        }
        let root = canonical_existing(workspace_root)?;
        let target = canonical_existing(&approval.path)?;
        if !target.starts_with(&root) || target == root {
            return Err("Approval target must be an existing path below the workspace root.".into());
        }
        let mut pending = self.pending.lock().map_err(|_| "Approval registry unavailable.")?;
        if pending.contains_key(&token) {
            return Err("Approval token already exists.".into());
        }
        pending.insert(token, Approval { path: target, ..approval });
        Ok(())
    }

    /// Consume before executing any operation. Failed matches also consume the token
    /// so it cannot be replayed or used to probe alternative targets.
    pub(crate) fn consume(&self, token: &str, expected: &Approval, workspace_root: &Path) -> Result<(), String> {
        let stored = self.pending.lock().map_err(|_| "Approval registry unavailable.")?
            .remove(token).ok_or_else(|| "Approval is missing or already consumed.".to_string())?;
        let root = canonical_existing(workspace_root)?;
        let target = canonical_existing(&expected.path)?;
        if target == root || !target.starts_with(&root) ||
            stored.workspace_id != expected.workspace_id ||
            stored.operation != expected.operation || stored.path != target {
            return Err("Approval does not match the requested workspace, path, or operation.".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn approvals_are_single_use_and_bound_to_operation() {
        let root = std::env::temp_dir().join(format!("razedtunnel-approval-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let target = root.join("target.txt");
        std::fs::write(&target, b"untouched").unwrap();
        let registry = ApprovalRegistry::default();
        let approval = Approval { workspace_id: "workspace-a".into(), path: target.clone(), operation: FileOperation::Delete };
        let token = "a".repeat(40);
        registry.register(token.clone(), approval.clone(), &root).unwrap();
        let wrong = Approval { operation: FileOperation::Write, ..approval.clone() };
        assert!(registry.consume(&token, &wrong, &root).is_err());
        assert!(registry.consume(&token, &approval, &root).is_err());
        let token2 = "b".repeat(40);
        registry.register(token2.clone(), approval.clone(), &root).unwrap();
        registry.consume(&token2, &approval, &root).unwrap();
        assert!(registry.consume(&token2, &approval, &root).is_err());
        assert_eq!(std::fs::read(&target).unwrap(), b"untouched");
        std::fs::remove_file(target).unwrap();
        std::fs::remove_dir(root).unwrap();
    }

    #[test]
    fn approval_cannot_target_workspace_root_or_outside() {
        let base = std::env::temp_dir();
        let root = base.join(format!("razedtunnel-scope-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let registry = ApprovalRegistry::default();
        for path in [&root, &base] {
            let approval = Approval { workspace_id: "w".into(), path: path.to_path_buf(), operation: FileOperation::Delete };
            assert!(registry.register("c".repeat(40), approval, &root).is_err());
        }
        std::fs::remove_dir(root).unwrap();
    }
}
