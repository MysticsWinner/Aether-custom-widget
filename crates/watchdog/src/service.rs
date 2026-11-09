//! Windows Service Control Manager (SCM) Wrapper (`sc.exe` / `SERVICE_STATUS`)
//!
//! Provides service lifecycle state transitions, SCM control code handling
//! (`SERVICE_CONTROL_STOP`, `SERVICE_CONTROL_PAUSE`, `SERVICE_CONTROL_CONTINUE`,
//! `SERVICE_CONTROL_INTERROGATE`), and `sc.exe` registration command synthesis
//! for running `core_engine.exe` / `aether_watchdog.exe` as a background Windows Service.

use serde::{Deserialize, Serialize};

/// Windows Service execution state matching `SERVICE_STATUS::dwCurrentState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowsServiceState {
    Stopped = 1,
    StartPending = 2,
    StopPending = 3,
    Running = 4,
    ContinuePending = 5,
    PausePending = 6,
    Paused = 7,
}

/// Windows Service control codes sent by `services.msc` / SCM (`LPHANDLER_FUNCTION_EX`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceControlCode {
    Stop = 1,
    Pause = 2,
    Continue = 3,
    Interrogate = 4,
    Shutdown = 5,
}

/// Configuration for registering the Aether daemon with the Windows Service Control Manager.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowsServiceConfig {
    pub service_name: String,
    pub display_name: String,
    pub description: String,
    pub binary_path: String,
    pub auto_start: bool,
    pub failure_restart_delay_ms: u32,
}

impl Default for WindowsServiceConfig {
    fn default() -> Self {
        Self {
            service_name: "AetherEngineService".to_string(),
            display_name: "Aether Desktop Customization Engine".to_string(),
            description: "Hardware-accelerated zero-trust desktop widget daemon and telemetry host."
                .to_string(),
            binary_path: r"%LOCALAPPDATA%\Aether\bin\core_engine.exe --service".to_string(),
            auto_start: true,
            failure_restart_delay_ms: 5000,
        }
    }
}

impl WindowsServiceConfig {
    pub fn new(service_name: impl Into<String>, binary_path: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            binary_path: binary_path.into(),
            ..Self::default()
        }
    }

    /// Generates the `sc.exe create` command line to register the Windows Service.
    pub fn build_sc_create_command(&self) -> String {
        let start_mode = if self.auto_start { "auto" } else { "demand" };
        format!(
            "sc.exe create \"{}\" binPath= \"{}\" DisplayName= \"{}\" start= {}",
            self.service_name, self.binary_path, self.display_name, start_mode
        )
    }

    /// Generates the `sc.exe failure` command line to configure automatic SCM crash recovery.
    pub fn build_sc_failure_recovery_command(&self) -> String {
        format!(
            "sc.exe failure \"{}\" reset= 60 actions= restart/{}/restart/{}/restart/{}",
            self.service_name,
            self.failure_restart_delay_ms,
            self.failure_restart_delay_ms,
            self.failure_restart_delay_ms
        )
    }
}

/// Stateful Windows Service Control Handler managing SCM state transitions and checkpoints.
#[derive(Debug, Clone)]
pub struct WindowsServiceRunner {
    config: WindowsServiceConfig,
    state: WindowsServiceState,
    checkpoint: u32,
    exit_code: u32,
}

impl WindowsServiceRunner {
    pub fn new(config: WindowsServiceConfig) -> Self {
        Self {
            config,
            state: WindowsServiceState::Stopped,
            checkpoint: 0,
            exit_code: 0,
        }
    }

    pub fn config(&self) -> &WindowsServiceConfig {
        &self.config
    }

    pub fn state(&self) -> WindowsServiceState {
        self.state
    }

    pub fn checkpoint(&self) -> u32 {
        self.checkpoint
    }

    /// Transitions the service through `StartPending` to `Running`.
    pub fn start(&mut self) -> WindowsServiceState {
        self.state = WindowsServiceState::StartPending;
        self.checkpoint += 1;
        self.state = WindowsServiceState::Running;
        self.checkpoint = 0;
        self.state
    }

    /// Handles an incoming `ServiceControlCode` from the Windows Service Control Manager.
    pub fn handle_control_code(&mut self, code: ServiceControlCode) -> WindowsServiceState {
        match code {
            ServiceControlCode::Stop | ServiceControlCode::Shutdown => {
                self.state = WindowsServiceState::StopPending;
                self.checkpoint += 1;
                self.state = WindowsServiceState::Stopped;
                self.checkpoint = 0;
            }
            ServiceControlCode::Pause => {
                if self.state == WindowsServiceState::Running {
                    self.state = WindowsServiceState::PausePending;
                    self.state = WindowsServiceState::Paused;
                }
            }
            ServiceControlCode::Continue => {
                if self.state == WindowsServiceState::Paused {
                    self.state = WindowsServiceState::ContinuePending;
                    self.state = WindowsServiceState::Running;
                }
            }
            ServiceControlCode::Interrogate => {}
        }
        self.state
    }

    pub fn exit_code(&self) -> u32 {
        self.exit_code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_service_runner_lifecycle_and_sc_commands() {
        let cfg = WindowsServiceConfig::new(
            "AetherEngineService",
            r"C:\Program Files\Aether\core_engine.exe --service",
        );
        let create_cmd = cfg.build_sc_create_command();
        assert!(create_cmd.contains("sc.exe create \"AetherEngineService\""));
        assert!(create_cmd.contains("start= auto"));

        let recovery_cmd = cfg.build_sc_failure_recovery_command();
        assert!(recovery_cmd.contains("actions= restart/5000/restart/5000/restart/5000"));

        let mut runner = WindowsServiceRunner::new(cfg);
        assert_eq!(runner.state(), WindowsServiceState::Stopped);

        assert_eq!(runner.start(), WindowsServiceState::Running);
        assert_eq!(
            runner.handle_control_code(ServiceControlCode::Pause),
            WindowsServiceState::Paused
        );
        assert_eq!(
            runner.handle_control_code(ServiceControlCode::Continue),
            WindowsServiceState::Running
        );
        assert_eq!(
            runner.handle_control_code(ServiceControlCode::Stop),
            WindowsServiceState::Stopped
        );
        assert_eq!(runner.exit_code(), 0);
    }
}
