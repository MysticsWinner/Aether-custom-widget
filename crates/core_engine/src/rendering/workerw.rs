//! Windows Desktop WorkerW Shell Attachment & Recovery Subsystem
//!
//! Handles safe attachment of widget surfaces behind desktop icons (WorkerW parent window),
//! monitors Explorer.exe lifecycle, and recovers composition surfaces automatically upon
//! shell restart or crash.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

/// Current attachment state of the desktop surface beneath desktop icons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerWSurfaceState {
    /// Initial unattached state before any surface acquisition.
    Uninitialized,
    /// Successfully bound to an active WorkerW window handle.
    Bound {
        hwnd: isize,
        acquired_at_ms: u64,
    },
    /// Detected that Explorer.exe has crashed or restarted (HWND is dead/invalid).
    ExplorerRestartDetected,
    /// Currently enumerating windows and dispatching Progman messages to spawn a new WorkerW.
    Rebinding,
    /// Failed to attach to WorkerW; fallback to floating topmost layered window is required.
    Failed {
        reason: String,
    },
}

/// Manages the desktop background WorkerW surface lifecycle and shell crash recovery.
#[derive(Debug, Clone)]
pub struct DesktopSurfaceManager {
    state: WorkerWSurfaceState,
    recovery_attempts: u32,
    max_recovery_attempts: u32,
    last_health_check_ms: u64,
}

impl DesktopSurfaceManager {
    /// Creates a new `DesktopSurfaceManager` ready to acquire the desktop surface.
    pub fn new() -> Self {
        Self {
            state: WorkerWSurfaceState::Uninitialized,
            recovery_attempts: 0,
            max_recovery_attempts: 5,
            last_health_check_ms: current_time_ms(),
        }
    }

    /// Returns the current state of the desktop surface.
    pub fn state(&self) -> &WorkerWSurfaceState {
        &self.state
    }

    /// Returns the active WorkerW HWND if currently bound.
    pub fn current_hwnd(&self) -> Option<isize> {
        match self.state {
            WorkerWSurfaceState::Bound { hwnd, .. } => Some(hwnd),
            _ => None,
        }
    }

    /// Returns whether the surface is currently valid and attached to a live HWND.
    pub fn is_surface_valid(&self) -> bool {
        if let Some(hwnd) = self.current_hwnd() {
            is_workerw_valid_hwnd(hwnd)
        } else {
            false
        }
    }

    /// Attempts initial acquisition of the desktop WorkerW handle.
    pub fn acquire_surface(&mut self) -> anyhow::Result<isize> {
        info!("Acquiring desktop WorkerW surface behind icons...");
        self.state = WorkerWSurfaceState::Rebinding;

        #[cfg(windows)]
        {
            match find_desktop_workerw_hwnd() {
                Some(hwnd) => {
                    let raw_hwnd = hwnd.0 as isize;
                    let now = current_time_ms();
                    info!("Successfully acquired WorkerW handle: 0x{:X}", raw_hwnd);
                    self.state = WorkerWSurfaceState::Bound {
                        hwnd: raw_hwnd,
                        acquired_at_ms: now,
                    };
                    self.recovery_attempts = 0;
                    Ok(raw_hwnd)
                }
                None => {
                    let err = "Progman message did not spawn WorkerW or SHELLDLL_DefView not found".to_string();
                    warn!("{}", err);
                    self.state = WorkerWSurfaceState::Failed { reason: err.clone() };
                    anyhow::bail!(err);
                }
            }
        }

        #[cfg(not(windows))]
        {
            let simulated_hwnd = 0x1337usize as isize;
            self.state = WorkerWSurfaceState::Bound {
                hwnd: simulated_hwnd,
                acquired_at_ms: current_time_ms(),
            };
            Ok(simulated_hwnd)
        }
    }

    /// Checks the health of the current surface binding. If the HWND is no longer valid
    /// (e.g. Explorer was killed or restarted), triggers reacquisition and returns `Some(new_hwnd)`.
    pub fn check_health_and_rebind(&mut self) -> anyhow::Result<Option<isize>> {
        self.last_health_check_ms = current_time_ms();

        match &self.state {
            WorkerWSurfaceState::Bound { hwnd, .. } => {
                let current_raw = *hwnd;
                if is_workerw_valid_hwnd(current_raw) {
                    // Surface is alive and well
                    return Ok(None);
                }

                warn!("WorkerW HWND 0x{:X} is no longer valid! Shell restart detected.", current_raw);
                self.state = WorkerWSurfaceState::ExplorerRestartDetected;
                self.recovery_attempts += 1;

                if self.recovery_attempts > self.max_recovery_attempts {
                    let reason = format!(
                        "Exceeded maximum recovery attempts ({}) for WorkerW rebind",
                        self.max_recovery_attempts
                    );
                    error!("{}", reason);
                    self.state = WorkerWSurfaceState::Failed { reason: reason.clone() };
                    anyhow::bail!(reason);
                }

                let new_hwnd = self.acquire_surface()?;
                info!("Desktop surface successfully rebound to new WorkerW HWND: 0x{:X}", new_hwnd);
                Ok(Some(new_hwnd))
            }
            WorkerWSurfaceState::Uninitialized | WorkerWSurfaceState::ExplorerRestartDetected | WorkerWSurfaceState::Failed { .. } => {
                let new_hwnd = self.acquire_surface()?;
                Ok(Some(new_hwnd))
            }
            WorkerWSurfaceState::Rebinding => {
                debug!("WorkerW rebind is already in progress");
                Ok(None)
            }
        }
    }

