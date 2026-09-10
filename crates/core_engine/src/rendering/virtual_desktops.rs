//! Windows 11 Virtual Desktop Management & Canonical Multi-Monitor / DPI Virtualization
//!
//! Enforces a strict separation between Canonical DIPs (Device-Independent Pixels at 96 DPI)
//! and Physical Display Pixels across heterogeneous multi-monitor setups.
//! Handles Windows 11 COM virtual desktop pinning and per-monitor dynamic DPI scaling.

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

/// Strongly typed coordinate in Canonical Device-Independent Pixels (96 DPI base).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DipRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl DipRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn to_rectf(&self) -> RectF {
        RectF::new(self.x, self.y, self.width, self.height)
    }

    pub fn from_rectf(rect: RectF) -> Self {
        Self::new(rect.x, rect.y, rect.width, rect.height)
    }

    pub fn to_physical(&self, dpi_scale: &DpiMonitorScale) -> PhysicalRect {
        PhysicalRect::from_rectf(dpi_scale.scale_rect(self.to_rectf()))
    }
}

/// Strongly typed coordinate in physical hardware display pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PhysicalRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl PhysicalRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn to_rectf(&self) -> RectF {
        RectF::new(self.x, self.y, self.width, self.height)
    }

    pub fn from_rectf(rect: RectF) -> Self {
        Self::new(rect.x, rect.y, rect.width, rect.height)
    }

    pub fn to_dip(&self, dpi_scale: &DpiMonitorScale) -> DipRect {
        DipRect::from_rectf(dpi_scale.unscale_rect(self.to_rectf()))
    }
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

    /// Scales DIP coordinates to physical pixels.
    pub fn scale_rect(&self, rect: RectF) -> RectF {
        RectF::new(
            rect.x * self.dpi_scale_x,
            rect.y * self.dpi_scale_y,
            rect.width * self.dpi_scale_x,
            rect.height * self.dpi_scale_y,
        )
    }

    /// Unscales physical pixels back to DIP coordinates.
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

/// Hardware and virtual screen metadata for a connected physical display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub monitor_id: String,
    pub device_name: String,
    pub physical_bounds: RectF,
    pub work_area: RectF,
    pub dpi_scale: DpiMonitorScale,
    pub is_primary: bool,
}

impl MonitorInfo {
    pub fn new(
        monitor_id: impl Into<String>,
        device_name: impl Into<String>,
        physical_bounds: RectF,
        work_area: RectF,
        dpi: u32,
        is_primary: bool,
    ) -> Self {
        Self {
            monitor_id: monitor_id.into(),
            device_name: device_name.into(),
            physical_bounds,
            work_area,
            dpi_scale: DpiMonitorScale::from_dpi(dpi),
            is_primary,
        }
    }
}

/// Multi-monitor virtual desktop topology map.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesktopTopology {
    pub monitors: Vec<MonitorInfo>,
    pub virtual_screen_bounds: RectF,
}

impl DesktopTopology {
    pub fn new() -> Self {
        Self {
            monitors: Vec::new(),
            virtual_screen_bounds: RectF::zero(),
        }
    }

    /// Adds a monitor to the active topology and recalculates the virtual screen bounds.
    pub fn add_monitor(&mut self, monitor: MonitorInfo) {
        self.virtual_screen_bounds = if self.monitors.is_empty() {
            monitor.physical_bounds
        } else {
            self.virtual_screen_bounds.union(&monitor.physical_bounds)
        };
        self.monitors.push(monitor);
    }

    /// Returns the monitor containing the specified physical coordinates.
    pub fn monitor_at_point(&self, x: f32, y: f32) -> Option<&MonitorInfo> {
        self.monitors.iter().find(|m| {
            x >= m.physical_bounds.x
                && x < m.physical_bounds.right()
                && y >= m.physical_bounds.y
                && y < m.physical_bounds.bottom()
        })
    }

    /// Returns the primary monitor, if configured.
    pub fn primary_monitor(&self) -> Option<&MonitorInfo> {
        self.monitors.iter().find(|m| m.is_primary).or_else(|| self.monitors.first())
    }

    /// Removes a disconnected monitor and recalculates the virtual screen bounds.
    pub fn remove_monitor(&mut self, monitor_id: &str) -> bool {
        let initial_len = self.monitors.len();
        self.monitors.retain(|m| m.monitor_id != monitor_id);
        if self.monitors.len() != initial_len {
            self.recalculate_bounds();
            true
        } else {
            false
        }
    }

    /// Recalculates virtual screen bounding box across all active physical monitors.
    pub fn recalculate_bounds(&mut self) {
        if self.monitors.is_empty() {
            self.virtual_screen_bounds = RectF::zero();
            return;
        }
        let mut bounds = self.monitors[0].physical_bounds;
        for m in &self.monitors[1..] {
            bounds = bounds.union(&m.physical_bounds);
        }
        self.virtual_screen_bounds = bounds;
    }

    /// Converts a canonical DIP rectangle into physical pixels for the target monitor.
    pub fn dip_to_physical_on_monitor(&self, monitor_id: &str, dip_rect: DipRect) -> Option<PhysicalRect> {
        self.monitors
            .iter()
            .find(|m| m.monitor_id == monitor_id)
            .map(|m| dip_rect.to_physical(&m.dpi_scale))
    }

