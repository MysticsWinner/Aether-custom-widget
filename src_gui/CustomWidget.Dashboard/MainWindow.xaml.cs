// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using CustomWidget.Dashboard.Pages;
using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.Services.Interfaces;
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
    private const string LogSource = "MainWindow";
    private readonly ITelemetryPollerService _poller;
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

            // B3 Fix: Resolve using interface ITelemetryPollerService without concrete cast
            _poller = App.Services.GetRequiredService<ITelemetryPollerService>();

            // Subscribe to poller events (B4 Fix: UpdateStatusIndicator dispatches to UI thread)
            _poller.OnNewSample += _ => UpdateStatusIndicator(true);
            _poller.OnConnectionChanged += connected => UpdateStatusIndicator(connected);

            // Start telemetry polling
            _poller.Start();

            // Window visibility-aware polling throttle (5s when minimized)
            this.VisibilityChanged += (_, args) =>
            {
                try
                {
                    DashboardLogger.Debug(LogSource, $"Visibility changed: visible={args.Visible}");
                    _poller.SetWindowVisibility(args.Visible);
                }
                catch (Exception ex)
                {
                    DashboardLogger.Warn(LogSource, $"Error updating poller window visibility: {ex.Message}");
                }
            };

            // Detect battery vs AC power and adapt telemetry poll frequency
            try
            {
                var status = Windows.System.Power.PowerManager.BatteryStatus;
                _poller.SetPowerState(status == Windows.System.Power.BatteryStatus.Discharging);

                Windows.System.Power.PowerManager.BatteryStatusChanged += (_, _) =>
                {
                    try
                    {
                        var s = Windows.System.Power.PowerManager.BatteryStatus;
                        DashboardLogger.Debug(LogSource, $"Battery status changed: {s}");
                        _poller.SetPowerState(s == Windows.System.Power.BatteryStatus.Discharging);
                    }
                    catch (Exception ex)
                    {
                        DashboardLogger.Warn(LogSource, $"Error updating power state: {ex.Message}");
                    }
                };
            }
            catch (Exception ex)
            {
                DashboardLogger.Debug(LogSource, $"PowerManager unavailable on this system: {ex.Message}");
            }

            // Status update timer (updates UI connection indicator every 1s)
            _statusTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
            _statusTimer.Tick += (_, _) => UpdateStatusIndicator(_poller.Latest != null);
            _statusTimer.Start();

            // Handle window close → stop poller, terminate background core_engine processes, and trim memory working set.
            // Execute synchronous shutdown to ensure cleanup completes before the process terminates.
            this.Closed += (_, _) =>
            {
                try
                {
                    DashboardLogger.Info(LogSource, "MainWindow closing — initiating clean shutdown...");
                    _statusTimer?.Stop();
                    var memoryManager = App.Services.GetService<IMemoryManagerService>();
                    memoryManager?.ShutdownAndCleanAllDependenciesAsync().GetAwaiter().GetResult();
                }
                catch (Exception ex)
                {
                    DashboardLogger.Error(LogSource, "Error during window close shutdown", ex);
                }
            };

            AutoStartEngineIfNeeded();
            DashboardLogger.Info(LogSource, "MainWindow initialized successfully");
        }
        catch (Exception ex)
        {
            DashboardLogger.Fatal(LogSource, "Fatal error in MainWindow constructor", ex);
            throw;
        }
    }

    /// <summary>
    /// B11 Fix: Added structured logging to AutoStartEngineIfNeeded instead of silent catch.
    /// </summary>
    private async void AutoStartEngineIfNeeded()
    {
        try
        {
            string settingsFile = System.IO.Path.Combine(AppContext.BaseDirectory, "settings.json");
            if (System.IO.File.Exists(settingsFile))
            {
                string json = System.IO.File.ReadAllText(settingsFile);
                var dict = System.Text.Json.JsonSerializer.Deserialize<System.Collections.Generic.Dictionary<string, string>>(json);
                if (dict != null && dict.TryGetValue("AutoStartEngine", out var autoStr) && bool.TryParse(autoStr, out var autoVal) && autoVal)
                {
                    DashboardLogger.Info(LogSource, "AutoStartEngine is true in settings — launching core engine...");
                    var pm = App.Services.GetRequiredService<IProcessManagerService>();
                    await pm.StartEngineAsync();
                }
            }
            else
            {
                DashboardLogger.Debug(LogSource, "settings.json not present — skipping engine autostart");
            }
        }
        catch (Exception ex)
        {
            DashboardLogger.Warn(LogSource, $"Failed to auto-start engine: {ex.Message}");
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
            DashboardLogger.Debug(LogSource, $"Window resized to {width}x{height}");
        }
        catch (Exception ex)
        {
            DashboardLogger.Warn(LogSource, $"Failed to set window size: {ex.Message}");
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
                DashboardLogger.Debug(LogSource, "Mica system backdrop applied successfully");
            }
            else
            {
                DashboardLogger.Debug(LogSource, "MicaController not supported on this Windows version");
            }
        }
        catch (Exception ex)
        {
            DashboardLogger.Warn(LogSource, $"TrySetMicaBackdrop failed: {ex.Message}");
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
            else if (args.SelectedItem is NavigationViewItem selItem)
            {
                NavigateToPage(selItem.Tag?.ToString());
            }
        }
        catch (Exception ex)
        {
            DashboardLogger.Error(LogSource, "Error during NavView selection change", ex);
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
            DashboardLogger.Error(LogSource, "Error during NavView loaded", ex);
        }
    }

    private void NavigateToPage(string? tag)
    {
        try
        {
            DashboardLogger.Debug(LogSource, $"Navigating to page: {tag}");
            var pageType = tag switch
            {
                "Overview" => typeof(OverviewPage),
                "Tokens" => typeof(DesignTokensPage),
                "Profiles" => typeof(ProfilesPage),
                "AiComposer" => typeof(AiComposerPage),
                "Widgets" => typeof(WidgetsPage),
                "Marketplace" => typeof(MarketplacePage),
                "Snapshots" => typeof(SnapshotsPage),
                "Security" => typeof(SecurityPage),
                "Services" => typeof(ServicesPage),
                "Performance" => typeof(PerformancePage),
                "Diagnostics" => typeof(DiagnosticsPage),
                "Settings" => typeof(SettingsPage),
                "About" => typeof(AboutPage),
                _ => typeof(OverviewPage),
            };

            if (ContentFrame.CurrentSourcePageType != pageType)
            {
                ContentFrame.Navigate(pageType);
            }
        }
        catch (Exception ex)
        {
            DashboardLogger.Error(LogSource, $"NavigateToPage failed for tag '{tag}'", ex);
        }
    }

    /// <summary>
    /// Applies visual theme (Dark, Light, Default) live to the WinUI 3 Window framework element.
    /// </summary>
    public void SetAppTheme(ElementTheme theme)
    {
        try
        {
            if (Content is FrameworkElement rootElement)
            {
                rootElement.RequestedTheme = theme;
                DashboardLogger.Info(LogSource, $"App theme set to: {theme}");
            }
        }
        catch (Exception ex)
        {
            DashboardLogger.Error(LogSource, $"SetAppTheme failed for {theme}", ex);
        }
    }

    /// <summary>
    /// Updates the IPC connection status indicator in the nav pane footer and the InfoBar.
    /// B4 Fix: Ensures UI updates always execute on the UI thread via DispatcherQueue.
    /// </summary>
    private void UpdateStatusIndicator(bool connected)
    {
        if (DispatcherQueue != null && !DispatcherQueue.HasThreadAccess)
        {
            DispatcherQueue.TryEnqueue(() => UpdateStatusIndicator(connected));
            return;
        }

        try
        {
            var ipcService = App.Services.GetRequiredService<IAetherIpcService>();

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
                string version = !string.IsNullOrEmpty(ipcService.LastEngineVersion) ? ipcService.LastEngineVersion : "0.6.0";
                EngineVersionText.Text = connected ? $"v{version}" : "";
            }

            if (ConnectionInfoBar != null)
            {
                ConnectionInfoBar.IsOpen = !connected;
            }
        }
        catch (Exception ex)
        {
            DashboardLogger.Error(LogSource, "UpdateStatusIndicator error", ex);
        }
    }
}
