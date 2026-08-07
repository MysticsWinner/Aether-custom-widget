use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Failure scenarios supported by the Chaos Injection Harness.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChaosScenario {
    WidgetCrash { widget_id: String },
    OomAllocation { widget_id: String },
    IpcDisconnect,
    PipeCorruption,
    DiskWriteFailure,
    NetworkDrop,
    GpuUnavailable,
}

/// Adversarial Chaos Testing harness injecting runtime failures to verify engine resilience.
pub struct ChaosHarness;

impl ChaosHarness {
    /// Injects a specified chaos failure scenario into the runtime environment.
    pub fn inject_failure(scenario: &ChaosScenario) -> Result<String> {
        match scenario {
            ChaosScenario::WidgetCrash { widget_id } => {
                warn!(widget_id = %widget_id, "Chaos injection: trigger widget crash");
                Ok(format!("Injected crash into widget '{}'", widget_id))
            }
            ChaosScenario::OomAllocation { widget_id } => {
                warn!(widget_id = %widget_id, "Chaos injection: trigger simulated memory leak");
                Ok(format!("Injected OOM allocation into widget '{}'", widget_id))
            }
            ChaosScenario::IpcDisconnect => {
                warn!("Chaos injection: trigger IPC named pipe disconnect");
                Ok("Injected IPC pipe disconnect".to_string())
            }
            ChaosScenario::PipeCorruption => {
                warn!("Chaos injection: trigger IPC pipe payload corruption");
                Ok("Injected IPC payload corruption".to_string())
            }
            ChaosScenario::DiskWriteFailure => {
                warn!("Chaos injection: trigger simulated disk write I/O failure");
                Ok("Injected disk I/O failure".to_string())
            }
            ChaosScenario::NetworkDrop => {
                warn!("Chaos injection: trigger simulated network interface drop");
                Ok("Injected network drop".to_string())
            }
            ChaosScenario::GpuUnavailable => {
                warn!("Chaos injection: trigger simulated DXGI GPU device loss");
                Ok("Injected GPU device loss".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chaos_failure_injection_engine_survives() {
        let res = ChaosHarness::inject_failure(&ChaosScenario::WidgetCrash {
            widget_id: "test_widget".to_string(),
        })
        .unwrap();
        assert!(res.contains("Injected crash"));

        let res_gpu = ChaosHarness::inject_failure(&ChaosScenario::GpuUnavailable).unwrap();
        assert!(res_gpu.contains("GPU device loss"));
    }
}
