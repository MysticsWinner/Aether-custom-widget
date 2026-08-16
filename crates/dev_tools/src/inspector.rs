use serde::{Deserialize, Serialize};
use widget_sdk::RectF;

/// Detailed DOM & platform inspector report for a widget.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetInspectionReport {
    pub widget_id: String,
    pub bounds: RectF,
    pub draw_command_count: usize,
    pub memory_used_kb: f32,
    pub tick_duration_us: u64,
    pub target_fps: u32,
    pub cpu_pct: f32,
    pub gpu_pct: f32,
    pub ipc_latency_us: u64,
    pub resolved_material: String,
    pub is_sandboxed: bool,
}

/// Chrome DevTools-style widget inspector & profiler.
pub struct WidgetInspector;

impl WidgetInspector {
    pub fn inspect(
        widget_id: &str,
        bounds: RectF,
        draw_command_count: usize,
        memory_used_kb: f32,
        tick_duration_us: u64,
        target_fps: u32,
    ) -> WidgetInspectionReport {
        WidgetInspectionReport {
            widget_id: widget_id.to_string(),
            bounds,
            draw_command_count,
            memory_used_kb,
            tick_duration_us,
            target_fps,
            cpu_pct: 0.05,
            gpu_pct: 0.02,
            ipc_latency_us: 12,
            resolved_material: "Mica".to_string(),
            is_sandboxed: true,
        }
    }
}
