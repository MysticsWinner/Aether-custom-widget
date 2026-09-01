use std::path::{Path, PathBuf};
use tracing::{error, info};
use serde::{Deserialize, Serialize};

/// Zero-PII Crash Report metadata record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrashReport {
    pub report_id: String,
    pub plugin_id: String,
    pub exit_code: i32,
    pub timestamp_ms: u64,
    pub minidump_path: String,
    pub crash_reason: String,
}

/// Zero-PII Crash Analytics & Minidump Collector.
///
/// Creates structured, zero-PII crash minidumps and JSON diagnostic reports on disk
/// whenever a sandboxed plugin or subsystem process encounters a fatal fault.
pub struct CrashAnalytics {
    output_dir: PathBuf,
}

impl CrashAnalytics {
    pub fn new() -> Self {
        let base_dir = std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir())
            .join("Aether")
            .join("crashes");
        Self { output_dir: base_dir }
    }

    pub fn with_output_dir<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            output_dir: dir.as_ref().to_path_buf(),
        }
    }

    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    /// Captures a privacy-first crash minidump and persists it to disk.
    /// Returns the path string to the generated minidump file.
    pub fn capture_minidump(&self, plugin_id: &str, exit_code: i32) -> String {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if let Err(e) = std::fs::create_dir_all(&self.output_dir) {
            error!("Failed to create crash analytics directory: {e}");
        }

        let report_id = format!("{}_{}_{}", plugin_id.replace(['/', '\\', ':'], "_"), exit_code, now_ms);
        let dmp_filename = format!("minidump_{}.dmp", report_id);
        let dmp_path = self.output_dir.join(&dmp_filename);

        let crash_reason = match exit_code {
            -1073741819 => "STATUS_ACCESS_VIOLATION (0xC0000005)",
            -1073741674 => "STATUS_IN_PAGE_ERROR (0xC0000006)",
            -1073741510 => "STATUS_ILLEGAL_INSTRUCTION (0xC000001D)",
            -1073740791 => "STATUS_STACK_BUFFER_OVERRUN (0xC0000409)",
            1 => "General Process Error (Exit code 1)",
            _code => "Unexpected Process Termination",
        };

        // Write zero-PII structured binary/text minidump header & report
        let minidump_content = format!(
            "AETHER_MINIDUMP_V1\nREPORT_ID={}\nPLUGIN_ID={}\nEXIT_CODE={}\nREASON={}\nTIMESTAMP_MS={}\n",
            report_id, plugin_id, exit_code, crash_reason, now_ms
        );

        if let Err(e) = std::fs::write(&dmp_path, minidump_content.as_bytes()) {
            error!("Failed to write minidump file at {:?}: {e}", dmp_path);
        } else {
            info!(
                "Zero-PII crash minidump captured for '{}' (Exit code: {}) -> {:?}",
                plugin_id, exit_code, dmp_path
            );
        }

        dmp_path.to_string_lossy().into_owned()
    }

    /// Lists all captured minidump files in the crash output directory.
    pub fn list_minidumps(&self) -> Vec<PathBuf> {
        if !self.output_dir.exists() {
            return Vec::new();
        }
        std::fs::read_dir(&self.output_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.extension().map_or(false, |ext| ext == "dmp"))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for CrashAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_analytics_creates_real_file() {
        let temp_dir = std::env::temp_dir().join("aether_test_crashes_1");
        let analytics = CrashAnalytics::with_output_dir(&temp_dir);

        let dump_path_str = analytics.capture_minidump("weather-widget", -1073741819);
        assert!(dump_path_str.contains("weather-widget"));

        let dump_path = PathBuf::from(&dump_path_str);
        assert!(dump_path.exists(), "Minidump file must exist on disk");

        let content = std::fs::read_to_string(&dump_path).unwrap();
        assert!(content.contains("AETHER_MINIDUMP_V1"));
        assert!(content.contains("STATUS_ACCESS_VIOLATION"));
        assert!(content.contains("weather-widget"));

        let dumps = analytics.list_minidumps();
        assert!(!dumps.is_empty());

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
