// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using CustomWidget.Dashboard.Services.Interfaces;
using Microsoft.UI.Xaml;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// Memory and Resource Garbage Disposal Manager.
/// Handles automatic dependency shutdown on app close, working set trimming,
/// periodic garbage collection, and resource reclamation.
/// </summary>
public sealed class MemoryManagerService : IMemoryManagerService, IDisposable
{
    private const string LogSource = "MemoryManager";

    private readonly IProcessManagerService _processManager;
    private readonly ITelemetryPollerService _telemetryPoller;
    private readonly ILogCollectorService _logCollector;
    private readonly DispatcherTimer? _autoMemoryTimer;
    private bool _isDisposed;

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool SetProcessWorkingSetSize(IntPtr hProcess, IntPtr dwMinimumWorkingSetSize, IntPtr dwMaximumWorkingSetSize);

    public MemoryManagerService(
        IProcessManagerService processManager,
        ITelemetryPollerService telemetryPoller,
        ILogCollectorService logCollector)
    {
        _processManager = processManager;
        _telemetryPoller = telemetryPoller;
        _logCollector = logCollector;

        DashboardLogger.Debug(LogSource, "MemoryManagerService created");

        // Auto-cleanup timer (trims working set and collects GC garbage every 5 min)
        try
        {
            _autoMemoryTimer = new DispatcherTimer { Interval = TimeSpan.FromMinutes(5) };
            _autoMemoryTimer.Tick += (_, _) => PerformAutoMemoryCleanup();
            _autoMemoryTimer.Start();
            DashboardLogger.Info(LogSource, "Auto-memory cleanup timer started (interval=5min)");
        }
        catch
        {
            DashboardLogger.Debug(LogSource, "DispatcherTimer unavailable (headless/test environment) — auto-cleanup disabled");
        }
    }

    /// <summary>
    /// Forces immediate garbage collection and trims process physical working set memory.
    /// Reclaims physical RAM back to the operating system.
    /// </summary>
    public void OptimizeMemory()
    {
        try
        {
            long beforeBytes = GC.GetTotalMemory(false);

            GC.Collect(2, GCCollectionMode.Optimized, false, false);
            GC.WaitForPendingFinalizers();

            long afterBytes = GC.GetTotalMemory(false);
            long reclaimedKb = (beforeBytes - afterBytes) / 1024;

            // Trim working set on Windows OS
            IntPtr procHandle = Process.GetCurrentProcess().Handle;
            SetProcessWorkingSetSize(procHandle, (IntPtr)(-1), (IntPtr)(-1));

            DashboardLogger.Debug(LogSource, $"Memory optimized: GC reclaimed ~{reclaimedKb}KB (before={beforeBytes / 1024}KB, after={afterBytes / 1024}KB), working set trimmed");
        }
        catch (Exception ex)
        {
            DashboardLogger.Warn(LogSource, "Memory optimization failed", ex);
        }
    }

    public void TrimWorkingSet() => OptimizeMemory();

    /// <summary>
    /// Periodic auto-memory cleanup tick handler.
    /// </summary>
    private void PerformAutoMemoryCleanup()
    {
        DashboardLogger.Debug(LogSource, "Auto-memory cleanup tick");
        OptimizeMemory();
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

            // 5. Force final full garbage disposal and RAM working set release
            OptimizeMemory();

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
    /// B5 Fix: Dispose() now synchronously waits for shutdown with a timeout
    /// instead of fire-and-forget.
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
