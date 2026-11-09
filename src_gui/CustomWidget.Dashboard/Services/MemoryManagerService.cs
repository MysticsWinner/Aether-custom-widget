// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.Diagnostics;
using System.Linq;
using System.Runtime;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using CustomWidget.Dashboard.Services.Interfaces;
using Microsoft.UI.Xaml;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// Memory and Resource Garbage Disposal Manager.
/// Handles automatic dependency shutdown on app close, working set trimming,
/// periodic garbage collection, development mode aggressive reclamation, and engine IPC coordination.
/// </summary>
public sealed class MemoryManagerService : IMemoryManagerService, IDisposable
{
    private const string LogSource = "MemoryManager";

    private readonly IProcessManagerService _processManager;
    private readonly ITelemetryPollerService _telemetryPoller;
    private readonly ILogCollectorService _logCollector;
    private readonly IAetherIpcService? _ipc;
    private readonly DispatcherTimer? _autoMemoryTimer;
    private bool _isDisposed;
    private GarbageCollectionStats _lastStats;

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool SetProcessWorkingSetSize(IntPtr hProcess, IntPtr dwMinimumWorkingSetSize, IntPtr dwMaximumWorkingSetSize);

    private bool _isDevelopmentMode;
    public bool IsDevelopmentMode
    {
        get => _isDevelopmentMode;
        set
        {
            _isDevelopmentMode = value;
            GcInterval = value ? TimeSpan.FromSeconds(15) : TimeSpan.FromMinutes(5);
            DashboardLogger.Info(LogSource, $"Development mode set to {value}, GC interval updated to {GcInterval.TotalSeconds}s");
        }
    }

    public bool IsAggressiveGcEnabled { get; set; }

    private TimeSpan _gcInterval = TimeSpan.FromMinutes(5);
    public TimeSpan GcInterval
    {
        get => _gcInterval;
        set
        {
            _gcInterval = value;
            if (_autoMemoryTimer != null)
            {
                try
                {
                    _autoMemoryTimer.Interval = value;
                    DashboardLogger.Debug(LogSource, $"Auto-memory timer interval updated to {value.TotalSeconds}s");
                }
                catch (Exception ex)
                {
                    DashboardLogger.Warn(LogSource, "Failed to update timer interval", ex);
                }
            }
        }
    }

    public MemoryManagerService(
        IProcessManagerService processManager,
        ITelemetryPollerService telemetryPoller,
        ILogCollectorService logCollector,
        IAetherIpcService? ipc = null)
    {
        _processManager = processManager;
        _telemetryPoller = telemetryPoller;
        _logCollector = logCollector;
        _ipc = ipc;

        // Auto-detect development mode from environment, debugger, or CLI arguments
        bool devEnv = Environment.GetEnvironmentVariable("AETHER_DEV_MODE") is "1" or "true"
            || Environment.GetEnvironmentVariable("DOTNET_ENVIRONMENT")?.Equals("Development", StringComparison.OrdinalIgnoreCase) == true
            || Debugger.IsAttached
            || Environment.GetCommandLineArgs().Any(a => a is "--dev" or "--dev-mode");

        bool aggEnv = Environment.GetEnvironmentVariable("AETHER_AGGRESSIVE_GC") is "1" or "true"
            || Environment.GetCommandLineArgs().Any(a => a is "--aggressive-gc");

        _isDevelopmentMode = devEnv;
        IsAggressiveGcEnabled = aggEnv;
        _gcInterval = devEnv ? TimeSpan.FromSeconds(15) : TimeSpan.FromMinutes(5);

        _lastStats = new GarbageCollectionStats(
            BeforeBytes: 0,
            AfterBytes: 0,
            ReclaimedBytes: 0,
            Gen0Count: GC.CollectionCount(0),
            Gen1Count: GC.CollectionCount(1),
            Gen2Count: GC.CollectionCount(2),
            WasAggressive: false,
            WasLohCompacted: false,
            IsDevMode: _isDevelopmentMode,
            Timestamp: DateTime.UtcNow
        );

        DashboardLogger.Info(LogSource, $"MemoryManagerService created (DevMode={_isDevelopmentMode}, AggressiveGC={IsAggressiveGcEnabled}, Interval={_gcInterval.TotalSeconds}s)");

        // Auto-cleanup timer (trims working set and collects GC garbage periodically)
        try
        {
            _autoMemoryTimer = new DispatcherTimer { Interval = _gcInterval };
            _autoMemoryTimer.Tick += (_, _) => PerformAutoMemoryCleanup();
            _autoMemoryTimer.Start();
            DashboardLogger.Info(LogSource, $"Auto-memory cleanup timer started (interval={_gcInterval.TotalSeconds}s)");
        }
        catch
        {
            DashboardLogger.Debug(LogSource, "DispatcherTimer unavailable (headless/test environment) — auto-cleanup disabled");
        }
    }

