//! RazedBrowser session lifecycle state machine.
//!
//! No browser is launched here. Callers must provide an isolated runtime and
//! confirm its termination and profile cleanup before marking a session closed.
use std::collections::HashMap;

use super::razedbrowser_policy::{RazedBrowserSessionPolicy, SessionAction};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionState {
    AwaitingIsolation,
    Active,
    Terminating,
    Closed,
}

#[derive(Clone, Debug)]
pub(crate) struct ManagedSession {
    pub(crate) policy: RazedBrowserSessionPolicy,
    pub(crate) state: SessionState,
}

#[derive(Default)]
pub(crate) struct SessionRegistry {
    sessions: HashMap<String, ManagedSession>,
}

impl SessionRegistry {
    /// Register an approved policy without launching or authorizing any browser.
    pub(crate) fn register(&mut self, policy: RazedBrowserSessionPolicy) -> Result<(), String> {
        policy.validate()?;
        if self.sessions.contains_key(&policy.session_id) {
            return Err("Session ID already exists and cannot be reused.".into());
        }
        self.sessions.insert(
            policy.session_id.clone(),
            ManagedSession { policy, state: SessionState::AwaitingIsolation },
        );
        Ok(())
    }

    /// Only call after the isolated process, egress controls, and disposable
    /// profile have been independently established and verified.
    pub(crate) fn activate_after_isolation_verified(&mut self, id: &str, now: u64) -> Result<(), String> {
        let session = self.get_mut(id)?;
        if session.state != SessionState::AwaitingIsolation || session.policy.revoked
            || now < session.policy.issued_at || now >= session.policy.expires_at
        {
            return Err("Session is not eligible for activation.".into());
        }
        session.state = SessionState::Active;
        Ok(())
    }

    pub(crate) fn authorize(&self, id: &str, now: u64, action: SessionAction, url: &str) -> Result<(), String> {
        let session = self.sessions.get(id).ok_or("Unknown RazedBrowser session.")?;
        if session.state != SessionState::Active {
            return Err("Session is not active.".into());
        }
        session.policy.authorize(now, action, url)
    }

    /// Revocation is immediate and irreversible; resource teardown is separate.
    pub(crate) fn revoke(&mut self, id: &str) -> Result<(), String> {
        let session = self.get_mut(id)?;
        session.policy.revoked = true;
        if session.state != SessionState::Closed {
            session.state = SessionState::Terminating;
        }
        Ok(())
    }

    /// Expiry transitions all affected sessions to termination without extending
    /// grants. The runtime owner must still kill processes and remove profiles.
    pub(crate) fn expire(&mut self, now: u64) -> Vec<String> {
        let mut expired = Vec::new();
        for (id, session) in &mut self.sessions {
            if session.state != SessionState::Closed
                && session.state != SessionState::Terminating
                && now >= session.policy.expires_at
            {
                session.policy.revoked = true;
                session.state = SessionState::Terminating;
                expired.push(id.clone());
            }
        }
        expired.sort();
        expired
    }

    /// Must only be called after process termination and profile deletion have
    /// both succeeded. This method does not perform either operation.
    pub(crate) fn confirm_cleanup(&mut self, id: &str) -> Result<(), String> {
        let session = self.get_mut(id)?;
        if session.state != SessionState::Terminating {
            return Err("Only terminating sessions can be closed.".into());
        }
        session.state = SessionState::Closed;
        Ok(())
    }

    pub(crate) fn state(&self, id: &str) -> Option<SessionState> {
        self.sessions.get(id).map(|session| session.state)
    }

    fn get_mut(&mut self, id: &str) -> Result<&mut ManagedSession, String> {
        self.sessions.get_mut(id).ok_or_else(|| "Unknown RazedBrowser session.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(id: &str) -> RazedBrowserSessionPolicy {
        RazedBrowserSessionPolicy {
            session_id: id.into(),
            approved_origins: vec!["https://example.org".into()],
            issued_at: 100,
            expires_at: 500,
            revoked: false,
        }
    }

    #[test]
    fn session_is_default_denied_until_isolation_verified() {
        let mut registry = SessionRegistry::default();
        registry.register(policy("one")).unwrap();
        assert!(registry.authorize("one", 200, SessionAction::Navigate, "https://example.org/").is_err());
        registry.activate_after_isolation_verified("one", 200).unwrap();
        assert!(registry.authorize("one", 200, SessionAction::Navigate, "https://example.org/").is_ok());
        assert!(registry.activate_after_isolation_verified("one", 201).is_err());
    }

    #[test]
    fn revocation_blocks_actions_and_requires_cleanup() {
        let mut registry = SessionRegistry::default();
        registry.register(policy("one")).unwrap();
        registry.activate_after_isolation_verified("one", 200).unwrap();
        registry.revoke("one").unwrap();
        assert_eq!(registry.state("one"), Some(SessionState::Terminating));
        assert!(registry.authorize("one", 200, SessionAction::Observe, "https://example.org/").is_err());
        registry.confirm_cleanup("one").unwrap();
        assert_eq!(registry.state("one"), Some(SessionState::Closed));
        assert!(registry.activate_after_isolation_verified("one", 200).is_err());
        assert!(registry.register(policy("one")).is_err());
    }

    #[test]
    fn expiry_covers_unactivated_and_active_sessions() {
        let mut registry = SessionRegistry::default();
        registry.register(policy("pending")).unwrap();
        registry.register(policy("active")).unwrap();
        registry.activate_after_isolation_verified("active", 200).unwrap();
        assert_eq!(registry.expire(500), vec!["active", "pending"]);
        assert!(registry.expire(600).is_empty());
        assert!(registry.authorize("active", 501, SessionAction::Observe, "https://example.org/").is_err());
    }

    #[test]
    fn cannot_claim_cleanup_before_termination() {
        let mut registry = SessionRegistry::default();
        registry.register(policy("one")).unwrap();
        assert!(registry.confirm_cleanup("one").is_err());
        assert!(registry.activate_after_isolation_verified("one", 500).is_err());
    }
}
