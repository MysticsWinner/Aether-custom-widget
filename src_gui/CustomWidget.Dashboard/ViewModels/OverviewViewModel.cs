// Copyright (c) Aether Platform. Licensed under the MIT License.

using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.Services;
using Microsoft.UI.Xaml;

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

    // Current theme cycle position: 0=dark, 1=light, 2=system
    private int _currentThemeIndex = 0;
    private static readonly string[] _themeNames = ["dark", "light", "system"];

    // Ping result timer — clears the ping banner after 8 seconds
    private DispatcherTimer? _pingClearTimer;

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

    /// <summary>
    /// Separate field for the ping result — not overwritten by the telemetry refresh loop.
    /// Displayed below the Quick Actions buttons and auto-clears after 8 seconds.
    /// </summary>
    [ObservableProperty] private string _pingResultText = "";

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
        _currentThemeIndex = (_currentThemeIndex + 1) % _themeNames.Length;
        string next = _themeNames[_currentThemeIndex];

        // Apply live to the WinUI 3 dashboard window
        var elementTheme = next switch
        {
            "light" => ElementTheme.Light,
            "system" => ElementTheme.Default,
            _ => ElementTheme.Dark,
        };

        if (App.Current.MainWindow is MainWindow window)
        {
            window.SetAppTheme(elementTheme);
        }

        // Sync theme with Core Engine daemon via IPC
        await _ipc.SetThemeModeAsync(next);

        // Show feedback in status text briefly
        PingResultText = $"🎨 Theme switched to '{next}'.";
        SchedulePingClear();
    }

    [RelayCommand]
    private async Task PingEngineAsync()
    {
        IsBusy = true;
        try
        {
            bool ok = await _ipc.PingAsync();
            // Write to dedicated PingResultText — not StatusText — so it isn't overwritten by telemetry refresh
            PingResultText = ok ? "📡 Pong! Engine is alive ✓" : "📡 No response from engine ✗";
            SchedulePingClear();

            // Also update status text (will be overwritten on next sample, that's fine)
            StatusText = ok ? "Engine Online — Pong received" : "Engine Offline";
        }
        finally
        {
            IsBusy = false;
        }
    }

    /// <summary>
    /// Starts a timer that clears <see cref="PingResultText"/> after 8 seconds.
    /// </summary>
    private void SchedulePingClear()
    {
        _pingClearTimer?.Stop();
        _pingClearTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(8) };
        _pingClearTimer.Tick += (_, _) =>
        {
            PingResultText = "";
            _pingClearTimer?.Stop();
        };
        _pingClearTimer.Start();
    }
}