    /// <summary>
    /// Returns the metrics from the most recent garbage collection pass.
    /// </summary>
    public GarbageCollectionStats GetLastStats() => _lastStats;

    /// <summary>
    /// Forces garbage collection and trims process physical working set memory.
    /// In development mode or when aggressive is true, compacts the Large Object Heap (LOH)
    /// and performs full blocking generational collections.
    /// </summary>
    public void OptimizeMemory(bool aggressive = false, bool compactLoh = false)
    {
        try
        {
            bool shouldBeAggressive = aggressive || IsAggressiveGcEnabled || IsDevelopmentMode;
            bool shouldCompactLoh = compactLoh || IsDevelopmentMode;

            long beforeBytes = GC.GetTotalMemory(false);
            int gen0Before = GC.CollectionCount(0);
            int gen1Before = GC.CollectionCount(1);
            int gen2Before = GC.CollectionCount(2);

            if (shouldCompactLoh)
            {
                GCSettings.LargeObjectHeapCompactionMode = GCLargeObjectHeapCompactionMode.CompactOnce;
            }

            if (shouldBeAggressive)
            {
                // Full blocking generational garbage collection
                GC.Collect(2, GCCollectionMode.Forced, blocking: true, compacting: shouldCompactLoh);
                GC.WaitForPendingFinalizers();
                GC.Collect(2, GCCollectionMode.Forced, blocking: true, compacting: false);
            }
            else
            {
                GC.Collect(2, GCCollectionMode.Optimized, blocking: false, compacting: false);
                GC.WaitForPendingFinalizers();
            }

            long afterBytes = GC.GetTotalMemory(false);
            long reclaimedBytes = Math.Max(0, beforeBytes - afterBytes);

            // Trim working set on Windows OS
            IntPtr procHandle = Process.GetCurrentProcess().Handle;
            SetProcessWorkingSetSize(procHandle, (IntPtr)(-1), (IntPtr)(-1));

            _lastStats = new GarbageCollectionStats(
                BeforeBytes: beforeBytes,
                AfterBytes: afterBytes,
                ReclaimedBytes: reclaimedBytes,
                Gen0Count: GC.CollectionCount(0) - gen0Before,
                Gen1Count: GC.CollectionCount(1) - gen1Before,
                Gen2Count: GC.CollectionCount(2) - gen2Before,
                WasAggressive: shouldBeAggressive,
                WasLohCompacted: shouldCompactLoh,
                IsDevMode: IsDevelopmentMode,
                Timestamp: DateTime.UtcNow
            );

            DashboardLogger.Debug(LogSource, $"Memory optimized: reclaimed ~{reclaimedBytes / 1024}KB (before={beforeBytes / 1024}KB, after={afterBytes / 1024}KB, aggressive={shouldBeAggressive}, loh={shouldCompactLoh}), working set trimmed");
        }
        catch (Exception ex)
        {
            DashboardLogger.Warn(LogSource, "Memory optimization failed", ex);
        }
    }

    public void TrimWorkingSet() => OptimizeMemory(aggressive: false, compactLoh: false);

