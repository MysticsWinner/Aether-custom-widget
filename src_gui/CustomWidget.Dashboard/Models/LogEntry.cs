// Copyright (c) Aether Platform. Licensed under the MIT License.

namespace CustomWidget.Dashboard.Models;

/// <summary>
/// Parsed structured log entry from the Rust core engine's <c>tracing</c> output.
/// </summary>
public sealed class LogEntry
{
    public DateTime Timestamp { get; init; } = DateTime.Now;

    /// <summary>
    /// Log level: TRACE, DEBUG, INFO, WARN, ERROR.
    /// </summary>
    public string Level { get; init; } = "INFO";

    /// <summary>
    /// Tracing target — e.g. "core_engine::ipc_server", "perf_monitor_widget".
    /// </summary>
    public string Target { get; init; } = "";

    /// <summary>
    /// The log message text.
    /// </summary>
    public string Message { get; init; } = "";

    /// <summary>
    /// The raw unparsed log line (for fallback display).
    /// </summary>
    public string RawLine { get; init; } = "";
}
