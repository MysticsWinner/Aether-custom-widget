// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;

namespace CustomWidget.Dashboard.Pages;

/// <summary>
/// Services page — engine lifecycle control and subsystem health monitoring.
/// </summary>
public sealed partial class ServicesPage : Page
{
    private const string LogSource = "ServicesPage";
    private readonly ServicesViewModel _vm;
    private readonly DispatcherTimer _refreshTimer;

    public ServicesPage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<ServicesViewModel>();

        SubsystemRepeater.ItemsSource = _vm.Subsystems;

        _refreshTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
        _refreshTimer.Tick += RefreshUI;
        _refreshTimer.Start();

        this.Unloaded += (_, _) =>
        {
            _refreshTimer.Stop();
            DashboardLogger.Debug(LogSource, "ServicesPage unloaded");
        };

        DashboardLogger.Debug(LogSource, "ServicesPage loaded");
    }

    private void RefreshUI(object? sender, object e)
    {
        EngineStatusText.Text = _vm.EngineStatusText;
        EnginePidText.Text = $"PID: {_vm.EnginePidText}";
        BusyText.Text = _vm.BusyText;
        PingResultText.Text = _vm.PingResult;

        EngineStatusDot.Fill = _vm.IsEngineRunning
            ? (SolidColorBrush)Application.Current.Resources["AetherSuccessBrush"]
            : (SolidColorBrush)Application.Current.Resources["AetherErrorBrush"];

        EmptySubsystemText.Visibility = _vm.Subsystems.Count == 0
            ? Visibility.Visible : Visibility.Collapsed;
    }

    private void StartBtn_Click(object sender, RoutedEventArgs e)
    {
        DashboardLogger.Info(LogSource, "Start Engine clicked");
        _ = _vm.StartEngineCommand.ExecuteAsync(null);
    }

    private void StopBtn_Click(object sender, RoutedEventArgs e)
    {
        DashboardLogger.Info(LogSource, "Stop Engine clicked");
        _ = _vm.StopEngineCommand.ExecuteAsync(null);
    }

    private void RestartBtn_Click(object sender, RoutedEventArgs e)
    {
        DashboardLogger.Info(LogSource, "Restart Engine clicked");
        _ = _vm.RestartEngineCommand.ExecuteAsync(null);
    }

    private void PingBtn_Click(object sender, RoutedEventArgs e)
    {
        DashboardLogger.Info(LogSource, "Ping Engine clicked");
        _ = _vm.PingEngineCommand.ExecuteAsync(null);
    }
}
