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

            // Subscribe to poller events
            _poller.OnNewSample += _ => UpdateStatusIndicator(true);
            _poller.OnConnectionChanged += connected => UpdateStatusIndicator(connected);

            // Start telemetry polling
            _poller.Start();

            // Status update timer (updates UI connection indicator every 1s)
            _statusTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
            _statusTimer.Tick += (_, _) => UpdateStatusIndicator(_poller.Latest != null);
            _statusTimer.Start();

            // Handle window close → stop poller, terminate background core_engine processes, and trim memory working set
            this.Closed += async (_, _) =>
            {
                _statusTimer?.Stop();
                var memoryManager = App.Services.GetService<MemoryManagerService>();
                if (memoryManager != null)
                {
                    await memoryManager.ShutdownAndCleanAllDependenciesAsync();
                }
            };

            AutoStartEngineIfNeeded();
        }
        catch (Exception ex)
        {
            App.LogCrash("MainWindow_Constructor", ex);
            throw;
        }
    }

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
                    var pm = App.Services.GetRequiredService<ProcessManagerService>();
                    await pm.StartEngineAsync();
                }
            }
        }
        catch { }
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
            else if (args.SelectedItem is NavigationViewItem selItem)
            {
                NavigateToPage(selItem.Tag?.ToString());
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
            App.LogCrash($"NavigateToPage({tag})", ex);
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
            }
        }
        catch (Exception ex)
        {
            App.LogCrash("SetAppTheme", ex);
        }
    }

    /// <summary>
    /// Updates the IPC connection status indicator in the nav pane footer and the InfoBar.
    /// </summary>
    private void UpdateStatusIndicator(bool connected)
    {
        try
        {
            var ipcService = App.Services.GetRequiredService<AetherIpcService>();

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
                EngineVersionText.Text = connected ? $"v0.7.0" : "";
            }

            if (ConnectionInfoBar != null)
            {
                ConnectionInfoBar.IsOpen = !connected;
            }
        }
        catch (Exception ex)
        {
            App.LogCrash("UpdateStatusIndicator", ex);
        }
    }
}
