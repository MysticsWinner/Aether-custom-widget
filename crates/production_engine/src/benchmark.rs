use crate::security_audit::SecurityAuditor;
use crate::stress_test::StressTestingHarness;
use std::time::Instant;
use tracing::info;

/// Master Production Release Candidate Verification Suite.
pub struct MasterReleaseSuite;

impl MasterReleaseSuite {
    pub fn run_release_audit() -> bool {
        let start = Instant::now();
        info!("===========================================================");
        info!(" MASTER PRODUCTION RELEASE CANDIDATE VERIFICATION SUITE");
        info!("===========================================================");

        let audit_pass = SecurityAuditor::run_security_audit();
        let stress_pass = StressTestingHarness::run_stress_test(100, 1000);

        let elapsed = start.elapsed();
        let passed = audit_pass && stress_pass;

        info!(
            "Production Release Candidate Verification Complete in {:?}: Passed = {}",
            elapsed, passed
        );

        passed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_release_suite_execution() {
        assert!(MasterReleaseSuite::run_release_audit());
    }
}
