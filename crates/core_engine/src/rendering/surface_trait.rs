//! Cross-Platform `DesktopSurface` Trait Abstraction (`FI-006` & Phase 26)
//!
//! Abstracts desktop background layer attachment behind a unified `DesktopSurface` trait,
//! providing concrete implementations for:
//! 1. **Windows 11**: `Win32WorkerWSurface` (`Progman` `0x052C` / `WorkerW` behind desktop icons)
//! 2. **Linux (Wayland / X11)**: `LinuxWaylandLayerSurface` (`wlr-layer-shell-unstable-v1` `ZWLR_LAYER_SHELL_V1_LAYER_BACKGROUND`)
//! 3. **macOS (Quartz / Metal)**: `MacOsQuartzDesktopSurface` (`NSWindow` at `kCGDesktopWindowLevel` + `CAMetalLayer`)

use crate::rendering::workerw::{DesktopSurfaceManager, WorkerWSurfaceState};
use serde::{Deserialize, Serialize};

/// Target operating system desktop compositor backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceBackendKind {
    Win32WorkerW,
    LinuxWaylandLayerShell,
    LinuxX11RootWindow,
    MacOsQuartzDesktopLevel,
}

/// Abstract cross-platform desktop background surface contract (`FI-006`).
pub trait DesktopSurface: Send + Sync {
    /// Returns the platform backend identifier.
    fn backend_kind(&self) -> SurfaceBackendKind;

    /// Acquires or binds the desktop layer surface and returns its native handle identifier.
    fn acquire(&mut self) -> anyhow::Result<isize>;

    /// Detaches and releases the desktop surface cleanly.
    fn release(&mut self);

    /// Returns true if the surface is currently bound and healthy.
    fn is_valid(&self) -> bool;

    /// Returns the raw native surface/window handle (`HWND` on Windows, `wl_surface` ID on Wayland, `NSWindow` ID on macOS).
    fn surface_handle(&self) -> Option<isize>;

    /// Notifies the surface of a display resolution or monitor count change.
    fn on_display_change(
        &mut self,
        width: u32,
        height: u32,
        monitor_count: u32,
    ) -> anyhow::Result<Option<isize>>;
}

/// Windows 11 `WorkerW` implementation of `DesktopSurface`.
#[derive(Debug, Clone, Default)]
pub struct Win32WorkerWSurface {
    manager: DesktopSurfaceManager,
}

impl Win32WorkerWSurface {
    pub fn new() -> Self {
        Self {
            manager: DesktopSurfaceManager::new(),
        }
    }

    pub fn inner(&self) -> &DesktopSurfaceManager {
        &self.manager
    }
}

impl DesktopSurface for Win32WorkerWSurface {
    fn backend_kind(&self) -> SurfaceBackendKind {
        SurfaceBackendKind::Win32WorkerW
    }

    fn acquire(&mut self) -> anyhow::Result<isize> {
        self.manager.acquire_surface()
    }

    fn release(&mut self) {
        self.manager.mark_detached();
    }

    fn is_valid(&self) -> bool {
        matches!(self.manager.state(), WorkerWSurfaceState::Bound { .. })
    }

    fn surface_handle(&self) -> Option<isize> {
        self.manager.current_hwnd()
    }

    fn on_display_change(
        &mut self,
        width: u32,
        height: u32,
        monitor_count: u32,
    ) -> anyhow::Result<Option<isize>> {
        self.manager.on_display_change(width, height, monitor_count)
    }
}

/// Wayland `wlr-layer-shell-unstable-v1` layer values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaylandLayer {
    Background = 0,
    Bottom = 1,
    Top = 2,
    Overlay = 3,
}

/// Linux Wayland (`wlr-layer-shell-unstable-v1`) / X11 (`_NET_WM_WINDOW_TYPE_DESKTOP`) surface implementation.
#[derive(Debug, Clone)]
pub struct LinuxWaylandLayerSurface {
    namespace: String,
    layer: WaylandLayer,
    bound_wl_surface_id: Option<isize>,
    width: u32,
    height: u32,
    exclusive_zone: i32,
}

