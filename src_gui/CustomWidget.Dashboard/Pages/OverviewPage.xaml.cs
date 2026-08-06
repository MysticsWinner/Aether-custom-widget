// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
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
    private readonly OverviewViewModel _vm;
    private readonly DispatcherTimer _refreshTimer;

    public OverviewPage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<OverviewViewModel>();

        _refreshTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(500) };
        _refreshTimer.Tick += RefreshUI;
        _refreshTimer.Start();

        this.Unloaded += (_, _) => _refreshTimer.Stop();
    }

    private void RefreshUI(object? sender, object e)
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
        SubsystemText.Text = "9"; // Known subsystem count from architecture

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
        var ipc = App.Services.GetRequiredService<AetherIpcService>();
        bool connected = ipc.IsConnected;
        IpcDot.Fill = connected
            ? (SolidColorBrush)Application.Current.Resources["AetherSuccessBrush"]
            : (SolidColorBrush)Application.Current.Resources["AetherErrorBrush"];
        IpcText.Text = connected ? "Connected" : "Disconnected";
    }

    private async void DesktopWidgetBtn_Click(object sender, RoutedEventArgs e)
    {
        var ipc = App.Services.GetRequiredService<AetherIpcService>();
        await ipc.ToggleDesktopWidgetAsync();
    }

    private void ReloadBtn_Click(object sender, RoutedEventArgs e)
        => _ = _vm.ReloadAllCommand.ExecuteAsync(null);

    private void ThemeBtn_Click(object sender, RoutedEventArgs e)
        => _ = _vm.ToggleThemeCommand.ExecuteAsync(null);

    private void PingBtn_Click(object sender, RoutedEventArgs e)
        => _ = _vm.PingEngineCommand.ExecuteAsync(null);

    private static string FormatBytes(ulong bytes) => bytes switch
    {
        >= 1_048_576 => $"{bytes / 1_048_576.0:F1} MB/s",
        >= 1_024 => $"{bytes / 1_024.0:F1} KB/s",
        _ => $"{bytes} B/s",
    };
}
