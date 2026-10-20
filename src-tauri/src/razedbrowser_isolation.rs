//! Explicit isolation requirements for the future Windows RazedBrowser runtime.
//! This is a fail-closed readiness model, not a sandbox implementation.
//! Never derive these assertions from user input or from Job Object membership.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct IsolationEvidence {
    pub(crate) restricted_identity: bool,
    pub(crate) filesystem_boundary: bool,
    pub(crate) network_egress_boundary: bool,
    pub(crate) process_tree_contained: bool,
    pub(crate) disposable_profile: bool,
}

impl IsolationEvidence {
    /// This only evaluates evidence; it does not establish any boundary.
    /// Production activation remains disabled independently in SessionRegistry.
    pub(crate) fn missing_requirements(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if !self.restricted_identity { missing.push("restricted execution identity"); }
        if !self.filesystem_boundary { missing.push("filesystem boundary"); }
        if !self.network_egress_boundary { missing.push("network egress boundary"); }
        if !self.process_tree_contained { missing.push("process-tree containment"); }
        if !self.disposable_profile { missing.push("disposable profile"); }
        missing
    }

    pub(crate) fn require_all(&self) -> Result<(), String> {
        let missing = self.missing_requirements();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!("RazedBrowser isolation incomplete: {}", missing.join(", ")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_isolation_is_denied() {
        let evidence = IsolationEvidence::default();
        assert_eq!(evidence.missing_requirements().len(), 5);
        assert!(evidence.require_all().is_err());
    }

    #[test]
    fn job_and_profile_are_not_sufficient() {
        let evidence = IsolationEvidence {
            process_tree_contained: true,
            disposable_profile: true,
            ..Default::default()
        };
        assert_eq!(evidence.missing_requirements(), vec![
            "restricted execution identity",
            "filesystem boundary",
            "network egress boundary",
        ]);
        assert!(evidence.require_all().is_err());
    }

    #[test]
    fn each_boundary_is_independently_required() {
        let complete = IsolationEvidence {
            restricted_identity: true,
            filesystem_boundary: true,
            network_egress_boundary: true,
            process_tree_contained: true,
            disposable_profile: true,
        };
        assert!(complete.require_all().is_ok());
        for incomplete in [
            IsolationEvidence { restricted_identity: false, ..complete },
            IsolationEvidence { filesystem_boundary: false, ..complete },
            IsolationEvidence { network_egress_boundary: false, ..complete },
            IsolationEvidence { process_tree_contained: false, ..complete },
            IsolationEvidence { disposable_profile: false, ..complete },
        ] {
            assert!(incomplete.require_all().is_err());
        }
    }
}