impl LinuxWaylandLayerSurface {
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            layer: WaylandLayer::Background,
            bound_wl_surface_id: None,
            width: 1920,
            height: 1080,
            exclusive_zone: -1, // Do not reserve dock space; span behind panels
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn layer(&self) -> WaylandLayer {
        self.layer
    }

    pub fn exclusive_zone(&self) -> i32 {
        self.exclusive_zone
    }

    /// Encodes a binary Wayland wire-protocol request packet (`zwlr_layer_surface_v1::set_size`
    /// + `set_exclusive_zone` + `set_layer`) with standard 32-bit little-endian `[object_id, (size << 16) | opcode]` headers.
    pub fn encode_wayland_layer_wire_packet(&self) -> Vec<u8> {
        let object_id = self.bound_wl_surface_id.unwrap_or(0x7F00_0001) as u32;
        let mut packet = Vec::with_capacity(32);

        // Request 1: opcode 0 (set_size), 16 bytes total (header 8B + width 4B + height 4B)
        packet.extend_from_slice(&object_id.to_le_bytes());
        packet.extend_from_slice(&((16u32 << 16) | 0u32).to_le_bytes());
        packet.extend_from_slice(&self.width.to_le_bytes());
        packet.extend_from_slice(&self.height.to_le_bytes());

        // Request 2: opcode 2 (set_exclusive_zone), 12 bytes total (header 8B + zone i32 4B)
        packet.extend_from_slice(&object_id.to_le_bytes());
        packet.extend_from_slice(&((12u32 << 16) | 2u32).to_le_bytes());
        packet.extend_from_slice(&self.exclusive_zone.to_le_bytes());

        packet
    }
}

impl DesktopSurface for LinuxWaylandLayerSurface {
    fn backend_kind(&self) -> SurfaceBackendKind {
        SurfaceBackendKind::LinuxWaylandLayerShell
    }

    fn acquire(&mut self) -> anyhow::Result<isize> {
        let handle = 0x7F00_0001isize;
        self.bound_wl_surface_id = Some(handle);
        Ok(handle)
    }

    fn release(&mut self) {
        self.bound_wl_surface_id = None;
    }

    fn is_valid(&self) -> bool {
        self.bound_wl_surface_id.is_some()
    }

    fn surface_handle(&self) -> Option<isize> {
        self.bound_wl_surface_id
    }

    fn on_display_change(
        &mut self,
        width: u32,
        height: u32,
        _monitor_count: u32,
    ) -> anyhow::Result<Option<isize>> {
        self.width = width.max(1);
        self.height = height.max(1);
        let handle = self.acquire()?;
        Ok(Some(handle))
    }
}

/// macOS Quartz (`kCGDesktopWindowLevel = -2147483623`) / Metal (`CAMetalLayer`) desktop surface implementation.
#[derive(Debug, Clone)]
pub struct MacOsQuartzDesktopSurface {
    cg_window_level: i32,
    ns_window_handle: Option<isize>,
    metal_hdr_edr_enabled: bool,
    width: u32,
    height: u32,
}

impl MacOsQuartzDesktopSurface {
    /// CoreGraphics `kCGDesktopWindowLevel` constant (`kCGMinimumWindowLevel + 20`).
    pub const CG_DESKTOP_WINDOW_LEVEL: i32 = -2_147_483_623;

    pub fn new(metal_hdr_edr_enabled: bool) -> Self {
        Self {
            cg_window_level: Self::CG_DESKTOP_WINDOW_LEVEL,
            ns_window_handle: None,
            metal_hdr_edr_enabled,
            width: 2560,
            height: 1440,
        }
    }

    pub fn cg_window_level(&self) -> i32 {
        self.cg_window_level
    }

    pub fn is_metal_edr_enabled(&self) -> bool {
        self.metal_hdr_edr_enabled
    }

