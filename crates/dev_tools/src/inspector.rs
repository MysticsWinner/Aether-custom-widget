use serde::{Deserialize, Serialize};
use widget_sdk::RectF;

/// Detailed DOM inspector report for a widget.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetInspectionReport {
    pub widget_id: String,
    pub bounds: RectF,
    pub draw_command_count: usize,
    pub memory_used_kb: f32,
    pub tick_duration_us: u64,
    pub target_fps: u32,
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
        }
    }
}
