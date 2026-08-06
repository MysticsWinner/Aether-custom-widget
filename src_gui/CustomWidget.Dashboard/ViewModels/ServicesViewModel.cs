// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.Services;

namespace CustomWidget.Dashboard.ViewModels;

/// <summary>
/// ViewModel for the Services page — engine lifecycle control and subsystem health monitoring.
/// </summary>
public partial class ServicesViewModel : ObservableObject
{
    private readonly ProcessManagerService _processManager;
    private readonly AetherIpcService _ipc;
    private readonly TelemetryPollerService _poller;

    [ObservableProperty] private bool _isEngineRunning;
    [ObservableProperty] private string _engineStatusText = "Checking...";
    [ObservableProperty] private string _enginePidText = "—";
    [ObservableProperty] private bool _isBusy;
    [ObservableProperty] private string _busyText = "";
    [ObservableProperty] private string _pingResult = "";

    public ObservableCollection<SubsystemInfo> Subsystems { get; } = new();

    /// <summary>
    /// Known subsystems from the Aether architecture (populated from real data when available,
    /// falls back to known names from AGENTS.md).
    /// </summary>
    private static readonly string[] KnownSubsystemNames =
    [
        "telemetry_subsystem",
        "gpu_render_engine",
        "theme_engine",
        "plugin_sandbox",
        "profiler",
        "marketplace",
        "cloud_sync",
        "ai_intelligence",
        "production_readiness",
    ];

    public ServicesViewModel(
        ProcessManagerService processManager,
        AetherIpcService ipc,
        TelemetryPollerService poller)
    {
        _processManager = processManager;
        _ipc = ipc;
        _poller = poller;

        _poller.OnNewSample += _ => RefreshStatus();
        _poller.OnConnectionChanged += connected =>
        {
            RefreshStatus();
            RefreshSubsystems(connected);
        };

        // Initialize immediately so the UI shows correct state without waiting for first poll
        RefreshStatus();
        if (_ipc.IsConnected)
            RefreshSubsystems(true);
    }

    private void RefreshStatus()
    {
        IsEngineRunning = _processManager.IsEngineRunning || _ipc.IsConnected;
        EngineStatusText = IsEngineRunning ? "Running" : "Stopped";
        EnginePidText = _processManager.EnginePid?.ToString() ?? "—";
    }

    private void RefreshSubsystems(bool connected)
    {
        Subsystems.Clear();

        if (connected)
        {
            // Populate with known subsystem names — mark all as Healthy when connected
            // (The extended IPC protocol in Phase I will provide real per-subsystem health)
            foreach (var name in KnownSubsystemNames)
            {
                Subsystems.Add(new SubsystemInfo
                {
                    Name = name,
                    Health = "Healthy",
                    Description = GetSubsystemDescription(name),
                });
            }
        }
    }

    [RelayCommand]
    private async Task StartEngineAsync()
    {
        IsBusy = true;
        BusyText = "Starting engine...";
        try
        {
            bool started = await _processManager.StartEngineAsync();
            EngineStatusText = started ? "Running" : "Failed to start";
            IsEngineRunning = started;
        }
        finally
        {
            IsBusy = false;
            BusyText = "";
        }
    }

    [RelayCommand]
    private async Task StopEngineAsync()
    {
        IsBusy = true;
        BusyText = "Stopping engine...";
        try
        {
            await _processManager.StopEngineAsync();
            IsEngineRunning = false;
            EngineStatusText = "Stopped";
            Subsystems.Clear();
        }
        finally
        {
            IsBusy = false;
            BusyText = "";
        }
    }

    [RelayCommand]
    private async Task RestartEngineAsync()
    {
        IsBusy = true;
        BusyText = "Restarting engine...";
        try
        {
            bool ok = await _processManager.RestartEngineAsync();
            EngineStatusText = ok ? "Running" : "Failed to restart";
            IsEngineRunning = ok;
        }
        finally
        {
            IsBusy = false;
            BusyText = "";
        }
    }

    [RelayCommand]
    private async Task PingEngineAsync()
    {
        IsBusy = true;
        try
        {
            bool ok = await _ipc.PingAsync();
            PingResult = ok ? "✓ Pong! Engine is alive." : "✗ No response.";
        }
        finally
        {
            IsBusy = false;
        }
    }

    [RelayCommand]
    private void RefreshSubsystemList()
    {
        RefreshSubsystems(_ipc.IsConnected);
    }

    private static string GetSubsystemDescription(string name) => name switch
    {
        "telemetry_subsystem" => "Hardware metrics collector (CPU, GPU, RAM, NET)",
        "gpu_render_engine" => "DirectComposition / Direct2D rendering pipeline",
        "theme_engine" => "Theme token resolver & hot-reload watcher",
        "plugin_sandbox" => "AppContainer process isolation supervisor",
        "profiler" => "13-metric continuous performance profiler",
        "marketplace" => "Widget package manager & Ed25519 verifier",
        "cloud_sync" => "CRDT encrypted multi-device sync engine",
        "ai_intelligence" => "AI layout/theme synthesis & voice commands",
        "production_readiness" => "Security auditor, stress testing & crash analytics",
        _ => name,
    };
}