    /// Converts a physical display rectangle into canonical DIP coordinates for the target monitor.
    pub fn physical_to_dip_on_monitor(&self, monitor_id: &str, physical_rect: PhysicalRect) -> Option<DipRect> {
        self.monitors
            .iter()
            .find(|m| m.monitor_id == monitor_id)
            .map(|m| physical_rect.to_dip(&m.dpi_scale))
    }
}

/// Manager for Virtual Desktop states and DPI coordinate translations.
#[derive(Debug, Clone)]
pub struct VirtualDesktopManager {
    pinning: VirtualDesktopPinning,
    monitor_dpi: DpiMonitorScale,
    topology: DesktopTopology,
}

impl VirtualDesktopManager {
    /// Creates a new `VirtualDesktopManager`.
    pub fn new(pinning: VirtualDesktopPinning, dpi: u32) -> Self {
        Self {
            pinning,
            monitor_dpi: DpiMonitorScale::from_dpi(dpi),
            topology: DesktopTopology::new(),
        }
    }

    /// Returns current pinning mode.
    pub fn pinning(&self) -> &VirtualDesktopPinning {
        &self.pinning
    }

    /// Sets pinning mode.
    pub fn set_pinning(&mut self, pinning: VirtualDesktopPinning) {
        self.pinning = pinning;
    }

    /// Updates monitor DPI scale upon `WM_DPICHANGED`.
    pub fn update_dpi(&mut self, new_dpi: u32) {
        self.monitor_dpi = DpiMonitorScale::from_dpi(new_dpi);
    }

    /// Returns the active DPI scaling structure.
    pub fn dpi_scale(&self) -> &DpiMonitorScale {
        &self.monitor_dpi
    }

    /// Returns a reference to the multi-monitor desktop topology.
    pub fn topology(&self) -> &DesktopTopology {
        &self.topology
    }

    /// Returns a mutable reference to the multi-monitor desktop topology.
    pub fn topology_mut(&mut self) -> &mut DesktopTopology {
        &mut self.topology
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

    #[test]
    fn test_multi_monitor_topology_and_dip_conversions() {
        let mut topology = DesktopTopology::new();

        // Monitor 1: 1920x1080 at 100% DPI (96)
        topology.add_monitor(MonitorInfo::new(
            "mon-1",
            r"\\.\DISPLAY1",
            RectF::new(0.0, 0.0, 1920.0, 1080.0),
            RectF::new(0.0, 0.0, 1920.0, 1040.0),
            96,
            true,
        ));

        // Monitor 2: 3840x2160 at 200% DPI (192), positioned to the right
        topology.add_monitor(MonitorInfo::new(
            "mon-2",
            r"\\.\DISPLAY2",
            RectF::new(1920.0, 0.0, 3840.0, 2160.0),
            RectF::new(1920.0, 0.0, 3840.0, 2112.0),
            192,
            false,
        ));

        assert_eq!(topology.virtual_screen_bounds.width, 5760.0);
        assert_eq!(topology.virtual_screen_bounds.height, 2160.0);

        let mon1 = topology.monitor_at_point(500.0, 500.0).unwrap();
        assert_eq!(mon1.monitor_id, "mon-1");

        let mon2 = topology.monitor_at_point(2000.0, 500.0).unwrap();
        assert_eq!(mon2.monitor_id, "mon-2");

        // Convert 100x100 DIP widget on mon-2 (200% scale) -> 200x200 physical pixels
        let dip_box = DipRect::new(10.0, 10.0, 100.0, 100.0);
        let phys_box = topology.dip_to_physical_on_monitor("mon-2", dip_box).unwrap();
        assert_eq!(phys_box.width, 200.0);
        assert_eq!(phys_box.height, 200.0);
    }

    #[test]
    fn test_topology_monitor_hotplug_and_dpi_change_reconciliation() {
        let mut topology = DesktopTopology::new();
        topology.add_monitor(MonitorInfo::new(
            "mon-1",
            r"\\.\DISPLAY1",
            RectF::new(0.0, 0.0, 1920.0, 1080.0),
            RectF::new(0.0, 0.0, 1920.0, 1040.0),
            96,
            true,
        ));
        topology.add_monitor(MonitorInfo::new(
            "mon-2",
            r"\\.\DISPLAY2",
            RectF::new(1920.0, 0.0, 1920.0, 1080.0),
            RectF::new(1920.0, 0.0, 1920.0, 1040.0),
            96,
            false,
        ));

        assert_eq!(topology.virtual_screen_bounds.width, 3840.0);

        // Disconnect monitor 2 (hotplug remove)
        assert!(topology.remove_monitor("mon-2"));
        assert_eq!(topology.virtual_screen_bounds.width, 1920.0);
        assert!(topology.monitor_at_point(2500.0, 500.0).is_none());

        // Reconnect monitor 2 with 150% DPI (144 DPI)
        topology.add_monitor(MonitorInfo::new(
            "mon-2",
            r"\\.\DISPLAY2",
            RectF::new(1920.0, 0.0, 2560.0, 1440.0),
            RectF::new(1920.0, 0.0, 2560.0, 1400.0),
            144,
            false,
        ));
        assert_eq!(topology.virtual_screen_bounds.width, 4480.0);
        let dip_box = DipRect::new(0.0, 0.0, 100.0, 100.0);
        let phys_box = topology.dip_to_physical_on_monitor("mon-2", dip_box).unwrap();
        assert_eq!(phys_box.width, 150.0); // 100 * 1.5 = 150
    }
}
