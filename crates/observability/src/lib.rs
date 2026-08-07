pub mod correlation;
pub mod etw;
pub mod minidump;
pub mod prometheus;

pub use correlation::TraceContext;
pub use etw::{EtwEvent, EtwProvider};
pub use minidump::MinidumpWriter;
pub use prometheus::PrometheusExporter;

#[cfg(test)]
mod tests {
    use super::*;
    use system_providers::TelemetrySnapshot;
    use tempfile::tempdir;

    #[test]
    fn test_trace_context_span_propagation() {
        let root = TraceContext::new_root();
        assert!(root.parent_span_id.is_none());

        let child = root.new_child();
        assert_eq!(child.trace_id, root.trace_id);
        assert_eq!(child.parent_span_id, Some(root.span_id.clone()));
        assert_ne!(child.span_id, root.span_id);
    }

    #[test]
    fn test_prometheus_exporter_formatting() {
        let snap = TelemetrySnapshot {
            timestamp_ms: 1000,
            cpu_usage_pct: 12.5,
            gpu_usage_pct: 34.0,
            memory_used_mb: 512.0,
            memory_total_mb: 16384.0,
            net_recv_bytes_per_sec: 1024,
            net_sent_bytes_per_sec: 2048,
            custom_metrics: std::collections::HashMap::new(),
            ..TelemetrySnapshot::default()
        };

        let metrics = PrometheusExporter::format_snapshot(&snap, 3);
        assert!(metrics.contains("aether_cpu_usage_percent 12.50"));
        assert!(metrics.contains("aether_memory_used_mb 512.00"));
        assert!(metrics.contains("aether_active_widgets_count 3"));
    }

    #[test]
    fn test_minidump_writer_creates_dump_file() {
        let dir = tempdir().unwrap();
        let writer = MinidumpWriter::new(dir.path());

        let dump_path = writer.create_minidump("Panic test").unwrap();
        assert!(dump_path.exists());

        let dumps = writer.list_minidumps().unwrap();
        assert_eq!(dumps.len(), 1);
    }

    #[test]
    fn test_etw_provider_event_formatting() {
        let provider = EtwProvider::new("AetherEngine");
        let event = provider.write_event("RenderFrame", "{\"fps\":60}");

        assert_eq!(event.provider_name, "AetherEngine");
        assert_eq!(event.event_name, "RenderFrame");
        assert_eq!(event.payload_json, "{\"fps\":60}");
    }
}
