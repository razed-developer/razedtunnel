//! Disposable RazedBrowser profile ownership.
//!
//! This is a filesystem lifecycle primitive, NOT an OS sandbox. No browser
//! launch or session activation is permitted merely because a profile exists.
use std::{fs, path::{Path, PathBuf}};

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
