// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Collections.ObjectModel;
using System.Text.RegularExpressions;
using CustomWidget.Dashboard.Models;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// Captures and parses log output from the Aether core engine process.
/// Subscribes to <see cref="ProcessManagerService.OnEngineOutput"/> to receive raw log lines,
/// parses them into structured <see cref="LogEntry"/> objects.
/// </summary>
public sealed partial class LogCollectorService
{
    private readonly ProcessManagerService _processManager;

    /// <summary>
    /// Circular buffer of parsed log entries (most recent last).
    /// </summary>
    public ObservableCollection<LogEntry> Entries { get; } = new();

    /// <summary>
    /// Maximum number of log entries retained.
    /// </summary>
    public int MaxEntries { get; set; } = 1000;

    /// <summary>
    /// Count of WARN-level entries since last clear.
    /// </summary>
    public int WarnCount { get; private set; }

    /// <summary>
    /// Count of ERROR-level entries since last clear.
    /// </summary>
    public int ErrorCount { get; private set; }

    /// <summary>
    /// Fired when a new log entry is parsed.
    /// </summary>
    public event Action<LogEntry>? OnNewEntry;

    public LogCollectorService(ProcessManagerService processManager)
    {
        _processManager = processManager;
        _processManager.OnEngineOutput += HandleOutputLine;
    }

    /// <summary>
    /// Clears all collected log entries and resets counters.
    /// </summary>
    public void Clear()
    {
        Entries.Clear();
        WarnCount = 0;
        ErrorCount = 0;
    }

    /// <summary>
    /// Adds a manual log entry (e.g., from IPC console output).
    /// </summary>
    public void AddManualEntry(string level, string target, string message)
    {
        var entry = new LogEntry
        {
            Timestamp = DateTime.Now,
            Level = level,
            Target = target,
            Message = message,
            RawLine = $"[{level}] {target}: {message}",
        };
        AddEntry(entry);
    }

    private void HandleOutputLine(string line)
    {
        var entry = ParseTracingLine(line);
        AddEntry(entry);
    }

    private void AddEntry(LogEntry entry)
    {
        if (entry.Level == "WARN") WarnCount++;
        if (entry.Level == "ERROR") ErrorCount++;

        lock (Entries)
        {
            Entries.Add(entry);
            while (Entries.Count > MaxEntries)
                Entries.RemoveAt(0);
        }

        OnNewEntry?.Invoke(entry);
    }

    /// <summary>
    /// Parses a Rust <c>tracing</c> log line into a structured <see cref="LogEntry"/>.
    /// Expected format: <c>2026-08-04T05:56:00.123Z  INFO core_engine::ipc_server: Message here</c>
    /// </summary>
    private static LogEntry ParseTracingLine(string line)
    {
        // Try to match tracing format: TIMESTAMP LEVEL TARGET: MESSAGE
        var match = TracingLineRegex().Match(line);
        if (match.Success)
        {
            return new LogEntry
            {
                Timestamp = DateTime.TryParse(match.Groups["ts"].Value, out var ts) ? ts : DateTime.Now,
                Level = match.Groups["level"].Value.Trim().ToUpperInvariant(),
                Target = match.Groups["target"].Value.Trim(),
                Message = match.Groups["msg"].Value.Trim(),
                RawLine = line,
            };
        }

        // Fallback: treat the whole line as an INFO message
        return new LogEntry
        {
            Timestamp = DateTime.Now,
            Level = "INFO",
            Target = "engine",
            Message = line.Trim(),
            RawLine = line,
        };
    }

    [GeneratedRegex(@"^(?<ts>\S+)\s+(?<level>TRACE|DEBUG|INFO|WARN|ERROR)\s+(?<target>[^:]+):\s*(?<msg>.*)$",
        RegexOptions.Compiled)]
    private static partial Regex TracingLineRegex();
}
