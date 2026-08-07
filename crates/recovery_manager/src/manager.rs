use crate::policy::CrashPolicy;
use crate::quarantine::QuarantineStore;
use crate::rollback::RollbackCoordinator;
use crate::safe_mode::SafeModeGuard;
use crate::types::{CrashRecord, LaunchMode, QuarantineRecord};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use tracing::{error, info, warn};

/// Result of evaluating a crash event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrashRecoveryAction {
    /// Restart with exponential backoff delay in ms.
    Restart { backoff_ms: u64, attempt: u32 },
    /// Widget crash-looped and is now quarantined.
    Quarantine { reason: String },
}

/// Central Recovery Manager orchestrating widget crash tracking, quarantining, and safe mode checks.
pub struct RecoveryManager {
    policy: CrashPolicy,
    quarantine_store: QuarantineStore,
    safe_mode_guard: SafeModeGuard,
    rollback_coordinator: Option<RollbackCoordinator>,
    crash_history: HashMap<String, Vec<CrashRecord>>,
}

impl RecoveryManager {
    pub fn new<P: AsRef<Path>>(base_dir: P, policy: CrashPolicy) -> Self {
        let base_dir = base_dir.as_ref();
        let quarantine_store = QuarantineStore::new(base_dir.join("quarantine.json"));
        let safe_mode_guard = SafeModeGuard::new(base_dir, 3);

        Self {
            policy,
            quarantine_store,
            safe_mode_guard,
            rollback_coordinator: None,
            crash_history: HashMap::new(),
        }
    }

    pub fn set_rollback_coordinator(&mut self, coordinator: RollbackCoordinator) {
        self.rollback_coordinator = Some(coordinator);
    }

    pub fn safe_mode_guard(&self) -> &SafeModeGuard {
        &self.safe_mode_guard
    }

    pub fn quarantine_store(&self) -> &QuarantineStore {
        &self.quarantine_store
    }

    /// Evaluates current launch mode (Normal vs Safe Mode).
    pub fn evaluate_launch_mode(&self) -> Result<LaunchMode> {
        self.safe_mode_guard.evaluate_and_arm()
    }

    /// Primary entrypoint when a widget plugin crashes or panics.
    pub fn handle_widget_crash(
        &mut self,
        widget_id: &str,
        exit_code: Option<i32>,
        now_ms: u64,
    ) -> Result<CrashRecoveryAction> {
        let history = self.crash_history.entry(widget_id.to_string()).or_default();

        // Prune records older than policy.window_secs
        let window_ms = self.policy.window_secs * 1000;
        history.retain(|r| now_ms.saturating_sub(r.timestamp_ms) <= window_ms);

        let crash_count = (history.len() as u32) + 1;
        let record = CrashRecord {
            widget_id: widget_id.to_string(),
            timestamp_ms: now_ms,
            exit_code,
            crash_count,
        };
        history.push(record.clone());

        warn!(
            widget_id = %widget_id,
            crash_count,
            max_allowed = self.policy.max_crashes,
            "Widget crash recorded"
        );

        if crash_count >= self.policy.max_crashes {
            let reason = format!(
                "Crash loop detected: {} crashes in {}s window",
                crash_count, self.policy.window_secs
            );
            error!(widget_id = %widget_id, %reason, "Quarantining widget to keep engine healthy.");
            
            let q_record = QuarantineRecord {
                widget_id: widget_id.to_string(),
                quarantined_at_ms: now_ms,
                reason: reason.clone(),
            };
            self.quarantine_store.quarantine(q_record)?;

            Ok(CrashRecoveryAction::Quarantine { reason })
        } else {
            // Exponential backoff: 2^(attempt - 1) * base_ms
            let backoff_factor = 1u64 << (crash_count.saturating_sub(1));
            let backoff_ms = self.policy.backoff_base_ms.saturating_mul(backoff_factor);

            info!(
                widget_id = %widget_id,
                attempt = crash_count,
                backoff_ms,
                "Scheduling widget restart with exponential backoff"
            );

            Ok(CrashRecoveryAction::Restart {
                backoff_ms,
                attempt: crash_count,
            })
        }
    }

    /// Checks if a widget is currently blocked due to quarantine.
    pub fn is_quarantined(&self, widget_id: &str) -> bool {
        self.quarantine_store.is_quarantined(widget_id)
    }

    /// Manual un-quarantine of a widget.
    pub fn release_quarantine(&mut self, widget_id: &str) -> Result<bool> {
        self.crash_history.remove(widget_id);
        self.quarantine_store.remove(widget_id)
    }

    /// Retrieves crash history for a widget.
    pub fn get_crash_history(&self, widget_id: &str) -> Vec<CrashRecord> {
        self.crash_history.get(widget_id).cloned().unwrap_or_default()
    }
}
