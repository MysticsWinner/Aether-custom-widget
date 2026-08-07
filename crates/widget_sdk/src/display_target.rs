use serde::{Deserialize, Serialize};

/// Target monitor display selection for desktop widget placement.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DisplayTarget {
    PrimaryMonitor,
    MonitorIndex(u32),
    AllMonitors,
}

impl Default for DisplayTarget {
    fn default() -> Self {
        Self::PrimaryMonitor
    }
}

/// Desktop Z-order layer placement options.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DesktopLayer {
    /// Bottom-pinned desktop canvas behind desktop icons (WorkerW hook).
    DesktopOverlay,
    /// Standard desktop window layer.
    Normal,
    /// Pinned always on top of all application windows.
    AlwaysOnTop,
}

impl Default for DesktopLayer {
    fn default() -> Self {
        Self::DesktopOverlay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_target_and_layer_defaults() {
        assert_eq!(DisplayTarget::default(), DisplayTarget::PrimaryMonitor);
        assert_eq!(DesktopLayer::default(), DesktopLayer::DesktopOverlay);
    }
}
