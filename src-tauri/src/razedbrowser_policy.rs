//! RazedBrowser session permission model.
//!
//! This module is intentionally policy-only. URL validation does NOT establish
//! DNS-rebinding protection, OS process isolation, or network egress control.
//! Browser launch must remain disabled until those enforcement layers exist.
use serde::{Deserialize, Serialize};
use url::{Host, Url};

pub(crate) const MAX_SESSION_MILLIS: u64 = 30 * 60 * 1_000;
pub(crate) const MAX_ORIGINS: usize = 16;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RazedBrowserSessionPolicy {
    pub(crate) session_id: String,
    pub(crate) approved_origins: Vec<String>,
    pub(crate) issued_at: u64,
    pub(crate) expires_at: u64,
    pub(crate) revoked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionAction {
    Observe,
    Navigate,
    Click,
    Type,
    Download,
    Upload,
    ReadLocalFile,
    Clipboard,
    Submit,
    Purchase,
    Login,
}

/// Validate a syntactic public HTTPS origin. DNS resolution and every redirect
/// must also be checked by the future isolated network enforcement layer.
pub(crate) fn public_https_origin(input: &str) -> Result<String, String> {
    let url = Url::parse(input).map_err(|_| "Invalid website URL.".to_string())?;
    if url.scheme() != "https" {
        return Err("RazedBrowser requires HTTPS.".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("Embedded website credentials are not allowed.".to_string());
    }
    let host = url.host().ok_or_else(|| "Website host is required.".to_string())?;
    match host {
        Host::Domain(domain) => {
            let name = domain.trim_end_matches('.').to_ascii_lowercase();
            if name.is_empty() || !name.contains('.')
                || name == "localhost" || name.ends_with(".localhost")
                || name.ends_with(".local") || name.ends_with(".internal")
                || name.ends_with(".test") || name.ends_with(".invalid")
                || name.ends_with(".example")
            {
                return Err("Local, reserved, or unqualified hosts are blocked.".to_string());
            }
        }
        Host::Ipv4(_) | Host::Ipv6(_) => {
            // DNS names only in v1. This avoids treating IP literals as proof
            // of public reachability and simplifies the network isolation gate.
            return Err("IP-literal destinations are blocked.".to_string());
        }
    }
    Ok(url.origin().ascii_serialization())
}

impl RazedBrowserSessionPolicy {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.session_id.trim().is_empty() {
            return Err("Session ID is required.".to_string());
        }
        if self.expires_at <= self.issued_at
            || self.expires_at - self.issued_at > MAX_SESSION_MILLIS
        {
            return Err("Session lifetime must be positive and at most 30 minutes.".to_string());
        }
        if self.approved_origins.is_empty() || self.approved_origins.len() > MAX_ORIGINS {
            return Err("A session requires 1 to 16 approved origins.".to_string());
        }
        for origin in &self.approved_origins {
            if public_https_origin(origin)? != *origin {
                return Err("Approved origins must be canonical HTTPS origins.".to_string());
            }
        }
        let mut unique = self.approved_origins.clone();
        unique.sort();
        unique.dedup();
        if unique.len() != self.approved_origins.len() {
            return Err("Approved origins must be unique.".to_string());
        }
        Ok(())
    }

    pub(crate) fn authorize(
        &self,
        now_millis: u64,
        action: SessionAction,
        url: &str,
    ) -> Result<(), String> {
        self.validate()?;
        if self.revoked || now_millis < self.issued_at || now_millis >= self.expires_at {
            return Err("RazedBrowser session is inactive, expired, or revoked.".to_string());
        }
        if !matches!(action, SessionAction::Observe | SessionAction::Navigate) {
            return Err("This action requires a separate, explicit authorization.".to_string());
        }
        let origin = public_https_origin(url)?;
        if !self.approved_origins.iter().any(|allowed| allowed == &origin) {
            return Err("Website origin is outside the approved session scope.".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> RazedBrowserSessionPolicy {
        RazedBrowserSessionPolicy {
            session_id: "test-session".into(),
            approved_origins: vec!["https://example.org".into()],
            issued_at: 1_000,
            expires_at: 61_000,
            revoked: false,
        }
    }

    #[test]
    fn allows_only_approved_https_origin() {
        let p = policy();
        assert!(p.authorize(2_000, SessionAction::Navigate, "https://example.org/page").is_ok());
        assert!(p.authorize(2_000, SessionAction::Observe, "https://example.org:443/").is_ok());
        assert!(p.authorize(2_000, SessionAction::Navigate, "https://other.org/").is_err());
        assert!(p.authorize(2_000, SessionAction::Navigate, "http://example.org/").is_err());
        assert!(p.authorize(2_000, SessionAction::Navigate, "https://example.org.evil.test/").is_err());
    }

    #[test]
    fn blocks_private_and_local_targets() {
        for url in [
            "https://localhost/", "https://localhost.local/", "https://printer.local/",
            "https://127.0.0.1/", "https://10.0.0.1/", "https://192.168.1.1/",
            "https://169.254.169.254/", "https://[::1]/", "file:///etc/passwd",
            "https://user:pass@example.org/",
        ] {
            assert!(public_https_origin(url).is_err(), "unexpectedly allowed {url}");
        }
    }

    #[test]
    fn expires_and_revokes() {
        let mut p = policy();
        assert!(p.authorize(61_000, SessionAction::Observe, "https://example.org/").is_err());
        p.revoked = true;
        assert!(p.authorize(2_000, SessionAction::Observe, "https://example.org/").is_err());
    }

    #[test]
    fn blocks_mutating_actions() {
        let p = policy();
        for action in [
            SessionAction::Click, SessionAction::Type, SessionAction::Download,
            SessionAction::Upload, SessionAction::ReadLocalFile, SessionAction::Clipboard,
            SessionAction::Submit, SessionAction::Purchase, SessionAction::Login,
        ] {
            assert!(p.authorize(2_000, action, "https://example.org/").is_err());
        }
    }

    #[test]
    fn rejects_invalid_grants() {
        let mut p = policy();
        p.expires_at = p.issued_at + MAX_SESSION_MILLIS + 1;
        assert!(p.validate().is_err());
        p = policy();
        p.approved_origins.push("https://example.org".into());
        assert!(p.validate().is_err());
    }
}
