use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use system_providers::{SharedTelemetryCache, TelemetrySnapshot};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{info, warn};

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

        out.push_str("# HELP aether_memory_total_mb Total physical memory in megabytes\n");
        out.push_str("# TYPE aether_memory_total_mb gauge\n");
        out.push_str(&format!("aether_memory_total_mb {:.2}\n\n", snapshot.memory_total_mb));

        out.push_str("# HELP aether_net_recv_bytes_per_sec Inbound network bandwidth in bytes/sec\n");
        out.push_str("# TYPE aether_net_recv_bytes_per_sec gauge\n");
        out.push_str(&format!("aether_net_recv_bytes_per_sec {}\n\n", snapshot.net_recv_bytes_per_sec));

        out.push_str("# HELP aether_net_sent_bytes_per_sec Outbound network bandwidth in bytes/sec\n");
        out.push_str("# TYPE aether_net_sent_bytes_per_sec gauge\n");
        out.push_str(&format!("aether_net_sent_bytes_per_sec {}\n\n", snapshot.net_sent_bytes_per_sec));

        out.push_str("# HELP aether_battery_charge_percent Battery charge percentage\n");
        out.push_str("# TYPE aether_battery_charge_percent gauge\n");
        out.push_str(&format!("aether_battery_charge_percent {:.2}\n\n", snapshot.battery_charge_pct));

        out.push_str("# HELP aether_active_widgets_count Number of active widgets\n");
        out.push_str("# TYPE aether_active_widgets_count gauge\n");
        out.push_str(&format!("aether_active_widgets_count {}\n", active_widgets_count));

        out
    }
}

/// Embedded HTTP server exposing `/metrics` endpoint for Prometheus scraping.
pub struct PrometheusHttpServer {
    cache: SharedTelemetryCache,
    active_widgets_count: Arc<RwLock<usize>>,
    bind_addr: SocketAddr,
}

impl PrometheusHttpServer {
    pub fn new(cache: SharedTelemetryCache, bind_addr: SocketAddr) -> Self {
        Self {
            cache,
            active_widgets_count: Arc::new(RwLock::new(1)),
            bind_addr,
        }
    }

    pub fn set_active_widgets_count(&self, count: usize) {
        let count_ref = self.active_widgets_count.clone();
        tokio::spawn(async move {
            let mut lock = count_ref.write().await;
            *lock = count;
        });
    }

    /// Spawns the Prometheus HTTP exposition server listener.
    pub async fn start(self) -> Result<()> {
        let listener = TcpListener::bind(self.bind_addr).await?;
        info!("Prometheus Metrics Exporter HTTP server listening on http://{}/metrics", self.bind_addr);

        let cache = self.cache.clone();
        let widgets_count = self.active_widgets_count.clone();

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((mut socket, _peer_addr)) => {
                        let cache_ref = cache.clone();
                        let count_ref = widgets_count.clone();

                        tokio::spawn(async move {
                            let mut buf = [0u8; 1024];
                            if let Ok(n) = socket.read(&mut buf).await {
                                let request = String::from_utf8_lossy(&buf[..n]);
                                let active_count = *count_ref.read().await;

                                let (status, body) = if request.starts_with("GET /metrics") || request.starts_with("GET / ") {
                                    let snap = cache_ref.get_snapshot();
                                    let metrics = PrometheusExporter::format_snapshot(&snap, active_count);
                                    ("200 OK", metrics)
                                } else {
                                    ("404 Not Found", "Use GET /metrics to scrape metrics\n".to_string())
                                };

                                let response = format!(
                                    "HTTP/1.1 {}\r\nContent-Type: text/plain; version=0.0.4; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                    status,
                                    body.len(),
                                    body
                                );

                                let _ = socket.write_all(response.as_bytes()).await;
                            }
                        });
                    }
                    Err(e) => {
                        warn!("Prometheus TCP accept error: {e}");
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    }
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_prometheus_exporter_formatting() {
        let snap = TelemetrySnapshot {
            timestamp_ms: 1000,
            cpu_usage_pct: 12.5,
            memory_used_mb: 2048.0,
            memory_total_mb: 16384.0,
            gpu_usage_pct: 33.0,
            net_recv_bytes_per_sec: 10000,
            net_sent_bytes_per_sec: 2500,
            open_apps_count: 5,
            browser_tabs_count: 10,
            audio_playing_apps_count: 1,
            gaming_apps_count: 0,
            dev_suite_apps_count: 2,
            other_apps_count: 2,
            master_volume_pct: 80.0,
            is_muted: false,
            battery_charge_pct: 95.0,
            battery_remaining_secs: 7200,
            is_charging: true,
            total_gpu_count: 1,
            integrated_gpu_count: 1,
            dedicated_gpu_count: 0,
            total_display_count: 1,
            external_display_count: 0,
            virtual_display_count: 0,
            custom_metrics: Default::default(),
        };

        let metrics = PrometheusExporter::format_snapshot(&snap, 3);
        assert!(metrics.contains("aether_cpu_usage_percent 12.50"));
        assert!(metrics.contains("aether_memory_used_mb 2048.00"));
        assert!(metrics.contains("aether_gpu_usage_percent 33.00"));
        assert!(metrics.contains("aether_active_widgets_count 3"));
    }

    #[tokio::test]
    async fn test_prometheus_http_server_metrics_endpoint() {
        let cache = SharedTelemetryCache::new();
        // Bind to localhost on random free port
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = TcpListener::bind(addr).await.unwrap();
        let local_addr = listener.local_addr().unwrap();
        drop(listener);

        let server = PrometheusHttpServer::new(cache, local_addr);
        assert!(server.start().await.is_ok());

        // Perform HTTP GET /metrics request
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let mut stream = tokio::net::TcpStream::connect(local_addr).await.unwrap();
        stream.write_all(b"GET /metrics HTTP/1.1\r\nHost: localhost\r\n\r\n").await.unwrap();

        let mut buf = vec![0u8; 2048];
        let n = stream.read(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf[..n]);

        assert!(resp.contains("HTTP/1.1 200 OK"));
        assert!(resp.contains("aether_cpu_usage_percent"));
        assert!(resp.contains("Content-Type: text/plain"));
    }
}
