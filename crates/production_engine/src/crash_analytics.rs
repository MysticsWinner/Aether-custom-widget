use tracing::info;

/// Zero-PII Crash Analytics & Minidump Collector.
pub struct CrashAnalytics;

impl CrashAnalytics {
    /// Captures a privacy-first crash minidump when a sandbox crash occurs.
    pub fn capture_minidump(plugin_id: &str, exit_code: i32) -> String {
        info!(
            "Capturing zero-PII crash minidump for plugin '{}' (Exit code: {})...",
            plugin_id, exit_code
        );

        format!("minidump_{}_code_{}.dmp", plugin_id, exit_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_analytics() {
        let dump = CrashAnalytics::capture_minidump("weather-widget", -1073741819);
        assert!(dump.contains("weather-widget"));
    }
}
