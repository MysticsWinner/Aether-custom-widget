use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::{info, warn};

/// Failure scenarios supported by the Chaos Injection Harness.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChaosScenario {
    WidgetCrash { widget_id: String },
    OomAllocation { widget_id: String, megabytes: usize },
    IpcDisconnect,
    PipeCorruption,
    DiskWriteFailure,
    NetworkDrop,
    GpuUnavailable,
}

/// Global armed chaos failure state container and physical memory exhaustion buffer.
struct ChaosState {
    active_scenarios: HashSet<ChaosScenario>,
    oom_pressure_buffers: Vec<Vec<u8>>,
}

static ACTIVE_CHAOS: OnceLock<Arc<Mutex<ChaosState>>> = OnceLock::new();

fn get_chaos_state() -> &'static Arc<Mutex<ChaosState>> {
    ACTIVE_CHAOS.get_or_init(|| {
        Arc::new(Mutex::new(ChaosState {
            active_scenarios: HashSet::new(),
            oom_pressure_buffers: Vec::new(),
        }))
    })
}

/// Adversarial Chaos Testing harness injecting runtime failures to verify engine resilience.
pub struct ChaosHarness;

impl ChaosHarness {
    /// Injects and arms a specified chaos failure scenario in the runtime environment.
    /// Performs real physical side-effects where applicable (e.g. real heap allocations for OOM pressure).
    pub fn inject_failure(scenario: &ChaosScenario) -> Result<String> {
        let state_arc = get_chaos_state();
        let mut state = state_arc.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {e}"))?;
        state.active_scenarios.insert(scenario.clone());

        match scenario {
            ChaosScenario::WidgetCrash { widget_id } => {
                warn!(widget_id = %widget_id, "Chaos injection armed: widget crash fault");
                Ok(format!("Injected crash fault into widget '{}'", widget_id))
            }
            ChaosScenario::OomAllocation { widget_id, megabytes } => {
                let mb = *megabytes;
                warn!(widget_id = %widget_id, mb = mb, "Chaos injection: allocating physical RAM to generate real OOM memory pressure");
                // Actually allocate `mb` megabytes on the heap to induce real memory pressure
                let bytes_to_allocate = mb.saturating_mul(1024 * 1024);
                let buffer = vec![0xAAu8; bytes_to_allocate];
                state.oom_pressure_buffers.push(buffer);

                let total_held_mb: usize = state.oom_pressure_buffers.iter().map(|b| b.len() / (1024 * 1024)).sum();
                info!("Chaos Harness: Total allocated OOM pressure buffer is now {} MB", total_held_mb);
                Ok(format!("Injected real OOM memory allocation of {} MB (Total held: {} MB) for widget '{}'", mb, total_held_mb, widget_id))
            }
            ChaosScenario::IpcDisconnect => {
                warn!("Chaos injection armed: IPC named pipe disconnect");
                Ok("Injected IPC pipe disconnect".to_string())
            }
            ChaosScenario::PipeCorruption => {
                warn!("Chaos injection armed: IPC pipe payload corruption");
                Ok("Injected IPC payload corruption".to_string())
            }
            ChaosScenario::DiskWriteFailure => {
                warn!("Chaos injection armed: simulated disk write I/O failure");
                Ok("Injected disk I/O failure".to_string())
            }
            ChaosScenario::NetworkDrop => {
                warn!("Chaos injection armed: simulated network interface drop");
                Ok("Injected network drop".to_string())
            }
            ChaosScenario::GpuUnavailable => {
                warn!("Chaos injection armed: simulated DXGI GPU device loss (DXGI_ERROR_DEVICE_REMOVED: 0x887A0005)");
                Ok("Injected GPU device loss (DXGI_ERROR_DEVICE_REMOVED 0x887A0005)".to_string())
            }
        }
    }

    /// Corrupts an IPC JSON payload if `PipeCorruption` is currently armed.
    pub fn apply_pipe_corruption(payload: &str) -> String {
        if Self::is_failure_armed_kind("PipeCorruption") {
            warn!("Chaos Harness: Corrupting outgoing IPC buffer due to armed PipeCorruption fault!");
            let mut corrupted = payload.to_string();
            if corrupted.len() > 5 {
                corrupted.replace_range(1..5, "###CORRUPTED_PAYLOAD###");
            } else {
                corrupted = "{INVALID_JSON_TOKEN}".to_string();
            }
            corrupted
        } else {
            payload.to_string()
        }
    }

    /// Checks whether a specific chaos failure point is currently armed.
    pub fn is_failure_armed(scenario: &ChaosScenario) -> bool {
        let state_arc = get_chaos_state();
        state_arc.lock().map(|s| s.active_scenarios.contains(scenario)).unwrap_or(false)
    }

