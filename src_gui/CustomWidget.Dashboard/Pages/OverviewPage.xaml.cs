// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.Services.Interfaces;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;

namespace CustomWidget.Dashboard.Pages;

/// <summary>
/// Overview (dashboard home) page — displays live hardware gauges and quick actions.
/// All metrics are real, sourced from the Rust daemon via Named Pipe IPC.
/// </summary>
public sealed partial class OverviewPage : Page
{
    private const string LogSource = "OverviewPage";
    private readonly OverviewViewModel _vm;
    private readonly System.ComponentModel.PropertyChangedEventHandler _propertyChangedHandler;

    public OverviewPage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<OverviewViewModel>();

        _propertyChangedHandler = (_, _) => RefreshUI();
        _vm.PropertyChanged += _propertyChangedHandler;

        RefreshUI();

        this.Unloaded += (_, _) =>
        {
            _vm.PropertyChanged -= _propertyChangedHandler;
            DashboardLogger.Debug(LogSource, "OverviewPage unloaded");
        };

        DashboardLogger.Debug(LogSource, "OverviewPage loaded");
    }

    private void RefreshUI()
    {
        // Update gauge values
        CpuPctText.Text = $"{_vm.CpuPct:F1}%";
        CpuBar.Value = _vm.CpuPct;

        GpuPctText.Text = $"{_vm.GpuPct:F1}%";
        GpuBar.Value = _vm.GpuPct;

        RamText.Text = _vm.MemoryText;
        RamBar.Value = _vm.MemoryPct;

        NetRecvText.Text = $"↓ {FormatBytes(_vm.NetRecvBytesPerSec)}";
        NetSentText.Text = $"↑ {FormatBytes(_vm.NetSentBytesPerSec)}";

        // Update status
        StatusText.Text = _vm.StatusText;
        VersionText.Text = _vm.EngineVersion;
        WidgetCountText.Text = _vm.ActiveWidgetCount.ToString();
        WidgetListText.Text = _vm.ActiveWidgetsText;
        // B13 Fix: Derive subsystem count from ServicesViewModel.TotalKnownSubsystems
        SubsystemText.Text = ServicesViewModel.TotalKnownSubsystems.ToString();

        // Ping / action result feedback (shown/hidden based on content)
        if (!string.IsNullOrEmpty(_vm.PingResultText))
        {
            PingResultDisplay.Text = _vm.PingResultText;
            PingResultDisplay.Visibility = Visibility.Visible;
        }
        else
        {
            PingResultDisplay.Visibility = Visibility.Collapsed;
        }

        // IPC connection dot
        bool connected = _vm.IsConnected;
        if (Application.Current.Resources.TryGetValue(connected ? "AetherSuccessBrush" : "AetherErrorBrush", out var brushObj) && brushObj is SolidColorBrush scb)
        {
            IpcDot.Fill = scb;
        }
        IpcText.Text = connected ? "Connected" : "Disconnected";
    }

    private async void DesktopWidgetBtn_Click(object sender, RoutedEventArgs e)
    {
        DashboardLogger.Info(LogSource, "DesktopWidget toggle clicked");
        var ipc = App.Services.GetRequiredService<IAetherIpcService>();
        await ipc.ToggleDesktopWidgetAsync();
    }

    private void ReloadBtn_Click(object sender, RoutedEventArgs e)
    {
        DashboardLogger.Info(LogSource, "Reload All clicked");
        _ = _vm.ReloadAllCommand.ExecuteAsync(null);
    }

    private void ThemeBtn_Click(object sender, RoutedEventArgs e)
    {
        DashboardLogger.Info(LogSource, "Toggle Theme clicked");
        _ = _vm.ToggleThemeCommand.ExecuteAsync(null);
    }

    private void PingBtn_Click(object sender, RoutedEventArgs e)
    {
        DashboardLogger.Info(LogSource, "Ping Engine clicked");
        _ = _vm.PingEngineCommand.ExecuteAsync(null);
    }

    private static string FormatBytes(ulong bytes) => bytes switch
    {
        >= 1_048_576 => $"{bytes / 1_048_576.0:F1} MB/s",
        >= 1_024 => $"{bytes / 1_024.0:F1} KB/s",
        _ => $"{bytes} B/s",
    };
}
