//! In-memory, single-use approval foundation. Not exposed to desktop actions yet.
//! A desktop click must never be treated as proof of which filesystem path it affects.
use std::{collections::HashMap, path::{Path, PathBuf}, sync::Mutex, time::{Duration, Instant}};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FileOperation { Create, Write, Move, Delete }

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Approval {
    pub(crate) workspace_id: String,
    pub(crate) path: PathBuf,
    pub(crate) operation: FileOperation,
}

const APPROVAL_TTL: Duration = Duration::from_secs(120);

struct PendingApproval { approval: Approval, expires_at: Instant }

#[derive(Default)]
pub(crate) struct ApprovalRegistry {
    pending: Mutex<HashMap<String, PendingApproval>>,
}

fn canonical_existing(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize().map_err(|_| "Approval path must exist and be verifiable.".to_string())
}

// This capability cannot be constructed by MCP/HTTP request parsing.
// A future local UI confirmation handler must be the only production caller
// of the private constructor in this module.
struct LocalConfirmation { _private: () }

impl LocalConfirmation {
    fn from_trusted_local_ui() -> Self { Self { _private: () } }
}

impl ApprovalRegistry {
    /// Only call from a trusted local confirmation handler, never an AI-facing endpoint.
    fn issue_local(&self, _confirmation: LocalConfirmation, approval: Approval, workspace_root: &Path) -> Result<String, String> {
        let mut bytes = [0u8; 32];
        getrandom::fill(&mut bytes).map_err(|_| "Secure approval token generation failed.")?;
        let token: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        self.register(token.clone(), approval, workspace_root)?;
        Ok(token)
    }

    /// Caller must use an unpredictable, collision-resistant token; this registry
    /// intentionally does not generate credentials.
    fn register(&self, token: String, approval: Approval, workspace_root: &Path) -> Result<(), String> {
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
        pending.insert(token, PendingApproval { approval: Approval { path: target, ..approval }, expires_at: Instant::now() + APPROVAL_TTL });
        Ok(())
    }

    /// Consume before executing any operation. Failed matches also consume the token
    /// so it cannot be replayed or used to probe alternative targets.
    pub(crate) fn consume(&self, token: &str, expected: &Approval, workspace_root: &Path) -> Result<(), String> {
        let stored = self.pending.lock().map_err(|_| "Approval registry unavailable.")?
            .remove(token).ok_or_else(|| "Approval is missing or already consumed.".to_string())?;
        if Instant::now() >= stored.expires_at {
            return Err("Approval has expired.".into());
        }
        let stored = stored.approval;
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
    fn approval_requires_explicit_local_capability() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target.txt");
        std::fs::write(&target, b"safe").unwrap();
        let approval = Approval { workspace_id: "w".into(), path: target, operation: FileOperation::Write };
        let registry = ApprovalRegistry::default();
        let token = registry.issue_local(LocalConfirmation::from_trusted_local_ui(), approval.clone(), temp.path()).unwrap();
        assert!(registry.consume(&token, &approval, temp.path()).is_ok());
    }

    #[test]
    fn local_tokens_are_random_and_single_use() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("file.txt");
        std::fs::write(&target, b"safe").unwrap();
        let approval = Approval { workspace_id: "w".into(), path: target, operation: FileOperation::Write };
        let registry = ApprovalRegistry::default();
        let first = registry.issue_local(LocalConfirmation::from_trusted_local_ui(), approval.clone(), temp.path()).unwrap();
        let second = registry.issue_local(LocalConfirmation::from_trusted_local_ui(), approval.clone(), temp.path()).unwrap();
        assert_eq!(first.len(), 64);
        assert_ne!(first, second);
        registry.consume(&first, &approval, temp.path()).unwrap();
        assert!(registry.consume(&first, &approval, temp.path()).is_err());
        registry.consume(&second, &approval, temp.path()).unwrap();
    }

    #[test]
    fn expired_tokens_are_rejected_and_consumed() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("file.txt");
        std::fs::write(&target, b"safe").unwrap();
        let approval = Approval { workspace_id: "w".into(), path: target, operation: FileOperation::Delete };
        let registry = ApprovalRegistry::default();
        let token = registry.issue_local(LocalConfirmation::from_trusted_local_ui(), approval.clone(), temp.path()).unwrap();
        registry.pending.lock().unwrap().get_mut(&token).unwrap().expires_at = Instant::now() - Duration::from_secs(1);
        assert!(registry.consume(&token, &approval, temp.path()).unwrap_err().contains("expired"));
        assert!(registry.consume(&token, &approval, temp.path()).is_err());
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
