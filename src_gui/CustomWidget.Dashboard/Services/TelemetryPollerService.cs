// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Collections.ObjectModel;
using CustomWidget.Dashboard.Models;
using Microsoft.UI.Dispatching;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// Background telemetry polling service that periodically calls <c>GetStatus</c> via IPC
/// and maintains a rolling history buffer for real-time chart rendering.
/// Includes 3-strike connection hysteresis to prevent transient pipe timeouts from flickering connection warnings.
/// </summary>
public sealed class TelemetryPollerService
{
    private readonly AetherIpcService _ipc;
    private CancellationTokenSource? _cts;
    private Task? _pollTask;
    private DispatcherQueue? _dispatcherQueue;
    private int _consecutiveFailures;
    private bool _wasConnected;

    public ObservableCollection<TelemetrySample> History { get; } = new();

    public int MaxHistorySize { get; set; } = 120;
    public int PollIntervalMs { get; set; } = 500;

    public TelemetrySample? Latest { get; private set; }
    public EngineStatus? LastStatus { get; private set; }

    public event Action<TelemetrySample>? OnNewSample;
    public event Action<bool>? OnConnectionChanged;

    public TelemetryPollerService(AetherIpcService ipc)
    {
        _ipc = ipc;
    }

    public void Start()
    {
        if (_pollTask is not null) return;

        _dispatcherQueue = DispatcherQueue.GetForCurrentThread();
        _cts = new CancellationTokenSource();
        _pollTask = Task.Run(() => PollLoop(_cts.Token));
    }

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
                    _consecutiveFailures = 0;
                    LastStatus = status;
                    var sample = TelemetrySample.FromStatus(status);
                    Latest = sample;

                    bool connStateChanged = !_wasConnected;
                    if (connStateChanged) _wasConnected = true;

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
                    _consecutiveFailures++;

                    // Only declare disconnected after 3 consecutive failed IPC samples
                    if (_consecutiveFailures >= 3 && _wasConnected)
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