    /// Checks whether any failure of a specific variant name is armed.
    pub fn is_failure_armed_kind(kind: &str) -> bool {
        let state_arc = get_chaos_state();
        state_arc.lock().map(|s| {
            s.active_scenarios.iter().any(|sc| match (sc, kind) {
                (ChaosScenario::IpcDisconnect, "IpcDisconnect") => true,
                (ChaosScenario::PipeCorruption, "PipeCorruption") => true,
                (ChaosScenario::DiskWriteFailure, "DiskWriteFailure") => true,
                (ChaosScenario::NetworkDrop, "NetworkDrop") => true,
                (ChaosScenario::GpuUnavailable, "GpuUnavailable") => true,
                (ChaosScenario::WidgetCrash { .. }, "WidgetCrash") => true,
                (ChaosScenario::OomAllocation { .. }, "OomAllocation") => true,
                _ => false,
            })
        }).unwrap_or(false)
    }

    /// Clears and disarms a specific chaos failure scenario, releasing any physical resources held.
    pub fn clear_failure(scenario: &ChaosScenario) -> bool {
        let state_arc = get_chaos_state();
        if let Ok(mut state) = state_arc.lock() {
            let removed = state.active_scenarios.remove(scenario);
            if matches!(scenario, ChaosScenario::OomAllocation { .. }) {
                state.oom_pressure_buffers.clear();
                info!("Chaos Harness: Released physical OOM allocation buffers.");
            }
            if removed {
                info!("Chaos failure point disarmed: {:?}", scenario);
            }
            removed
        } else {
            false
        }
    }

    /// Clears all active chaos failures and frees all retained memory buffers.
    pub fn reset() {
        let state_arc = get_chaos_state();
        if let Ok(mut state) = state_arc.lock() {
            state.active_scenarios.clear();
            state.oom_pressure_buffers.clear();
            info!("All chaos failure points reset and physical pressure buffers released.");
        }
    }

    /// Returns the count of currently armed chaos failure scenarios.
    pub fn active_failure_count() -> usize {
        let state_arc = get_chaos_state();
        state_arc.lock().map(|s| s.active_scenarios.len()).unwrap_or(0)
    }

    /// Returns total megabytes of physical memory currently held by OOM pressure injection.
    pub fn physical_oom_held_mb() -> usize {
        let state_arc = get_chaos_state();
        state_arc.lock().map(|s| s.oom_pressure_buffers.iter().map(|b| b.len() / (1024 * 1024)).sum()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_failure_injection_lifecycle() {
        ChaosHarness::reset();
        assert_eq!(ChaosHarness::active_failure_count(), 0);

        let scenario = ChaosScenario::GpuUnavailable;
        assert!(!ChaosHarness::is_failure_armed(&scenario));

        let res = ChaosHarness::inject_failure(&scenario).unwrap();
        assert!(res.contains("GPU device loss"));
        assert!(ChaosHarness::is_failure_armed(&scenario));
        assert!(ChaosHarness::is_failure_armed_kind("GpuUnavailable"));
        assert_eq!(ChaosHarness::active_failure_count(), 1);

        assert!(ChaosHarness::clear_failure(&scenario));
        assert!(!ChaosHarness::is_failure_armed(&scenario));
        assert_eq!(ChaosHarness::active_failure_count(), 0);
    }

    #[test]
    fn test_real_oom_physical_allocation_pressure() {
        ChaosHarness::reset();
        let oom_scenario = ChaosScenario::OomAllocation {
            widget_id: "heavy_widget".to_string(),
            megabytes: 8, // allocate real 8 MB
        };

        let res = ChaosHarness::inject_failure(&oom_scenario).unwrap();
        assert!(res.contains("8 MB"));
        assert_eq!(ChaosHarness::physical_oom_held_mb(), 8);

        // Disarm and verify physical buffer is freed
        ChaosHarness::clear_failure(&oom_scenario);
        assert_eq!(ChaosHarness::physical_oom_held_mb(), 0);
    }

    #[test]
    fn test_pipe_corruption_injection() {
        ChaosHarness::reset();
        let payload = r#"{"GetStatus":{}}"#;
        assert_eq!(ChaosHarness::apply_pipe_corruption(payload), payload);

        let _ = ChaosHarness::inject_failure(&ChaosScenario::PipeCorruption);
        let corrupted = ChaosHarness::apply_pipe_corruption(payload);
        assert_ne!(corrupted, payload);
        assert!(corrupted.contains("CORRUPTED"));

        ChaosHarness::reset();
        assert_eq!(ChaosHarness::apply_pipe_corruption(payload), payload);
    }
}
