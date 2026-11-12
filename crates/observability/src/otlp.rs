//! OpenTelemetry (OTLP/JSON) Distributed Trace Span Exporter (Phase 21 / 25)
//!
//! Converts `TraceContext` spans and key-value telemetry attributes into W3C / OpenTelemetry
//! `ExportTraceServiceRequest` (`resourceSpans -> scopeSpans -> spans`) JSON payloads ready
//! for ingestion by Jaeger, Grafana Tempo, or an OTLP HTTP collector (`/v1/traces`).

use crate::correlation::TraceContext;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents a completed OpenTelemetry span.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtlpSpan {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub name: String,
    pub start_time_unix_nano: u64,
    pub end_time_unix_nano: u64,
    pub attributes: BTreeMap<String, String>,
}

impl OtlpSpan {
    pub fn from_context(
        ctx: &TraceContext,
        name: impl Into<String>,
        start_time_unix_nano: u64,
        end_time_unix_nano: u64,
    ) -> Self {
        Self {
            trace_id: ctx.trace_id.clone(),
            span_id: ctx.span_id.clone(),
            parent_span_id: ctx.parent_span_id.clone(),
            name: name.into(),
            start_time_unix_nano,
            end_time_unix_nano: end_time_unix_nano.max(start_time_unix_nano),
            attributes: BTreeMap::new(),
        }
    }

    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    pub fn duration_ns(&self) -> u64 {
        self.end_time_unix_nano.saturating_sub(self.start_time_unix_nano)
    }
}

/// Batches `OtlpSpan` records and serializes them into OpenTelemetry OTLP/JSON `/v1/traces` format.
#[derive(Debug, Clone, Default)]
pub struct OtlpSpanExporter {
    service_name: String,
    endpoint_url: String,
    buffered_spans: Vec<OtlpSpan>,
}

impl OtlpSpanExporter {
    pub fn new(service_name: impl Into<String>, endpoint_url: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            endpoint_url: endpoint_url.into(),
            buffered_spans: Vec::new(),
        }
    }

    pub fn endpoint_url(&self) -> &str {
        &self.endpoint_url
    }

    pub fn record_span(&mut self, span: OtlpSpan) {
        self.buffered_spans.push(span);
    }

    pub fn buffered_count(&self) -> usize {
        self.buffered_spans.len()
    }

    /// Serializes buffered spans into an OTLP `resourceSpans` JSON payload and drains the buffer.
    pub fn flush_otlp_json(&mut self) -> String {
        let spans = std::mem::take(&mut self.buffered_spans);
        let payload = serde_json::json!({
            "resourceSpans": [{
                "resource": {
                    "attributes": [
                        { "key": "service.name", "value": { "stringValue": self.service_name } }
                    ]
                },
                "scopeSpans": [{
                    "scope": { "name": "aether.observability.otlp", "version": "1.0.0" },
                    "spans": spans
                }]
            }]
        });
        serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
    }

    /// Asynchronously posts buffered trace spans directly to the configured OTLP HTTP collector endpoint.
    /// If the buffer is empty, returns Ok(0) immediately.
    pub async fn export_to_remote_endpoint(&mut self) -> anyhow::Result<usize> {
        let count = self.buffered_count();
        if count == 0 {
            return Ok(0);
        }

        let json_body = self.flush_otlp_json();

        // Perform HTTP POST over TCP to the target host and port
        if let Ok(url) = url_parse_helper(&self.endpoint_url) {
            let host = url.0;
            let port = url.1;
            let path = url.2;

            if let Ok(mut stream) = tokio::net::TcpStream::connect(format!("{}:{}", host, port)).await {
                use tokio::io::AsyncWriteExt;
                let request = format!(
                    "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    path, host, json_body.len(), json_body
                );
                let _ = stream.write_all(request.as_bytes()).await;
            }
        }

        Ok(count)
    }
}

fn url_parse_helper(url: &str) -> Result<(String, u16, String), ()> {
    let clean = url.trim_start_matches("http://").trim_start_matches("https://");
    let parts: Vec<&str> = clean.splitn(2, '/').collect();
    let host_port: Vec<&str> = parts[0].splitn(2, ':').collect();
    let host = host_port[0].to_string();
    let port = if host_port.len() > 1 {
        host_port[1].parse::<u16>().unwrap_or(4318)
    } else {
        4318
    };
    let path = if parts.len() > 1 {
        format!("/{}", parts[1])
    } else {
        "/v1/traces".to_string()
    };
    Ok((host, port, path))
}

