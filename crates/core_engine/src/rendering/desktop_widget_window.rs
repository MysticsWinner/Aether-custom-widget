use layout_engine::WidgetPositionStore;
use system_providers::SharedTelemetryCache;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

/// Manages the native transparent Windows desktop overlay widget window.
#[derive(Clone)]
pub struct DesktopWidgetWindow {
    visible: Arc<AtomicBool>,
    position_store: WidgetPositionStore,
}

impl DesktopWidgetWindow {
    pub fn new() -> Self {
        Self {
            visible: Arc::new(AtomicBool::new(true)),
            position_store: WidgetPositionStore::default(),
        }
    }

    pub fn with_position_store(position_store: WidgetPositionStore) -> Self {
        Self {
            visible: Arc::new(AtomicBool::new(true)),
            position_store,
        }
    }

    pub fn position_store(&self) -> &WidgetPositionStore {
        &self.position_store
    }

    pub fn is_visible(&self) -> bool {
        self.visible.load(Ordering::Relaxed)
    }

    pub fn toggle_visibility(&self) -> bool {
        let prev = self.visible.fetch_xor(true, Ordering::Relaxed);
        let new_state = !prev;
        info!("Desktop Widget Window visibility toggled: {}", new_state);
        new_state
    }

    pub fn set_position(&self, widget_id: &str, x: i32, y: i32) -> anyhow::Result<()> {
        self.position_store.set_position(widget_id, x, y)
    }

    pub fn is_locked(&self, widget_id: &str) -> bool {
        self.position_store.is_locked(widget_id)
    }

    pub fn set_locked(&self, widget_id: &str, locked: bool) -> anyhow::Result<()> {
        self.position_store.set_locked(widget_id, locked)
    }

    pub fn toggle_locked(&self, widget_id: &str) -> bool {
        self.position_store.toggle_locked(widget_id)
    }

    /// Spawns the transparent desktop overlay window bound to shared telemetry cache.
    pub fn spawn_overlay(&self, cache: SharedTelemetryCache) {
        let visible_flag = self.visible.clone();
        let pos_store = self.position_store.clone();
        tokio::task::spawn_blocking(move || {
            #[cfg(windows)]
            {
                if let Err(e) = run_desktop_window_loop(cache, visible_flag, pos_store) {
                    tracing::error!("Desktop widget window loop exited: {:?}", e);
                }
            }
            #[cfg(not(windows))]
            {
                let _ = (cache, visible_flag, pos_store);
            }
        });
    }
}

