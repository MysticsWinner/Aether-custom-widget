use system_providers::TelemetrySnapshot;

/// Formats telemetry metrics into standard Prometheus text exposition format.
pub struct PrometheusExporter;

impl PrometheusExporter {
    pub fn format_snapshot(snapshot: &TelemetrySnapshot, active_widgets_count: usize) -> String {
        let mut out = String::new();

        out.push_str("# HELP aether_cpu_usage_percent CPU utilization percentage\n");
        out.push_str("# TYPE aether_cpu_usage_percent gauge\n");
        out.push_str(&format!("aether_cpu_usage_percent {:.2}\n\n", snapshot.cpu_usage_pct));

        out.push_str("# HELP aether_gpu_usage_percent GPU utilization percentage\n");
        out.push_str("# TYPE aether_gpu_usage_percent gauge\n");
        out.push_str(&format!("aether_gpu_usage_percent {:.2}\n\n", snapshot.gpu_usage_pct));

        out.push_str("# HELP aether_memory_used_mb Memory used in megabytes\n");
        out.push_str("# TYPE aether_memory_used_mb gauge\n");
        out.push_str(&format!("aether_memory_used_mb {:.2}\n\n", snapshot.memory_used_mb));

        out.push_str("# HELP aether_active_widgets_count Number of active widgets\n");
        out.push_str("# TYPE aether_active_widgets_count gauge\n");
        out.push_str(&format!("aether_active_widgets_count {}\n", active_widgets_count));

        out
    }
}