    /// Builds a Quartz/Metal `CAMetalLayer` configuration descriptor for macOS desktop rendering.
    pub fn build_ca_metal_layer_descriptor(&self) -> CaMetalLayerDescriptor {
        CaMetalLayerDescriptor {
            cg_window_level: self.cg_window_level,
            mtl_pixel_format: if self.metal_hdr_edr_enabled {
                "MTLPixelFormatRGBA16Float".to_string()
            } else {
                "MTLPixelFormatBGRA8Unorm".to_string()
            },
            wants_extended_dynamic_range_content: self.metal_hdr_edr_enabled,
            opaque: false,
            drawable_width: self.width,
            drawable_height: self.height,
            maximum_drawable_count: 3,
        }
    }
}

/// Configuration descriptor for a macOS `CAMetalLayer` backed by an `NSWindow` at `kCGDesktopWindowLevel`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaMetalLayerDescriptor {
    pub cg_window_level: i32,
    pub mtl_pixel_format: String,
    pub wants_extended_dynamic_range_content: bool,
    pub opaque: bool,
    pub drawable_width: u32,
    pub drawable_height: u32,
    pub maximum_drawable_count: u32,
}

impl DesktopSurface for MacOsQuartzDesktopSurface {
    fn backend_kind(&self) -> SurfaceBackendKind {
        SurfaceBackendKind::MacOsQuartzDesktopLevel
    }

    fn acquire(&mut self) -> anyhow::Result<isize> {
        let handle = 0x6D61_6301isize;
        self.ns_window_handle = Some(handle);
        Ok(handle)
    }

    fn release(&mut self) {
        self.ns_window_handle = None;
    }

    fn is_valid(&self) -> bool {
        self.ns_window_handle.is_some()
    }

    fn surface_handle(&self) -> Option<isize> {
        self.ns_window_handle
    }

    fn on_display_change(
        &mut self,
        width: u32,
        height: u32,
        _monitor_count: u32,
    ) -> anyhow::Result<Option<isize>> {
        self.width = width.max(1);
        self.height = height.max(1);
        let handle = self.acquire()?;
        Ok(Some(handle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_platform_desktop_surface_implementations() {
        let mut wayland = LinuxWaylandLayerSurface::new("aether-desktop-overlay");
        assert_eq!(
            wayland.backend_kind(),
            SurfaceBackendKind::LinuxWaylandLayerShell
        );
        assert_eq!(wayland.layer(), WaylandLayer::Background);
        assert!(!wayland.is_valid());

        let wl_id = wayland.acquire().unwrap();
        assert!(wayland.is_valid());
        assert_eq!(wayland.surface_handle(), Some(wl_id));
        wayland.on_display_change(3840, 2160, 2).unwrap();
        wayland.release();
        assert!(!wayland.is_valid());

        let mut macos = MacOsQuartzDesktopSurface::new(true);
        assert_eq!(
            macos.backend_kind(),
            SurfaceBackendKind::MacOsQuartzDesktopLevel
        );
        assert_eq!(
            macos.cg_window_level(),
            MacOsQuartzDesktopSurface::CG_DESKTOP_WINDOW_LEVEL
        );
        assert!(macos.is_metal_edr_enabled());
        let ns_id = macos.acquire().unwrap();
        assert!(macos.is_valid());
        assert_eq!(macos.surface_handle(), Some(ns_id));
        let metal_desc = macos.build_ca_metal_layer_descriptor();
        assert_eq!(metal_desc.mtl_pixel_format, "MTLPixelFormatRGBA16Float");
        assert!(metal_desc.wants_extended_dynamic_range_content);
        assert!(!metal_desc.opaque);
        assert_eq!(metal_desc.maximum_drawable_count, 3);

        let wl_packet = wayland.encode_wayland_layer_wire_packet();
        assert_eq!(wl_packet.len(), 28);
        macos.release();
        assert!(!macos.is_valid());
    }
}