    /// Manually marks the surface as detached (e.g. when Explorer shutdown event is received).
    pub fn mark_detached(&mut self) {
        self.state = WorkerWSurfaceState::ExplorerRestartDetected;
    }

    /// Forces an immediate rebind regardless of current state.
    pub fn force_rebind(&mut self) -> anyhow::Result<isize> {
        self.state = WorkerWSurfaceState::Rebinding;
        self.acquire_surface()
    }
}

impl Default for DesktopSurfaceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if an HWND represents an existing, valid Win32 window.
#[cfg(windows)]
pub fn is_workerw_valid_hwnd(hwnd: isize) -> bool {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::IsWindow;

    if hwnd == 0 {
        return false;
    }
    unsafe { IsWindow(HWND(hwnd as *mut std::ffi::c_void)).as_bool() }
}

#[cfg(not(windows))]
pub fn is_workerw_valid_hwnd(hwnd: isize) -> bool {
    hwnd != 0
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Resolves the WorkerW desktop window handle spawned behind Windows desktop icons via Progman 0x052C message.
#[cfg(windows)]
pub fn find_desktop_workerw_hwnd() -> Option<windows::Win32::Foundation::HWND> {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, FindWindowExW, FindWindowW, SendMessageTimeoutW, SMTO_NORMAL,
    };

    unsafe {
        let progman = match FindWindowW(windows::core::w!("Progman"), None) {
            Ok(hwnd) if !hwnd.0.is_null() => hwnd,
            _ => return None,
        };

        let mut result = 0;
        let _ = SendMessageTimeoutW(
            progman,
            0x052C,
            WPARAM(0),
            LPARAM(0),
            SMTO_NORMAL,
            1000,
            Some(&mut result),
        );

        let mut workerw_hwnd = HWND(std::ptr::null_mut());

        unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let target_ptr = lparam.0 as *mut HWND;
            if let Ok(shell_hwnd) = FindWindowExW(
                hwnd,
                HWND(std::ptr::null_mut()),
                windows::core::w!("SHELLDLL_DefView"),
                None,
            ) {
                if !shell_hwnd.0.is_null() {
                    if let Ok(found_workerw) = FindWindowExW(
                        HWND(std::ptr::null_mut()),
                        hwnd,
                        windows::core::w!("WorkerW"),
                        None,
                    ) {
                        *target_ptr = found_workerw;
                        return BOOL(0);
                    }
                }
            }
            BOOL(1)
        }

        let _ = EnumWindows(
            Some(enum_window_proc),
            LPARAM(&mut workerw_hwnd as *mut HWND as isize),
        );

        if !workerw_hwnd.0.is_null() {
            Some(workerw_hwnd)
        } else {
            None
        }
    }
}

#[cfg(not(windows))]
pub fn find_desktop_workerw_hwnd() -> Option<()> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workerw_window_query() {
        let _result = find_desktop_workerw_hwnd();
    }

    #[test]
    fn test_surface_manager_lifecycle_and_recovery() {
        let mut manager = DesktopSurfaceManager::new();
        assert_eq!(manager.state(), &WorkerWSurfaceState::Uninitialized);
        assert!(!manager.is_surface_valid());

        // Initial acquisition
        let _ = manager.acquire_surface();
        // Even if WorkerW is not present in CI / non-desktop environment, state transitions cleanly
        match manager.state() {
            WorkerWSurfaceState::Bound { hwnd, .. } => {
                assert_eq!(manager.current_hwnd(), Some(*hwnd));
                // Mark detached to simulate Explorer restart
                manager.mark_detached();
                assert_eq!(manager.state(), &WorkerWSurfaceState::ExplorerRestartDetected);
            }
            WorkerWSurfaceState::Failed { .. } => {
                // If run without interactive shell / headless, fail state is handled
                assert_eq!(manager.current_hwnd(), None);
            }
            _ => {}
        }
    }

    #[test]
    fn test_is_workerw_valid_hwnd_null_is_false() {
        assert!(!is_workerw_valid_hwnd(0));
    }

    #[test]
    fn test_workerw_explorer_crash_and_rebind_simulation() {
        let mut manager = DesktopSurfaceManager::new();

        // 1. Manually bind initial state
        manager.state = WorkerWSurfaceState::Bound {
            hwnd: 0xDEAD,
            acquired_at_ms: 1000,
        };
        assert_eq!(manager.current_hwnd(), Some(0xDEAD));

        // 2. Simulate Explorer termination (mark detached)
        manager.mark_detached();
        assert_eq!(manager.state(), &WorkerWSurfaceState::ExplorerRestartDetected);

        // 3. Simulated recovery attempts counter progression
        manager.recovery_attempts = 6; // Exceeds max_recovery_attempts (5)
        let res = manager.check_health_and_rebind();
        assert!(res.is_err(), "Must fail gracefully when recovery attempts are exhausted");
        assert!(matches!(manager.state(), WorkerWSurfaceState::Failed { .. }));
    }
}
