//! Perf-card renderer — produces a `BatchRenderCanvas` draw-command list
//! representing a dark glassmorphism performance overlay (340 × 250 px).

use system_providers::TelemetrySnapshot;
use widget_sdk::rendering::{BatchRenderCanvas, Color, RectF, RenderCanvas};

// ── palette ───────────────────────────────────────────────────────────────────
const BG: Color          = Color { r: 0.07, g: 0.08, b: 0.11, a: 0.93 };
const SEPARATOR: Color   = Color { r: 0.28, g: 0.32, b: 0.42, a: 0.55 };
const TITLE: Color       = Color { r: 0.90, g: 0.94, b: 1.00, a: 1.00 };
const LABEL: Color       = Color { r: 0.72, g: 0.78, b: 0.90, a: 1.00 };
const TRACK: Color       = Color { r: 0.14, g: 0.17, b: 0.23, a: 1.00 };
const FOOTER: Color      = Color { r: 0.38, g: 0.43, b: 0.54, a: 1.00 };
const CYAN: Color        = Color { r: 0.00, g: 0.83, b: 1.00, a: 1.00 }; // CPU
const MAGENTA: Color     = Color { r: 0.73, g: 0.18, b: 1.00, a: 1.00 }; // GPU
const GREEN: Color       = Color { r: 0.08, g: 0.88, b: 0.48, a: 1.00 }; // RAM
const YELLOW: Color      = Color { r: 1.00, g: 0.78, b: 0.18, a: 1.00 }; // NET

// ── dimensions ────────────────────────────────────────────────────────────────
const CARD_X: f32 = 20.0;
const CARD_Y: f32 = 20.0;
const CARD_W: f32 = 340.0;
const CARD_H: f32 = 250.0;
const PAD: f32    = 16.0;
const BAR_H: f32  = 10.0;
const FONT: &str  = "Segoe UI Variable";

