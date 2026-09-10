// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.Collections.Concurrent;
using System.Diagnostics;
using System.IO;
using System.Threading;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// Centralized structured logging utility for the Aether WinUI 3 Dashboard.
/// Provides thread-safe dual output (Debug console + rolling log file) with
/// severity levels, source tagging, and consistent timestamp formatting.
///
/// Usage:
///   DashboardLogger.Info("MainWindow", "Navigation completed to Overview page");
///   DashboardLogger.Error("AetherIpcService", "IPC call failed", exception);
/// </summary>
public static class DashboardLogger
{
    /// <summary>
    /// Log severity levels ordered by increasing severity.
    /// </summary>
    public enum LogLevel
    {
        Trace = 0,
        Debug = 1,
        Info = 2,
        Warn = 3,
        Error = 4,
        Fatal = 5,
    }

    /// <summary>
    /// Minimum log level that will be written. Messages below this level are silently discarded.
    /// Default: <see cref="LogLevel.Debug"/> in debug builds, <see cref="LogLevel.Info"/> in release.
    /// </summary>
    public static LogLevel MinimumLevel { get; set; } =
#if DEBUG
        LogLevel.Debug;
#else
        LogLevel.Info;
#endif

    /// <summary>
    /// When true, log messages are also written to a file on disk.
    /// </summary>
    public static bool EnableFileLogging { get; set; } = true;

    private static string? _logFilePath;
    private static readonly object _fileLock = new();
    private static readonly ConcurrentQueue<string> _writeQueue = new();
    private static int _isFlushingQueue; // 0 = idle, 1 = flushing
    private static bool _initialized;

    /// <summary>
    /// Initializes the logger with the workspace root directory.
    /// Must be called once during app startup before any logging occurs.
    /// Safe to call multiple times — subsequent calls are no-ops.
    /// </summary>
    public static void Initialize(string workspaceRoot)
    {
        if (_initialized) return;

        try
        {
            string logsDir = Path.Combine(workspaceRoot, "logs");
            Directory.CreateDirectory(logsDir);
            _logFilePath = Path.Combine(logsDir, "dashboard.log");
            _initialized = true;

            // Write startup banner
            WriteToFile($"\n{"".PadRight(80, '═')}");
            WriteToFile($"  Aether Dashboard Logger Initialized — {DateTime.Now:yyyy-MM-dd HH:mm:ss.fff}");
            WriteToFile($"  Log File: {_logFilePath}");
            WriteToFile($"  Minimum Level: {MinimumLevel}");
            WriteToFile($"{"".PadRight(80, '═')}\n");
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[DashboardLogger] Failed to initialize file logging: {ex.Message}");
            EnableFileLogging = false;
        }
    }

    /// <summary>
    /// Initializes the logger using the app's base directory when workspace root is unknown.
    /// </summary>
    public static void InitializeDefault()
    {
        if (_initialized) return;

        // Walk up from the assembly location to find Cargo.toml (workspace root)
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        while (dir is not null && !File.Exists(Path.Combine(dir.FullName, "Cargo.toml")))
        {
            dir = dir.Parent;
        }

        string root = dir?.FullName ?? AppContext.BaseDirectory;
        Initialize(root);
    }

    // ── Convenience Methods ──────────────────────────────────────────────────

    public static void Trace(string source, string message)
        => Log(LogLevel.Trace, source, message);

    public static void Debug(string source, string message)
        => Log(LogLevel.Debug, source, message);

    public static void Info(string source, string message)
        => Log(LogLevel.Info, source, message);

    public static void Warn(string source, string message, Exception? ex = null)
        => Log(LogLevel.Warn, source, message, ex);

    public static void Error(string source, string message, Exception? ex = null)
        => Log(LogLevel.Error, source, message, ex);

    public static void Fatal(string source, string message, Exception? ex = null)
        => Log(LogLevel.Fatal, source, message, ex);

    // ── Core Log Method ─────────────────────────────────────────────────────

    /// <summary>
    /// Writes a structured log entry to all active outputs (Debug console + file).
    /// Thread-safe and non-blocking for file writes.
    /// </summary>
    public static void Log(LogLevel level, string source, string message, Exception? ex = null)
    {
        if (level < MinimumLevel) return;

        string timestamp = DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss.fff");
        string levelTag = level.ToString().ToUpperInvariant().PadRight(5);
        string logLine = $"[{timestamp}] [{levelTag}] [{source}] {message}";

        if (ex is not null)
        {
            logLine += $"\n  Exception: {ex.GetType().FullName}: {ex.Message}";
            if (ex.InnerException is not null)
            {
                logLine += $"\n  Inner: {ex.InnerException.GetType().FullName}: {ex.InnerException.Message}";
            }
            if (ex.StackTrace is not null)
            {
                // Include first 5 stack frames for conciseness
                var frames = ex.StackTrace.Split('\n');
                int maxFrames = Math.Min(5, frames.Length);
                for (int i = 0; i < maxFrames; i++)
                {
                    logLine += $"\n  {frames[i].Trim()}";
                }
                if (frames.Length > maxFrames)
                {
                    logLine += $"\n  ... ({frames.Length - maxFrames} more frames)";
                }
            }
        }

        // Always write to Debug console
        System.Diagnostics.Debug.WriteLine(logLine);

        // Write to console (visible in dotnet run output)
        if (level >= LogLevel.Info)
        {
            Console.WriteLine(logLine);
        }

        // Queue for async file write
        if (EnableFileLogging && _initialized)
        {
            _writeQueue.Enqueue(logLine);
            FlushQueueAsync();
        }
    }

    // ── File Writing Infrastructure ─────────────────────────────────────────

    private static void FlushQueueAsync()
    {
        // Only one flush operation at a time
        if (Interlocked.CompareExchange(ref _isFlushingQueue, 1, 0) != 0)
            return;

        ThreadPool.QueueUserWorkItem(_ =>
        {
            try
            {
                FlushQueue();
            }
            finally
            {
                Interlocked.Exchange(ref _isFlushingQueue, 0);

                // Check if more items arrived while we were flushing
                if (!_writeQueue.IsEmpty)
                {
                    FlushQueueAsync();
                }
            }
        });
    }

    private static void FlushQueue()
    {
        if (string.IsNullOrEmpty(_logFilePath)) return;

        lock (_fileLock)
        {
            try
            {
                using var writer = new StreamWriter(_logFilePath, append: true);
                while (_writeQueue.TryDequeue(out string? line))
                {
                    writer.WriteLine(line);
                }
                writer.Flush();
            }
            catch
            {
                // File write failure — don't propagate, just drain the queue
                while (_writeQueue.TryDequeue(out _)) { }
            }
        }
    }

    private static void WriteToFile(string line)
    {
        if (!EnableFileLogging || string.IsNullOrEmpty(_logFilePath)) return;

        _writeQueue.Enqueue(line);
        FlushQueueAsync();
    }

    /// <summary>
    /// Forces all queued log entries to be written to disk synchronously.
    /// Call during application shutdown to ensure no logs are lost.
    /// </summary>
    public static void Flush()
    {
        if (!_writeQueue.IsEmpty)
        {
            FlushQueue();
        }
    }
}
