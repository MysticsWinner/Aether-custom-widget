// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.Collections.ObjectModel;
using System.Threading;
using System.Threading.Tasks;
using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.Services.Interfaces;
using Microsoft.UI.Dispatching;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// Background telemetry polling service that periodically calls <c>GetStatus</c> via IPC
/// and maintains a rolling history buffer for real-time chart rendering.
/// Includes 3-strike connection hysteresis and adaptive polling frequency.
/// </summary>
public sealed class TelemetryPollerService : ITelemetryPollerService
{
    private const string LogSource = "TelemetryPoller";

    private readonly IAetherIpcService _ipc;
    private CancellationTokenSource? _cts;
    private Task? _pollTask;
    private DispatcherQueue? _dispatcherQueue;
    private int _consecutiveFailures;
    private bool _wasConnected;
    private int _basePollIntervalMs = 500;
    private readonly SemaphoreSlim _startStopLock = new(1, 1); // B2 Fix: Prevent concurrent start/stop races
    private long _sampleCount;

    public ObservableCollection<TelemetrySample> History { get; } = new();

    public int MaxHistorySize { get; set; } = 120;

    public int PollIntervalMs
    {
        get => _basePollIntervalMs;
        set
        {
            int old = _basePollIntervalMs;
            _basePollIntervalMs = Math.Max(100, value);
            if (old != _basePollIntervalMs)
                DashboardLogger.Info(LogSource, $"Poll interval changed: {old}ms → {_basePollIntervalMs}ms");
        }
    }

    public TelemetrySample? Latest { get; private set; }
    public EngineStatus? LastStatus { get; private set; }

    public event Action<TelemetrySample>? OnNewSample;
    public event Action<bool>? OnConnectionChanged;

    public TelemetryPollerService(IAetherIpcService ipc)
    {
        _ipc = ipc;
        DashboardLogger.Debug(LogSource, "TelemetryPollerService created");
    }

    /// <summary>
    /// B2 Fix: Start is now guarded by a semaphore to prevent double-start races.
    /// </summary>
    public void Start()
    {
        _startStopLock.Wait();
        try
        {
            if (_pollTask is not null)
            {
                DashboardLogger.Debug(LogSource, "Start() called but poller is already running — skipping");
                return;
            }

            try
            {
                _dispatcherQueue = DispatcherQueue.GetForCurrentThread();
                DashboardLogger.Debug(LogSource, "DispatcherQueue captured for UI thread dispatch");
            }
            catch
            {
                DashboardLogger.Debug(LogSource, "No DispatcherQueue available (headless/test environment)");
            }

            _cts = new CancellationTokenSource();
            _pollTask = Task.Run(() => PollLoop(_cts.Token));
            DashboardLogger.Info(LogSource, $"Polling started (interval={_basePollIntervalMs}ms, maxHistory={MaxHistorySize})");
        }
        finally
        {
            _startStopLock.Release();
        }
    }

    /// <summary>
    /// B2 Fix: Stop now awaits the poll task and properly disposes the CTS to prevent races.
    /// </summary>
    public void Stop()
    {
        _startStopLock.Wait();
        try
        {
            if (_cts is null)
            {
                DashboardLogger.Debug(LogSource, "Stop() called but poller is not running — skipping");
                return;
            }

            DashboardLogger.Info(LogSource, "Stopping telemetry poller...");
            _cts.Cancel();

            // B2 Fix: Wait for the poll task to complete (with a timeout to prevent deadlocks)
            if (_pollTask is not null)
            {
                try
                {
                    _pollTask.Wait(TimeSpan.FromSeconds(3));
                }
                catch (AggregateException)
                {
                    // Expected — task was cancelled
                }
            }

            _cts.Dispose();
            _cts = null;
            _pollTask = null;
            DashboardLogger.Info(LogSource, $"Poller stopped. Total samples collected: {_sampleCount}");
        }
        finally
        {
            _startStopLock.Release();
        }
    }

    private bool _isWindowVisible = true;
    private bool _isOnBattery = false;
    private bool _isThrottled = false;

    public void SetThrottleState(bool isLowFrequency)
    {
        _isThrottled = isLowFrequency;
        DashboardLogger.Debug(LogSource, $"Throttle state changed: isLowFrequency={isLowFrequency}");
        RecalculatePollInterval();
    }

