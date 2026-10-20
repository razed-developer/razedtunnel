//! Disposable RazedBrowser profile ownership.
//!
//! This is a filesystem lifecycle primitive, NOT an OS sandbox. No browser
//! launch or session activation is permitted merely because a profile exists.
use std::{fs, path::{Path, PathBuf}};

/// Read-only recovery inventory. A listed profile is NOT evidence that its
/// owning process has exited, and must not be automatically deleted.
pub(crate) fn discover_leftover_profiles() -> Result<Vec<PathBuf>, String> {
    discover_profiles_under(&std::env::temp_dir().join("razedtunnel-razedbrowser"))
}

fn discover_profiles_under(root: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("Cannot inspect RazedBrowser profile root: {e}")),
    };
    let mut profiles = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Cannot inspect profile entry: {e}"))?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(suffix) = name.strip_prefix("profile-") else { continue };
        if suffix.len() != 48 || !suffix.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        // Do not traverse or accept reparse points / symlinks as profiles.
        let kind = entry.file_type().map_err(|e| format!("Cannot inspect profile type: {e}"))?;
        if kind.is_dir() && !kind.is_symlink() {
            profiles.push(entry.path());
        }
    }
    profiles.sort();
    Ok(profiles)
}

/// Recovery intentionally cannot infer process ownership from a directory name.
/// No discovered profile is eligible for deletion without separate, durable
/// proof of process termination and ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryDisposition {
    RequiresManualVerification,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecoveryCandidate {
    pub(crate) path: PathBuf,
    pub(crate) disposition: RecoveryDisposition,
}

/// Inventory only: no filesystem mutation and no attempt to attach to a job.
pub(crate) fn inventory_recovery_candidates() -> Result<Vec<RecoveryCandidate>, String> {
    Ok(discover_leftover_profiles()?
        .into_iter()
        .map(|path| RecoveryCandidate {
            path,
            disposition: RecoveryDisposition::RequiresManualVerification,
        })
        .collect())
}

pub(crate) struct DisposableProfile {
    path: PathBuf,
    cleaned: bool,
    cleanup_on_drop: bool,
}

impl DisposableProfile {
    pub(crate) fn create() -> Result<Self, String> {
        let base = std::env::temp_dir().join("razedtunnel-razedbrowser");
        fs::create_dir_all(&base).map_err(|e| format!("Cannot create profile root: {e}"))?;
        for _ in 0..8 {
            let mut random = [0u8; 24];
            getrandom::fill(&mut random).map_err(|e| format!("Cannot generate profile identifier: {e}"))?;
            let suffix: String = random.iter().map(|b| format!("{b:02x}")).collect();
            let path = base.join(format!("profile-{suffix}"));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path, cleaned: false, cleanup_on_drop: true }),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(format!("Cannot create disposable profile: {e}")),
            }
        }
        Err("Could not allocate a unique disposable profile.".into())
    }

    /// Preserve profile data if process termination could not be verified.
    /// A later recovery routine must investigate and remove the directory.
    pub(crate) fn preserve_on_drop(&mut self) {
        self.cleanup_on_drop = false;
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// Only call after all browser processes using the profile are terminated.
    /// A failure leaves the profile owned and cleanup retryable.
    pub(crate) fn cleanup(&mut self) -> Result<(), String> {
        if self.cleaned {
            return Ok(());
        }
        match fs::remove_dir_all(&self.path) {
            Ok(()) => {
                self.cleaned = true;
                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Missing is not proof of successful cleanup: an unexpected
                // deletion or path replacement needs explicit investigation.
                Err("Disposable profile disappeared before cleanup.".into())
            }
            Err(e) => Err(format!("Disposable profile cleanup failed: {e}")),
        }
    }
}

impl Drop for DisposableProfile {
    fn drop(&mut self) {
        if !self.cleaned && self.cleanup_on_drop {
            // Best effort only: process shutdown must precede cleanup, and the
            // owning runtime must explicitly check cleanup() before closure.
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_inventory_never_marks_profiles_safe_to_delete() {
        let mut profile = DisposableProfile::create().unwrap();
        let path = profile.path().to_owned();
        fs::write(path.join("session-data"), b"retain").unwrap();
        let candidates = inventory_recovery_candidates().unwrap();
        let candidate = candidates.iter().find(|item| item.path == path).unwrap();
        assert_eq!(candidate.disposition, RecoveryDisposition::RequiresManualVerification);
        assert!(path.join("session-data").exists());
        profile.cleanup().unwrap();
    }

    #[test]
    fn discovery_lists_profiles_without_deleting_them() {
        let mut profile = DisposableProfile::create().unwrap();
        let path = profile.path().to_owned();
        fs::write(path.join("session-data"), b"retain").unwrap();
        let discovered = discover_leftover_profiles().unwrap();
        assert!(discovered.contains(&path));
        assert!(path.join("session-data").exists());
        profile.cleanup().unwrap();
    }

    #[test]
    fn discovery_ignores_non_profile_entries() {
        let profile = DisposableProfile::create().unwrap();
        let root = profile.path().parent().unwrap();
        let listed = discover_profiles_under(root).unwrap();
        assert!(listed.contains(&profile.path().to_owned()));
        assert!(listed.iter().all(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            name.strip_prefix("profile-").is_some_and(|suffix|
                suffix.len() == 48 && suffix.bytes().all(|b| b.is_ascii_hexdigit()))
        }));
    }

    #[test]
    fn discovery_of_missing_root_is_empty() {
        let profile = DisposableProfile::create().unwrap();
        let nonexistent = profile.path().join("not-created");
        assert!(discover_profiles_under(&nonexistent).unwrap().is_empty());
    }

    #[test]
    fn profiles_are_distinct_and_cleanup_removes_data() {
        let mut first = DisposableProfile::create().unwrap();
        let mut second = DisposableProfile::create().unwrap();
        assert_ne!(first.path(), second.path());
        let first_path = first.path().to_owned();
        let second_path = second.path().to_owned();
        fs::write(first.path().join("cookies.sqlite"), b"session-only").unwrap();
        first.cleanup().unwrap();
        assert!(!first_path.exists());
        assert!(second_path.exists());
        first.cleanup().unwrap();
        second.cleanup().unwrap();
        assert!(!second_path.exists());
    }

    #[test]
    fn drop_removes_unfinished_profile_best_effort() {
        let path = {
            let profile = DisposableProfile::create().unwrap();
            let path = profile.path().to_owned();
            fs::write(path.join("cache"), b"data").unwrap();
            path
        };
        assert!(!path.exists());
    }

    #[test]
    fn unverified_shutdown_can_preserve_profile() {
        let path = {
            let mut profile = DisposableProfile::create().unwrap();
            let path = profile.path().to_owned();
            profile.preserve_on_drop();
            path
        };
        assert!(path.exists());
        fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn missing_profile_is_not_reported_as_cleaned() {
        let mut profile = DisposableProfile::create().unwrap();
        fs::remove_dir(profile.path()).unwrap();
        assert!(profile.cleanup().is_err());
    }
}
