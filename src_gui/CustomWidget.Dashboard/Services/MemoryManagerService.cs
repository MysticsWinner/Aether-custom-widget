// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Diagnostics;
using System.Runtime.InteropServices;
using Microsoft.UI.Xaml;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// Memory and Resource Garbage Disposal Manager.
/// Handles automatic dependency shutdown on app close, working set trimming,
/// periodic garbage collection, and resource reclamation.
/// </summary>
public sealed class MemoryManagerService : IDisposable
{
    private readonly ProcessManagerService _processManager;
    private readonly TelemetryPollerService _telemetryPoller;
    private readonly LogCollectorService _logCollector;
    private readonly DispatcherTimer? _autoMemoryTimer;
    private bool _isDisposed;

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool SetProcessWorkingSetSize(IntPtr hProcess, IntPtr dwMinimumWorkingSetSize, IntPtr dwMaximumWorkingSetSize);

    public MemoryManagerService(
        ProcessManagerService processManager,
        TelemetryPollerService telemetryPoller,
        LogCollectorService logCollector)
    {
        _processManager = processManager;
        _telemetryPoller = telemetryPoller;
        _logCollector = logCollector;

        // Auto-cleanup timer (trims working set and collects GC garbage every 30s)
        try
        {
            _autoMemoryTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(30) };
            _autoMemoryTimer.Tick += (_, _) => PerformAutoMemoryCleanup();
            _autoMemoryTimer.Start();
        }
        catch
        {
            // Headless unit test environment without WinUI XAML dispatcher context
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
            GC.Collect(2, GCCollectionMode.Forced, true, true);
            GC.WaitForPendingFinalizers();
            GC.Collect();

            // Trim working set on Windows OS
            IntPtr procHandle = Process.GetCurrentProcess().Handle;
            SetProcessWorkingSetSize(procHandle, (IntPtr)(-1), (IntPtr)(-1));
        }
        catch { }
    }

    /// <summary>
    /// Periodic auto-memory cleanup tick handler.
    /// </summary>
    private void PerformAutoMemoryCleanup()
    {
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

        try
        {
            _autoMemoryTimer?.Stop();

            // 1. Stop background telemetry poller
            _telemetryPoller.Stop();

            // 2. Stop core_engine background processes & process tree
            await _processManager.StopEngineAsync();

            // 3. Clear logs buffer
            _logCollector.Clear();

            // 4. Force final full garbage disposal and RAM working set release
            OptimizeMemory();
        }
        catch { }
    }

    public void Dispose()
    {
        _ = ShutdownAndCleanAllDependenciesAsync();
    }
}
