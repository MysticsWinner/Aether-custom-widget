// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using CustomWidget.Dashboard.Pages;
using CustomWidget.Dashboard.Services;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Composition.SystemBackdrops;
using Microsoft.UI.Windowing;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Windows.Graphics;

namespace CustomWidget.Dashboard;

/// <summary>
/// Main application window — hosts the NavigationView shell and manages page navigation.
/// Subscribes to the TelemetryPoller to update IPC connection status indicators.
/// </summary>
public sealed partial class MainWindow : Window
{
    private readonly TelemetryPollerService _poller;
    private readonly DispatcherTimer _statusTimer;

    public MainWindow()
    {
        try
        {
            this.InitializeComponent();

            // Set window size (1280x820)
            SetWindowSize(1280, 820);

            // Apply Mica backdrop (Windows 11 system material)
            TrySetMicaBackdrop();

            // Customize title bar
            ExtendsContentIntoTitleBar = true;
            Title = "Aether Studio";

            // Resolve services from DI
            _poller = App.Services.GetRequiredService<TelemetryPollerService>();

            // Start telemetry polling
            _poller.Start();

            // Status update timer (updates UI connection indicator every 1s)
            _statusTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
            _statusTimer.Tick += StatusTimer_Tick;
            _statusTimer.Start();

            // Handle window close → stop poller
            this.Closed += (_, _) =>
            {
                _statusTimer?.Stop();
                _poller?.Stop();
            };
        }
        catch (Exception ex)
        {
            App.LogCrash("MainWindow_Constructor", ex);
            throw;
        }
    }

    private void SetWindowSize(int width, int height)
    {
        try
        {
            var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
            var windowId = Microsoft.UI.Win32Interop.GetWindowIdFromWindow(hwnd);
            var appWindow = AppWindow.GetFromWindowId(windowId);
            appWindow?.Resize(new SizeInt32(width, height));
        }
        catch (Exception ex)
        {
            App.LogCrash("SetWindowSize", ex);
        }
    }

    /// <summary>
    /// Attempts to set the Mica system backdrop safely.
    /// </summary>
    private void TrySetMicaBackdrop()
    {
        try
        {
            if (MicaController.IsSupported())
            {
                SystemBackdrop = new MicaBackdrop();
            }
        }
        catch (Exception ex)
        {
            App.LogCrash("TrySetMicaBackdrop", ex);
        }
    }

    /// <summary>
    /// Handles NavigationView selection changes — navigates the ContentFrame to the correct page.
    /// </summary>
    private void NavView_SelectionChanged(NavigationView sender, NavigationViewSelectionChangedEventArgs args)
    {
        try
        {
            if (args.SelectedItemContainer is NavigationViewItem item)
            {
                NavigateToPage(item.Tag?.ToString());
            }
        }
        catch (Exception ex)
        {
            App.LogCrash("NavView_SelectionChanged", ex);
        }
    }

    /// <summary>
    /// On initial load, navigate to the Overview page.
    /// </summary>
    private void NavView_Loaded(object sender, RoutedEventArgs e)
    {
        try
        {
            NavView.SelectedItem = NavOverview;
            NavigateToPage("Overview");
        }
        catch (Exception ex)
        {
            App.LogCrash("NavView_Loaded", ex);
        }
    }

    private void NavigateToPage(string? tag)
    {
        try
        {
            var pageType = tag switch
            {
                "Overview" => typeof(OverviewPage),
                "Widgets" => typeof(WidgetsPage),
                "Services" => typeof(ServicesPage),
                "Performance" => typeof(PerformancePage),
                "Diagnostics" => typeof(DiagnosticsPage),
                "Settings" => typeof(SettingsPage),
                _ => typeof(OverviewPage),
            };

            if (ContentFrame.CurrentSourcePageType != pageType)
            {
                ContentFrame.Navigate(pageType);
            }
        }
        catch (Exception ex)
        {
            App.LogCrash($"NavigateToPage({tag})", ex);
        }
    }

    /// <summary>
    /// Updates the IPC connection status indicator in the nav pane footer and the InfoBar.
    /// </summary>
    private void StatusTimer_Tick(object? sender, object e)
    {
        try
        {
            var ipcService = App.Services.GetRequiredService<AetherIpcService>();
            bool connected = ipcService.IsConnected;

            // Update footer dot
            if (IpcStatusDot != null)
            {
                if (Application.Current.Resources.TryGetValue(connected ? "AetherSuccessBrush" : "AetherErrorBrush", out var brushObj) && brushObj is SolidColorBrush scb)
                {
                    IpcStatusDot.Fill = scb;
                }
            }

            if (IpcStatusText != null)
            {
                IpcStatusText.Text = connected ? "Connected" : "Disconnected";
            }

            if (EngineVersionText != null)
            {
                EngineVersionText.Text = (connected && _poller.Latest != null) ? $"v{ipcService.LastEngineVersion}" : "";
            }

            if (ConnectionInfoBar != null)
            {
                ConnectionInfoBar.IsOpen = !connected;
            }
        }
        catch (Exception ex)
        {
            App.LogCrash("StatusTimer_Tick", ex);
        }
    }
}
