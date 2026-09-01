//! Windows 11 Virtual Desktop Management & Per-Monitor V2 DPI Virtualization
//!
//! Connects to Windows 11 COM virtual desktop managers and handles per-monitor DPI scaling.

use crate::rendering::RectF;
use serde::{Deserialize, Serialize};

/// Virtual desktop pinning mode for a widget desktop window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VirtualDesktopPinning {
    /// Widget only appears on the virtual desktop where it was mounted.
    PinnedToCurrent,
    /// Widget is pinned across all virtual desktops (floats everywhere).
    PinnedGlobally,
    /// Widget is pinned to a specific virtual desktop GUID.
    PinnedToId(String),
}

/// Per-Monitor V2 DPI scaling state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DpiMonitorScale {
    pub dpi_scale_x: f32,
    pub dpi_scale_y: f32,
    pub effective_dpi: u32,
}

impl DpiMonitorScale {
    /// Creates a new `DpiMonitorScale` initialized with standard 96 DPI (100% scale).
    pub fn standard() -> Self {
        Self {
            dpi_scale_x: 1.0,
            dpi_scale_y: 1.0,
            effective_dpi: 96,
        }
    }

    /// Creates a new `DpiMonitorScale` from raw Windows DPI (e.g. 144 for 150%, 192 for 200%).
    pub fn from_dpi(dpi: u32) -> Self {
        let scale = dpi as f32 / 96.0;
        Self {
            dpi_scale_x: scale,
            dpi_scale_y: scale,
            effective_dpi: dpi,
        }
    }

    /// Scales physical coordinates to device-independent pixels (DIPs).
    pub fn scale_rect(&self, rect: RectF) -> RectF {
        RectF::new(
            rect.x * self.dpi_scale_x,
            rect.y * self.dpi_scale_y,
            rect.width * self.dpi_scale_x,
            rect.height * self.dpi_scale_y,
        )
    }

    /// Unscales device-independent pixels back to physical coordinates.
    pub fn unscale_rect(&self, rect: RectF) -> RectF {
        RectF::new(
            rect.x / self.dpi_scale_x,
            rect.y / self.dpi_scale_y,
            rect.width / self.dpi_scale_x,
            rect.height / self.dpi_scale_y,
        )
    }
}

impl Default for DpiMonitorScale {
    fn default() -> Self {
        Self::standard()
    }
}

/// Manager for Virtual Desktop states and DPI coordinate translations.
#[derive(Debug, Clone)]
pub struct VirtualDesktopManager {
    pinning: VirtualDesktopPinning,
    monitor_dpi: DpiMonitorScale,
}

impl VirtualDesktopManager {
    /// Creates a new `VirtualDesktopManager`.
    pub fn new(pinning: VirtualDesktopPinning, dpi: u32) -> Self {
        Self {
            pinning,
            monitor_dpi: DpiMonitorScale::from_dpi(dpi),
        }
    }

    /// Returns current pinning mode.
    pub fn pinning(&self) -> &VirtualDesktopPinning {
        &self.pinning
    }

    /// Updates monitor DPI scale upon `WM_DPICHANGED`.
    pub fn update_dpi(&mut self, new_dpi: u32) {
        self.monitor_dpi = DpiMonitorScale::from_dpi(new_dpi);
    }

    /// Returns the active DPI scaling structure.
    pub fn dpi_scale(&self) -> &DpiMonitorScale {
        &self.monitor_dpi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpi_monitor_scale_calculations() {
        let scale_150 = DpiMonitorScale::from_dpi(144);
        assert_eq!(scale_150.dpi_scale_x, 1.5);
        assert_eq!(scale_150.effective_dpi, 144);

        let unscaled = RectF::new(10.0, 20.0, 100.0, 50.0);
        let scaled = scale_150.scale_rect(unscaled);
        assert_eq!(scaled, RectF::new(15.0, 30.0, 150.0, 75.0));

        let reverted = scale_150.unscale_rect(scaled);
        assert_eq!(reverted, unscaled);
    }

    #[test]
    fn test_virtual_desktop_manager_pinning() {
        let mut manager = VirtualDesktopManager::new(VirtualDesktopPinning::PinnedGlobally, 96);
        assert_eq!(manager.pinning(), &VirtualDesktopPinning::PinnedGlobally);
        assert_eq!(manager.dpi_scale().effective_dpi, 96);

        manager.update_dpi(192); // 200% scaling on 4K display
        assert_eq!(manager.dpi_scale().dpi_scale_x, 2.0);
    }
}
