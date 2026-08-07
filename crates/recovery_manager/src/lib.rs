pub mod manager;
pub mod policy;
pub mod quarantine;
pub mod rollback;
pub mod safe_mode;
pub mod types;

pub use manager::{CrashRecoveryAction, RecoveryManager};
pub use policy::CrashPolicy;
pub use quarantine::QuarantineStore;
pub use rollback::RollbackCoordinator;
pub use safe_mode::SafeModeGuard;
pub use types::{CrashRecord, LaunchMode, QuarantineRecord};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_recovery_manager_restarts_on_first_crash() {
        let dir = tempdir().unwrap();
        let mut manager = RecoveryManager::new(dir.path(), CrashPolicy::default());

        let action = manager.handle_widget_crash("test_widget", Some(1), 1000).unwrap();
        assert_eq!(
            action,
            CrashRecoveryAction::Restart {
                backoff_ms: 1000,
                attempt: 1
            }
        );
        assert!(!manager.is_quarantined("test_widget"));
    }

    #[test]
    fn test_recovery_manager_quarantines_after_crash_loop() {
        let dir = tempdir().unwrap();
        let mut policy = CrashPolicy::default();
        policy.max_crashes = 3;
        let mut manager = RecoveryManager::new(dir.path(), policy);

        manager.handle_widget_crash("broken_widget", Some(1), 1000).unwrap();
        manager.handle_widget_crash("broken_widget", Some(1), 2000).unwrap();
        let action = manager.handle_widget_crash("broken_widget", Some(1), 3000).unwrap();

        match action {
            CrashRecoveryAction::Quarantine { reason } => {
                assert!(reason.contains("Crash loop detected"));
            }
            _ => panic!("Expected Quarantine action"),
        }

        assert!(manager.is_quarantined("broken_widget"));
    }

    #[test]
    fn test_quarantine_store_persistence() {
        let dir = tempdir().unwrap();
        let q_path = dir.path().join("quarantine.json");
        let mut store = QuarantineStore::new(&q_path);

        store
            .quarantine(QuarantineRecord {
                widget_id: "bad_widget".to_string(),
                quarantined_at_ms: 5000,
                reason: "Test failure".to_string(),
            })
            .unwrap();

        assert!(store.is_quarantined("bad_widget"));

        // Reload store from disk
        let store2 = QuarantineStore::new(&q_path);
        assert!(store2.is_quarantined("bad_widget"));
        assert_eq!(store2.list().len(), 1);
    }

    #[test]
    fn test_safe_mode_guard_sentinel_lifecycle() {
        let dir = tempdir().unwrap();
        let guard = SafeModeGuard::new(dir.path(), 3);

        // Initial evaluation -> Normal mode & sentinel created
        let mode = guard.evaluate_and_arm().unwrap();
        assert_eq!(mode, LaunchMode::Normal);

        // Clean disarm
        guard.disarm_sentinel().unwrap();

        // Evaluate again -> Normal mode
        let mode2 = guard.evaluate_and_arm().unwrap();
        assert_eq!(mode2, LaunchMode::Normal);
    }

    #[test]
    fn test_safe_mode_triggered_after_n_abnormal_exits() {
        let dir = tempdir().unwrap();
        let guard = SafeModeGuard::new(dir.path(), 2);

        // Run 1: Armed
        let _ = guard.evaluate_and_arm().unwrap();
        // Simulate crash by NOT calling disarm_sentinel()

        // Run 2: Armed (sees 1st sentinel)
        let _ = guard.evaluate_and_arm().unwrap();
        // Simulate crash again

        // Run 3: Evaluates -> Safe Mode triggered (2 crashes)
        let mode = guard.evaluate_and_arm().unwrap();
        assert!(mode.is_safe_mode());
    }

    #[test]
    fn test_quarantine_release() {
        let dir = tempdir().unwrap();
        let mut policy = CrashPolicy::default();
        policy.max_crashes = 1;
        let mut manager = RecoveryManager::new(dir.path(), policy);

        manager.handle_widget_crash("flaky", Some(1), 1000).unwrap();
        assert!(manager.is_quarantined("flaky"));

        let released = manager.release_quarantine("flaky").unwrap();
        assert!(released);
        assert!(!manager.is_quarantined("flaky"));
    }
}