impl Default for DesktopWidgetWindow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(windows)]
fn run_desktop_window_loop(
    cache: SharedTelemetryCache,
    visible: Arc<AtomicBool>,
    pos_store: WidgetPositionStore,
) -> anyhow::Result<()> {
    use windows::core::w;
    use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM};
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateFontW, CreateSolidBrush, DeleteDC, DeleteObject, FillRect, GetDC,
        ReleaseDC, SelectObject, SetBkMode, SetTextColor, FW_BOLD, FW_SEMIBOLD, TRANSPARENT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowRect, RegisterClassW,
        ShowWindow, UpdateLayeredWindow, CS_HREDRAW, CS_VREDRAW, HMENU, MSG,
        SW_HIDE, SW_SHOW, ULW_ALPHA, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WM_NCHITTEST, WM_EXITSIZEMOVE, HTCAPTION,
    };

    static mut GLOBAL_POS_STORE: Option<WidgetPositionStore> = None;

    unsafe {
        GLOBAL_POS_STORE = Some(pos_store.clone());

        let hinstance: HINSTANCE = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?.into();
        let class_name = w!("AetherDesktopWidgetClass");

        unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
            let pos_store_ref = unsafe { (&raw const GLOBAL_POS_STORE).as_ref() }.and_then(|opt| opt.as_ref());
            match msg {
                WM_NCHITTEST => {
                    if let Some(store) = pos_store_ref {
                        if !store.is_locked("perf_monitor_widget") {
                            return LRESULT(HTCAPTION as isize);
                        }
                    }
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
                WM_EXITSIZEMOVE => {
                    let mut rect = RECT::default();
                    if GetWindowRect(hwnd, &mut rect).is_ok() {
                        if let Some(store) = pos_store_ref {
                            let _ = store.set_position("perf_monitor_widget", rect.left, rect.top);
                            info!(
                                "[DesktopWidgetWindow] Drag position updated and saved: ({}, {})",
                                rect.left, rect.top
                            );
                        }
                    }
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }

        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            lpszClassName: class_name,
            ..Default::default()
        };

        RegisterClassW(&wc);

        let width = 340;
        let height = 250;

        // Position: Load custom (x, y) if user moved widget previously, else default to upper-right screen corner
        let (x, y) = if let Some((saved_x, saved_y)) = pos_store.get_position("perf_monitor_widget") {
            (saved_x, saved_y)
        } else {
            let screen_width = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
                windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN,
            );
            let def_x = if screen_width > width + 40 { screen_width - width - 30 } else { 30 };
            let def_y = 60;
            (def_x, def_y)
        };

        let hwnd = CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name,
            w!("Aether Desktop Performance Monitor"),
            WS_POPUP,
            x,
            y,
            width,
            height,
            HWND(std::ptr::null_mut()),
            HMENU(std::ptr::null_mut()),
            hinstance,
            None,
        )?;

        // Attach to WorkerW desktop wallpaper layer if available
        if let Some(workerw_hwnd) = crate::rendering::workerw::find_desktop_workerw_hwnd() {
            let _ = windows::Win32::UI::WindowsAndMessaging::SetParent(hwnd, workerw_hwnd);
        }

        let _ = ShowWindow(hwnd, SW_SHOW);

        let mut msg = MSG::default();
        let mut last_visible = true;

        loop {
            // Check visibility toggle state
            let cur_visible = visible.load(Ordering::Relaxed);
            if cur_visible != last_visible {
                last_visible = cur_visible;
                if cur_visible {
                    let _ = ShowWindow(hwnd, SW_SHOW);
                } else {
                    let _ = ShowWindow(hwnd, SW_HIDE);
                }
            }

            if cur_visible {
                let mut current_rect = RECT::default();
                let (cur_x, cur_y) = if GetWindowRect(hwnd, &mut current_rect).is_ok() {
                    (current_rect.left, current_rect.top)
                } else {
                    (x, y)
                };

                // Render GDI / Layered Window Card
                let snap = cache.get_snapshot();
                let screen_dc = GetDC(hwnd);
                let mem_dc = CreateCompatibleDC(screen_dc);

                use windows::Win32::Graphics::Gdi::{BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CreateDIBSection, DIB_RGB_COLORS};
                let mut bmi = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: width,
                        biHeight: -height, // top-down DIB
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB.0,
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
                let hbmp = CreateDIBSection(screen_dc, &mut bmi, DIB_RGB_COLORS, &mut bits, None, 0)?;
                let old_bmp = SelectObject(mem_dc, hbmp);

                // Draw Glassmorphism Dark Card Background (#0F172A with alpha)
                let pixel_slice = std::slice::from_raw_parts_mut(bits as *mut u32, (width * height) as usize);
                for p in pixel_slice.iter_mut() {
                    *p = 0xD80F172A; 
                }

                // Draw title and telemetry gauges
                SetBkMode(mem_dc, TRANSPARENT);
                let hfont = CreateFontW(14, 0, 0, 0, FW_BOLD.0 as i32, 0, 0, 0, 0, 0, 0, 0, 0, w!("Segoe UI"));
                let old_font = SelectObject(mem_dc, hfont);

                SetTextColor(mem_dc, COLORREF(0x0000D4F5)); // Cyan title
                let lock_indicator = if pos_store.is_locked("perf_monitor_widget") { " [LOCKED]" } else { " [DRAG TO MOVE]" };
                let title = format!("AETHER PERFORMANCE MONITOR{}", lock_indicator);
                let mut r_title = RECT { left: 16, top: 12, right: width - 16, bottom: 32 };
                windows::Win32::Graphics::Gdi::DrawTextW(mem_dc, &mut title.encode_utf16().collect::<Vec<u16>>(), &mut r_title, windows::Win32::Graphics::Gdi::DT_SINGLELINE);

                // CPU Line
                let sub_font = CreateFontW(12, 0, 0, 0, FW_SEMIBOLD.0 as i32, 0, 0, 0, 0, 0, 0, 0, 0, w!("Segoe UI"));
                SelectObject(mem_dc, sub_font);

                SetTextColor(mem_dc, COLORREF(0x00FFFFFF));
                let cpu_str = format!("CPU Utilization: {:.1}%", snap.cpu_usage_pct);
                let mut r_cpu = RECT { left: 16, top: 38, right: width - 16, bottom: 56 };
                windows::Win32::Graphics::Gdi::DrawTextW(mem_dc, &mut cpu_str.encode_utf16().collect::<Vec<u16>>(), &mut r_cpu, windows::Win32::Graphics::Gdi::DT_SINGLELINE);

                // CPU Bar
                let cpu_bar_width = ((width - 32) as f32 * (snap.cpu_usage_pct / 100.0)).clamp(0.0, (width - 32) as f32) as i32;
                let brush_cpu = CreateSolidBrush(COLORREF(0x0000D4F5));
                let rect_cpu_bar = RECT { left: 16, top: 58, right: 16 + cpu_bar_width, bottom: 64 };
                FillRect(mem_dc, &rect_cpu_bar, brush_cpu);
                let _ = DeleteObject(brush_cpu);

                // GPU Line
                let gpu_str = format!("GPU Utilization: {:.1}%", snap.gpu_usage_pct);
                let mut r_gpu = RECT { left: 16, top: 72, right: width - 16, bottom: 90 };
                windows::Win32::Graphics::Gdi::DrawTextW(mem_dc, &mut gpu_str.encode_utf16().collect::<Vec<u16>>(), &mut r_gpu, windows::Win32::Graphics::Gdi::DT_SINGLELINE);

                // GPU Bar
                let gpu_bar_width = ((width - 32) as f32 * (snap.gpu_usage_pct / 100.0)).clamp(0.0, (width - 32) as f32) as i32;
                let brush_gpu = CreateSolidBrush(COLORREF(0x00EC4899));
                let rect_gpu_bar = RECT { left: 16, top: 92, right: 16 + gpu_bar_width, bottom: 98 };
                FillRect(mem_dc, &rect_gpu_bar, brush_gpu);
                let _ = DeleteObject(brush_gpu);

                // RAM Line
                let ram_used_gb = snap.memory_used_mb / 1024.0;
                let ram_total_gb = snap.memory_total_mb / 1024.0;
                let ram_pct = if snap.memory_total_mb > 0.0 { (snap.memory_used_mb / snap.memory_total_mb) * 100.0 } else { 0.0 };
                let ram_str = format!("RAM Memory: {:.1} / {:.1} GB ({:.0}%)", ram_used_gb, ram_total_gb, ram_pct);
                let mut r_ram = RECT { left: 16, top: 106, right: width - 16, bottom: 124 };
                windows::Win32::Graphics::Gdi::DrawTextW(mem_dc, &mut ram_str.encode_utf16().collect::<Vec<u16>>(), &mut r_ram, windows::Win32::Graphics::Gdi::DT_SINGLELINE);

                // RAM Bar
                let ram_bar_width = ((width - 32) as f32 * (ram_pct / 100.0)).clamp(0.0, (width - 32) as f32) as i32;
                let brush_ram = CreateSolidBrush(COLORREF(0x0010B981));
                let rect_ram_bar = RECT { left: 16, top: 126, right: 16 + ram_bar_width, bottom: 132 };
                FillRect(mem_dc, &rect_ram_bar, brush_ram);
                let _ = DeleteObject(brush_ram);

                // Network Line
                let net_str = format!("Network Throughput: {:.1} KB/s", snap.net_recv_bytes_per_sec as f32 / 1024.0);
                let mut r_net = RECT { left: 16, top: 140, right: width - 16, bottom: 158 };
                windows::Win32::Graphics::Gdi::DrawTextW(mem_dc, &mut net_str.encode_utf16().collect::<Vec<u16>>(), &mut r_net, windows::Win32::Graphics::Gdi::DT_SINGLELINE);

                // Status footer
                SetTextColor(mem_dc, COLORREF(0x0094A3B8));
                let footer_str = format!("Aether Engine v0.5.0 • Position ({}, {})", cur_x, cur_y);
                let mut r_footer = RECT { left: 16, top: 168, right: width - 16, bottom: 186 };
                windows::Win32::Graphics::Gdi::DrawTextW(mem_dc, &mut footer_str.encode_utf16().collect::<Vec<u16>>(), &mut r_footer, windows::Win32::Graphics::Gdi::DT_SINGLELINE);

                // Present layered transparent window
                let mut pt_dst = POINT { x: cur_x, y: cur_y };
                let mut size_dst = SIZE { cx: width, cy: height };
                let mut pt_src = POINT { x: 0, y: 0 };
                let mut blend = windows::Win32::Graphics::Gdi::BLENDFUNCTION {
                    BlendOp: windows::Win32::Graphics::Gdi::AC_SRC_OVER as u8,
                    BlendFlags: 0,
                    SourceConstantAlpha: 255,
                    AlphaFormat: windows::Win32::Graphics::Gdi::AC_SRC_ALPHA as u8,
                };

                let _ = UpdateLayeredWindow(
                    hwnd,
                    screen_dc,
                    Some(&mut pt_dst),
                    Some(&mut size_dst),
                    mem_dc,
                    Some(&mut pt_src),
                    COLORREF(0),
                    Some(&mut blend),
                    ULW_ALPHA,
                );

                // Clean up GDI objects
                SelectObject(mem_dc, old_bmp);
                SelectObject(mem_dc, old_font);
                let _ = DeleteObject(hbmp);
                let _ = DeleteObject(hfont);
                let _ = DeleteObject(sub_font);
                let _ = DeleteDC(mem_dc);
                ReleaseDC(hwnd, screen_dc);
            }

            // Pump window messages or sleep 50 ms
            if GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0).as_bool() {
                DispatchMessageW(&msg);
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_widget_window_toggle_and_position() {
        let store = WidgetPositionStore::in_memory();
        let window = DesktopWidgetWindow::with_position_store(store.clone());
        assert!(window.is_visible());
        let new_state = window.toggle_visibility();
        assert!(!new_state);
        assert!(!window.is_visible());

        assert_eq!(window.position_store().get_position("perf_monitor_widget"), None);
        window.set_position("perf_monitor_widget", 300, 150).unwrap();
        assert_eq!(window.position_store().get_position("perf_monitor_widget"), Some((300, 150)));

        assert!(!window.is_locked("perf_monitor_widget"));
        window.set_locked("perf_monitor_widget", true).unwrap();
        assert!(window.is_locked("perf_monitor_widget"));
        assert!(!window.toggle_locked("perf_monitor_widget"));
        assert!(!window.is_locked("perf_monitor_widget"));
    }
}

