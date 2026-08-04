// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Collections.ObjectModel;
using CustomWidget.Dashboard.Models;
using Microsoft.UI.Dispatching;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// Background telemetry polling service that periodically calls <c>GetStatus</c> via IPC
/// and maintains a rolling history buffer for real-time chart rendering.
/// 
/// All data is REAL — sourced from the Rust daemon's <c>SharedTelemetryCache</c>
/// which reads actual Windows hardware metrics (CPU via GetSystemTimes, RAM via GlobalMemoryStatusEx).
/// </summary>
public sealed class TelemetryPollerService
{
    private readonly AetherIpcService _ipc;
    private CancellationTokenSource? _cts;
    private Task? _pollTask;
    private DispatcherQueue? _dispatcherQueue;

    /// <summary>
    /// Rolling history buffer — last 120 samples (60 seconds at 500ms interval).
    /// Bound to LiveChartsCore series for real-time charts.
    /// </summary>
    public ObservableCollection<TelemetrySample> History { get; } = new();

    /// <summary>
    /// Maximum number of samples retained in the history buffer.
    /// </summary>
    public int MaxHistorySize { get; set; } = 120;

    /// <summary>
    /// Polling interval in milliseconds. Default: 500ms.
    /// </summary>
    public int PollIntervalMs { get; set; } = 500;

    /// <summary>
    /// The most recent telemetry sample, or null if no data has been received yet.
    /// </summary>
    public TelemetrySample? Latest { get; private set; }

    /// <summary>
    /// The last engine status response (contains widget list, version, etc.).
    /// </summary>
    public EngineStatus? LastStatus { get; private set; }

    /// <summary>
    /// Fired every time a new telemetry sample arrives from the engine (marshaled to UI thread).
    /// </summary>
    public event Action<TelemetrySample>? OnNewSample;

    /// <summary>
    /// Fired when connection state changes (connected ↔ disconnected, marshaled to UI thread).
    /// </summary>
    public event Action<bool>? OnConnectionChanged;

    private bool _wasConnected;

    public TelemetryPollerService(AetherIpcService ipc)
    {
        _ipc = ipc;
    }

    /// <summary>
    /// Starts the background polling loop. Must be called from the UI thread so the DispatcherQueue is captured.
    /// </summary>
    public void Start()
    {
        if (_pollTask is not null) return;

        _dispatcherQueue = DispatcherQueue.GetForCurrentThread();
        _cts = new CancellationTokenSource();
        _pollTask = Task.Run(() => PollLoop(_cts.Token));
    }

    /// <summary>
    /// Stops the background polling loop.
    /// </summary>
    public void Stop()
    {
        _cts?.Cancel();
        _pollTask = null;
    }

    private async Task PollLoop(CancellationToken ct)
    {
        while (!ct.IsCancellationRequested)
        {
            try
            {
                var status = await _ipc.GetStatusAsync();

                if (status is not null)
                {
                    LastStatus = status;
                    var sample = TelemetrySample.FromStatus(status);
                    Latest = sample;

                    bool connStateChanged = !_wasConnected;
                    if (connStateChanged) _wasConnected = true;

                    // Safely marshal UI updates to the WinUI DispatcherQueue
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
                                App.LogCrash("TelemetryPoller_UIUpdate", ex);
                            }
                        });
                    }
                    else
                    {
                        OnNewSample?.Invoke(sample);
                    }
                }
                else
                {
                    if (_wasConnected)
                    {
                        _wasConnected = false;
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
            catch (Exception ex)
            {
                App.LogCrash("TelemetryPoller_Loop", ex);
            }

            try
            {
                await Task.Delay(PollIntervalMs, ct);
            }
            catch (TaskCanceledException)
            {
                break;
            }
        }
    }
}
