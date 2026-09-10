// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.Collections.ObjectModel;
using CustomWidget.Dashboard.Models;

namespace CustomWidget.Dashboard.Services.Interfaces;

/// <summary>
/// Service abstraction for polling engine telemetry and streaming rolling samples.
/// </summary>
public interface ITelemetryPollerService
{
    ObservableCollection<TelemetrySample> History { get; }
    int MaxHistorySize { get; set; }
    int PollIntervalMs { get; set; }
    TelemetrySample? Latest { get; }
    EngineStatus? LastStatus { get; }

    event Action<TelemetrySample>? OnNewSample;
    event Action<bool>? OnConnectionChanged;

    void Start();
    void Stop();
    void SetThrottleState(bool isLowFrequency);
    void SetWindowVisibility(bool isVisible);
    void SetPowerState(bool isOnBattery);

    /// <summary>
    /// True if the poller has received at least one successful telemetry sample.
    /// Used by MainWindow to determine initial connection state without casting to concrete type.
    /// </summary>
    bool HasReceivedSample => Latest is not null;
}