/// Builds the complete draw-command list for the performance overlay card.
///
/// The caller owns the `BatchRenderCanvas`; the render host submits it to the
/// DirectComposition pipeline once per frame.
pub fn render_perf_card(canvas: &mut BatchRenderCanvas, snap: &TelemetrySnapshot) {
    let inner_w = CARD_W - PAD * 2.0;

    // ── background card ──────────────────────────────────────────────────────
    canvas.draw_rect(RectF::new(CARD_X, CARD_Y, CARD_W, CARD_H), BG, 12.0);

    // ── title ────────────────────────────────────────────────────────────────
    canvas.draw_text(
        "\u{26A1} Aether Performance Monitor",
        FONT, 13.0,
        RectF::new(CARD_X + PAD, CARD_Y + PAD, inner_w, 18.0),
        TITLE,
    );

    // ── separator ────────────────────────────────────────────────────────────
    canvas.draw_rect(
        RectF::new(CARD_X + PAD, CARD_Y + 36.0, inner_w, 1.0),
        SEPARATOR, 0.0,
    );

    // ── CPU row ──────────────────────────────────────────────────────────────
    let cpu_label = format!("CPU  {:5.1}%", snap.cpu_usage_pct);
    metric_row(
        canvas, CARD_X + PAD, CARD_Y + 44.0, inner_w,
        &cpu_label, snap.cpu_usage_pct / 100.0, CYAN,
    );

    // ── GPU row ──────────────────────────────────────────────────────────────
    let gpu_label = format!("GPU  {:5.1}%", snap.gpu_usage_pct);
    metric_row(
        canvas, CARD_X + PAD, CARD_Y + 88.0, inner_w,
        &gpu_label, snap.gpu_usage_pct / 100.0, MAGENTA,
    );

    // ── RAM row ──────────────────────────────────────────────────────────────
    let total_gb = snap.memory_total_mb / 1024.0;
    let used_gb  = snap.memory_used_mb  / 1024.0;
    let free_gb  = (total_gb - used_gb).max(0.0);
    let ram_pct  = if snap.memory_total_mb > 0.0 {
        snap.memory_used_mb / snap.memory_total_mb
    } else {
        0.0
    };
    let ram_label = format!(
        "RAM  {used:.2}/{total:.2} GB  ({pct:.0}% used \u{2022} {free:.2} GB free)",
        used = used_gb, total = total_gb,
        pct = ram_pct * 100.0, free = free_gb,
    );
    metric_row(
        canvas, CARD_X + PAD, CARD_Y + 132.0, inner_w,
        &ram_label, ram_pct, GREEN,
    );

    // ── NET row ──────────────────────────────────────────────────────────────
    let net_kbps = snap.net_recv_bytes_per_sec as f32 / 1024.0;
    let net_mbps = net_kbps / 1024.0;
    let net_text = if net_mbps >= 1.0 {
        format!("NET  {:.2} MB/s", net_mbps)
    } else {
        format!("NET  {:.1} KB/s", net_kbps)
    };
    let net_fill = (net_kbps / 10240.0).clamp(0.0, 1.0); // 10 MB/s full scale reference
    metric_row(
        canvas, CARD_X + PAD, CARD_Y + 176.0, inner_w,
        &net_text, net_fill, YELLOW,
    );

    // ── footer ───────────────────────────────────────────────────────────────
    canvas.draw_text(
        "Aether v0.6.0  \u{2022}  Phase 16 RC  \u{2022}  DirectComposition",
        "Segoe UI", 9.5,
        RectF::new(CARD_X + PAD, CARD_Y + CARD_H - 20.0, inner_w, 13.0),
        FOOTER,
    );
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Renders a single metric row: label line + progress bar track + fill.
fn metric_row(
    canvas: &mut BatchRenderCanvas,
    x: f32, y: f32, width: f32,
    label: &str, fill_ratio: f32, bar_color: Color,
) {
    // label
    canvas.draw_text(label, FONT, 11.5, RectF::new(x, y, width, 14.0), LABEL);

    let bar_y = y + 18.0;
    // track
    canvas.draw_rect(RectF::new(x, bar_y, width, BAR_H), TRACK, 5.0);
    // fill — minimum 6 px so the bar is always visible
    let fill_w = (width * fill_ratio.clamp(0.0, 1.0)).max(6.0);
    canvas.draw_rect(RectF::new(x, bar_y, fill_w, BAR_H), bar_color, 5.0);
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use system_providers::TelemetrySnapshot;

    fn snap(cpu: f32, gpu: f32, used: f32, total: f32) -> TelemetrySnapshot {
        TelemetrySnapshot {
            timestamp_ms: 0, cpu_usage_pct: cpu, gpu_usage_pct: gpu,
            memory_used_mb: used, memory_total_mb: total,
            net_recv_bytes_per_sec: 204800, net_sent_bytes_per_sec: 51200,
            custom_metrics: HashMap::new(),
        }
    }

    #[test]
    fn test_full_load_renders_correctly() {
        let mut canvas = BatchRenderCanvas::new();
        render_perf_card(&mut canvas, &snap(100.0, 100.0, 16384.0, 16384.0));
        // Each row = label + track + fill = 3 cmds; 4 rows = 12 cmds; + bg + separator + title + footer = 16
        assert!(canvas.commands().len() >= 16);
    }

    #[test]
    fn test_zero_metrics_no_panic() {
        let mut canvas = BatchRenderCanvas::new();
        // memory_total_mb = 0 should NOT divide-by-zero
        render_perf_card(&mut canvas, &snap(0.0, 0.0, 0.0, 0.0));
        assert!(!canvas.commands().is_empty());
    }

    #[test]
    fn test_bar_fill_clamped() {
        // Overflow values (>100%) must not crash or produce negative widths
        let mut canvas = BatchRenderCanvas::new();
        render_perf_card(&mut canvas, &snap(120.0, 200.0, 20000.0, 16384.0));
        assert!(!canvas.commands().is_empty());
    }
}