    /// <summary>
    /// Dispatches a garbage collection command to the Core Engine daemon over IPC.
    /// </summary>
    public async Task<string> TriggerEngineGarbageCollectionAsync(bool aggressive = false, bool trimWorkingSet = true)
    {
        if (_ipc == null)
        {
            DashboardLogger.Debug(LogSource, "IPC service unavailable for engine GC dispatch");
            return "{\"status\":\"skipped\",\"reason\":\"no_ipc\"}";
        }

        try
        {
            bool shouldBeAggressive = aggressive || IsAggressiveGcEnabled || IsDevelopmentMode;
            string command = $"{{\"TriggerGarbageCollection\":{{\"aggressive\":{shouldBeAggressive.ToString().ToLowerInvariant()},\"trim_working_set\":{trimWorkingSet.ToString().ToLowerInvariant()}}}}}";
            DashboardLogger.Info(LogSource, $"Dispatching engine GC over IPC: {command}");
            string response = await _ipc.SendRawCommandAsync(command);
            DashboardLogger.Debug(LogSource, $"Engine GC response: {response}");
            return response;
        }
        catch (Exception ex)
        {
            DashboardLogger.Warn(LogSource, "Failed to dispatch engine GC over IPC", ex);
            return $"{{\"status\":\"error\",\"message\":\"{ex.Message}\"}}";
        }
    }

    /// <summary>
    /// Periodic auto-memory cleanup tick handler.
    /// </summary>
    private void PerformAutoMemoryCleanup()
    {
        DashboardLogger.Debug(LogSource, $"Auto-memory cleanup tick (DevMode={IsDevelopmentMode})");
        OptimizeMemory(aggressive: IsDevelopmentMode, compactLoh: IsDevelopmentMode);

        // In dev mode, also trigger engine GC asynchronously
        if (IsDevelopmentMode && _ipc != null)
        {
            _ = TriggerEngineGarbageCollectionAsync(aggressive: true, trimWorkingSet: true);
        }
    }

    /// <summary>
    /// Cleanly closes all dependencies (core_engine daemon, IPC poller, file watchers)
    /// and disposes all allocated native/managed resources on app exit.
    /// </summary>
    public async Task ShutdownAndCleanAllDependenciesAsync()
    {
        if (_isDisposed) return;
        _isDisposed = true;

        DashboardLogger.Info(LogSource, "Beginning full shutdown sequence...");

        try
        {
            _autoMemoryTimer?.Stop();
            DashboardLogger.Debug(LogSource, "Auto-memory timer stopped");

            // 1. Stop background telemetry poller
            _telemetryPoller.Stop();
            DashboardLogger.Debug(LogSource, "Telemetry poller stopped");

            // 2. Stop core_engine background processes & process tree
            await _processManager.StopEngineAsync();
            DashboardLogger.Debug(LogSource, "Engine process stopped");

            // 3. Clear logs buffer
            await _logCollector.ClearLogsAsync();
            DashboardLogger.Debug(LogSource, "Log buffer cleared");

            // 4. Flush dashboard logger before final GC
            DashboardLogger.Flush();

            // 5. Force final aggressive garbage disposal and RAM working set release
            OptimizeMemory(aggressive: true, compactLoh: true);

            DashboardLogger.Info(LogSource, "Full shutdown sequence completed successfully");
            DashboardLogger.Flush();
        }
        catch (Exception ex)
        {
            DashboardLogger.Error(LogSource, "Shutdown sequence encountered errors", ex);
            DashboardLogger.Flush();
        }
    }

    /// <summary>
    /// Synchronously waits for shutdown with a timeout on disposal.
    /// </summary>
    public void Dispose()
    {
        try
        {
            ShutdownAndCleanAllDependenciesAsync().Wait(TimeSpan.FromSeconds(5));
        }
        catch (Exception ex)
        {
            DashboardLogger.Error(LogSource, "Dispose timeout or error during shutdown", ex);
        }
    }
}

