// Copyright (c) Aether Platform. Licensed under the MIT License.

using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.Services;

namespace CustomWidget.Dashboard.ViewModels;

/// <summary>
/// ViewModel for the Overview (dashboard home) page.
/// Displays real-time gauges, engine status, and quick actions.
/// All telemetry values are REAL — sourced from the Rust daemon.
/// </summary>
public partial class OverviewViewModel : ObservableObject
{
    private readonly TelemetryPollerService _poller;
    private readonly AetherIpcService _ipc;

    [ObservableProperty] private float _cpuPct;
    [ObservableProperty] private float _gpuPct;
    [ObservableProperty] private float _memoryUsedGb;
    [ObservableProperty] private float _memoryTotalGb;
    [ObservableProperty] private float _memoryPct;
    [ObservableProperty] private string _memoryText = "0 / 0 GB";
    [ObservableProperty] private ulong _netRecvBytesPerSec;
    [ObservableProperty] private ulong _netSentBytesPerSec;

    [ObservableProperty] private bool _isConnected;
    [ObservableProperty] private string _engineVersion = "—";
    [ObservableProperty] private int _activeWidgetCount;
    [ObservableProperty] private string _activeWidgetsText = "No active widgets";
    [ObservableProperty] private string _statusText = "Engine Offline";
    [ObservableProperty] private bool _isBusy;

    public OverviewViewModel(TelemetryPollerService poller, AetherIpcService ipc)
    {
        _poller = poller;
        _ipc = ipc;

        _poller.OnNewSample += OnNewSample;
    }

    private void OnNewSample(TelemetrySample sample)
    {
        CpuPct = sample.CpuPct;
        GpuPct = sample.GpuPct;
        MemoryUsedGb = sample.MemoryUsedGb;
        MemoryTotalGb = sample.MemoryTotalGb;
        MemoryPct = sample.MemoryPct;
        MemoryText = $"{sample.MemoryUsedGb:F1} / {sample.MemoryTotalGb:F1} GB";
        NetRecvBytesPerSec = sample.NetRecvBytesPerSec;
        NetSentBytesPerSec = sample.NetSentBytesPerSec;

        IsConnected = _ipc.IsConnected;
        EngineVersion = string.IsNullOrEmpty(_ipc.LastEngineVersion) ? "—" : $"v{_ipc.LastEngineVersion}";
        StatusText = _ipc.IsConnected ? "Engine Online" : "Engine Offline";

        if (_poller.LastStatus is { } status)
        {
            ActiveWidgetCount = status.ActiveWidgets.Length;
            ActiveWidgetsText = status.ActiveWidgets.Length > 0
                ? string.Join(", ", status.ActiveWidgets)
                : "No active widgets";
        }
    }

    [RelayCommand]
    private async Task ReloadAllAsync()
    {
        IsBusy = true;
        try
        {
            await _ipc.ReloadAllAsync();
        }
        finally
        {
            IsBusy = false;
        }
    }

    [RelayCommand]
    private async Task ToggleThemeAsync()
    {
        // Cycle: dark → light → system → dark
        string current = "dark";
        string next = current switch
        {
            "dark" => "light",
            "light" => "system",
            _ => "dark",
        };

        await _ipc.SetThemeModeAsync(next);
    }

    [RelayCommand]
    private async Task PingEngineAsync()
    {
        IsBusy = true;
        try
        {
            bool ok = await _ipc.PingAsync();
            StatusText = ok ? "Pong! Engine is alive" : "No response from engine";
        }
        finally
        {
            IsBusy = false;
        }
    }
}