    public void SetWindowVisibility(bool isVisible)
    {
        _isWindowVisible = isVisible;
        DashboardLogger.Debug(LogSource, $"Window visibility changed: isVisible={isVisible}");
        RecalculatePollInterval();
    }

    public void SetPowerState(bool isOnBattery)
    {
        _isOnBattery = isOnBattery;
        DashboardLogger.Info(LogSource, $"Power state changed: isOnBattery={isOnBattery}");
        RecalculatePollInterval();
    }

    private void RecalculatePollInterval()
    {
        int oldInterval = _basePollIntervalMs;

        if (!_isWindowVisible)
        {
            _basePollIntervalMs = 5000; // 5s when minimized/occluded
        }
        else if (_isOnBattery)
        {
            _basePollIntervalMs = 2000; // 2s on battery
        }
        else if (_isThrottled)
        {
            _basePollIntervalMs = 2000; // 2s when throttled
        }
        else
        {
            _basePollIntervalMs = 500; // 500ms normal
        }

        if (oldInterval != _basePollIntervalMs)
        {
            DashboardLogger.Info(LogSource, $"Poll interval recalculated: {oldInterval}ms → {_basePollIntervalMs}ms (visible={_isWindowVisible}, battery={_isOnBattery}, throttled={_isThrottled})");
        }
    }

    private async Task PollLoop(CancellationToken ct)
    {
        DashboardLogger.Debug(LogSource, "Poll loop started");

        while (!ct.IsCancellationRequested)
        {
            try
            {
                var status = await _ipc.GetStatusAsync(ct).ConfigureAwait(false);

                if (status is not null)
                {
                    _consecutiveFailures = 0;
                    LastStatus = status;
                    var sample = TelemetrySample.FromStatus(status);
                    Latest = sample;
                    Interlocked.Increment(ref _sampleCount);

                    bool connStateChanged = !_wasConnected;
                    if (connStateChanged)
                    {
                        _wasConnected = true;
                        DashboardLogger.Info(LogSource, "Engine connection established — receiving telemetry");
                    }

                    // Log every 60th sample at DEBUG level to avoid log spam
                    if (Interlocked.Read(ref _sampleCount) % 60 == 0)
                    {
                        DashboardLogger.Debug(LogSource, $"Telemetry sample #{_sampleCount}: CPU={sample.CpuPct:F1}% GPU={sample.GpuPct:F1}% RAM={sample.MemoryUsedGb:F1}GB");
                    }

                    if (_dispatcherQueue is not null)
                    {
                        _dispatcherQueue.TryEnqueue(() =>
                        {
                            try
                            {
                                History.Add(sample);
                                while (History.Count > MaxHistorySize)
                                    History.RemoveAt(0);

                                OnNewSample?.Invoke(sample);

                                if (connStateChanged)
                                    OnConnectionChanged?.Invoke(true);
                            }
                            catch (Exception ex)
                            {
                                DashboardLogger.Error(LogSource, "UI update dispatch failed", ex);
                            }
                        });
                    }
                    else
                    {
                        History.Add(sample);
                        while (History.Count > MaxHistorySize)
                            History.RemoveAt(0);

                        OnNewSample?.Invoke(sample);

                        if (connStateChanged)
                            OnConnectionChanged?.Invoke(true);
                    }
                }
                else
                {
                    _consecutiveFailures++;

                    // Only declare disconnected after 3 consecutive failed IPC samples
                    if (_consecutiveFailures >= 3 && _wasConnected)
                    {
                        _wasConnected = false;
                        DashboardLogger.Warn(LogSource, $"Engine disconnected after {_consecutiveFailures} consecutive failures");

                        if (_dispatcherQueue is not null)
                        {
                            _dispatcherQueue.TryEnqueue(() => OnConnectionChanged?.Invoke(false));
                        }
                        else
                        {
                            OnConnectionChanged?.Invoke(false);
                        }
                    }
                }
            }
            catch (OperationCanceledException) when (ct.IsCancellationRequested)
            {
                break;
            }
            catch (Exception ex)
            {
                DashboardLogger.Error(LogSource, "Poll loop iteration error", ex);
            }

            try
            {
                await Task.Delay(_basePollIntervalMs, ct).ConfigureAwait(false);
            }
            catch (OperationCanceledException)
            {
                break;
            }
        }

        DashboardLogger.Debug(LogSource, "Poll loop exited");
    }
}
