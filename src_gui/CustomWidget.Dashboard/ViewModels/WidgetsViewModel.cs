using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.Services.Interfaces;

namespace CustomWidget.Dashboard.ViewModels;

/// <summary>
/// ViewModel for the Widgets management page — lists discovered plugin manifests and running widgets.
/// Dynamically queries the Core Engine IPC daemon for real filesystem discovery and status.
/// </summary>
public partial class WidgetsViewModel : ObservableObject
{
    private const string LogSource = "WidgetsViewModel";

    private readonly IAetherIpcService _ipc;
    private readonly ITelemetryPollerService _poller;
    private readonly IWidgetSettingsService _settingsService;

    [ObservableProperty] private bool _isBusy;
    [ObservableProperty] private string _statusMessage = "";
    [ObservableProperty] private bool _isConnected;
    [ObservableProperty] private string _searchQuery = "";

    // B7 Fix: Search debounce CTS — cancels pending discovery on new keystrokes
    private CancellationTokenSource? _searchDebounceCts;

    // B6 Fix: Track last known active widget set to avoid unnecessary collection rebuilds
    private HashSet<string> _lastActiveWidgetSet = new();

    public ObservableCollection<WidgetInfo> Widgets { get; } = new();
    public ObservableCollection<WidgetInfo> DiscoveredWidgets { get; } = new();

    public WidgetsViewModel(IAetherIpcService ipc, ITelemetryPollerService poller, IWidgetSettingsService settingsService)
    {
        _ipc = ipc;
        _poller = poller;
        _settingsService = settingsService;

        _poller.OnNewSample += _ =>
        {
            IsConnected = _ipc.IsConnected;
            RefreshRunningWidgets();
        };

        DashboardLogger.Debug(LogSource, "WidgetsViewModel initialized — starting initial discovery");
        _ = DiscoverWidgetsAsync();
    }

    /// <summary>
    /// B7 Fix: Debounce search by 300ms to avoid IPC flooding on each keystroke.
    /// </summary>
    partial void OnSearchQueryChanged(string value)
    {
        // Cancel any pending debounced search
        _searchDebounceCts?.Cancel();
        _searchDebounceCts?.Dispose();
        _searchDebounceCts = new CancellationTokenSource();
        var ct = _searchDebounceCts.Token;

        DashboardLogger.Debug(LogSource, $"Search query changed: '{value}' — debouncing 300ms");

        _ = Task.Run(async () =>
        {
            try
            {
                await Task.Delay(300, ct);
                if (!ct.IsCancellationRequested)
                {
                    await DiscoverWidgetsAsync();
                }
            }
            catch (OperationCanceledException) { /* Debounce cancelled — newer keystroke arrived */ }
        });
    }

    [RelayCommand]
    public async Task DiscoverWidgetsAsync()
    {
        IsBusy = true;
        StatusMessage = "Scanning filesystem for widget.toml manifests...";
        DashboardLogger.Debug(LogSource, "Discovering widgets...");
        try
        {
            var list = await _ipc.DiscoverWidgetsAsync();
            DiscoveredWidgets.Clear();

            string query = SearchQuery.Trim().ToLowerInvariant();

            foreach (var w in list)
            {
                // Sync options from settings service
                var opts = _settingsService.Load(w.Id);
                w.Opacity = opts.Opacity;
                w.Scale = opts.Scale;
                w.IsLocked = opts.Locked;
                w.Enabled = opts.Enabled;

                if (string.IsNullOrEmpty(query) ||
                    w.Name.ToLowerInvariant().Contains(query) ||
                    w.Author.ToLowerInvariant().Contains(query) ||
                    w.Id.ToLowerInvariant().Contains(query))
                {
                    DiscoveredWidgets.Add(w);
                }
            }

            StatusMessage = $"✓ Found {list.Count} plugin manifest(s) on disk.";
            DashboardLogger.Info(LogSource, $"Widget discovery complete: {list.Count} total, {DiscoveredWidgets.Count} matching filter");
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Discovery failed: {ex.Message}";
            DashboardLogger.Error(LogSource, "Widget discovery failed", ex);
        }
        finally
        {
            IsBusy = false;
        }
    }

    /// <summary>
    /// B6 Fix: Only rebuilds the Widgets collection when the active widget set actually changes.
    /// Compares current active IDs against the last known set to avoid rebuilding 120x/minute.
    /// </summary>
    private void RefreshRunningWidgets()
    {
        if (_poller.LastStatus is not { } status) return;

        var activeIds = status.ActiveWidgets.ToHashSet();

        // B6 Fix: Skip rebuild if the active set hasn't changed
        if (_lastActiveWidgetSet.SetEquals(activeIds))
        {
            // Only update IsLoaded flags on discovered widgets (lightweight operation)
            foreach (var dw in DiscoveredWidgets)
            {
                dw.IsLoaded = activeIds.Contains(dw.Id);
            }
            return;
        }

        DashboardLogger.Debug(LogSource, $"Active widget set changed: {string.Join(", ", activeIds)}");
        _lastActiveWidgetSet = activeIds;

        // Full rebuild of running widgets list
        Widgets.Clear();
        foreach (var widgetId in status.ActiveWidgets)
        {
            var opts = _settingsService.Load(widgetId);
            Widgets.Add(new WidgetInfo
            {
                Id = widgetId,
                Name = FormatWidgetName(widgetId),
                IsLoaded = true,
                Opacity = opts.Opacity,
                Scale = opts.Scale,
                IsLocked = opts.Locked,
                Enabled = opts.Enabled,
            });
        }

        // Cross-reference running status with discovered widgets list
        foreach (var dw in DiscoveredWidgets)
        {
            dw.IsLoaded = activeIds.Contains(dw.Id);
        }
    }

    [RelayCommand]
    private async Task ToggleWidgetLoadAsync(WidgetInfo? widget)
    {
        if (widget is null || string.IsNullOrWhiteSpace(widget.ManifestPath)) return;

        DashboardLogger.Info(LogSource, $"Toggle widget load: {widget.Id} (currently loaded={widget.IsLoaded})");

        if (widget.IsLoaded)
        {
            await UnloadWidgetAsync(widget.Id);
        }
        else
        {
            await LoadWidgetAsync(widget.ManifestPath);
        }

        await DiscoverWidgetsAsync();
    }

    [RelayCommand]
    private async Task LoadWidgetAsync(string? manifestPath)
    {
        if (string.IsNullOrWhiteSpace(manifestPath)) return;

        IsBusy = true;
        StatusMessage = $"Loading {manifestPath}...";
        DashboardLogger.Info(LogSource, $"Loading widget: {manifestPath}");
        try
        {
            string result = await _ipc.LoadWidgetAsync(manifestPath);
            bool success = result.Contains("\"status\":\"ok\"") || result.Contains("\"status\": \"ok\"");
            StatusMessage = success
                ? $"✓ Widget loaded: {manifestPath}"
                : $"✗ Load failed: {result}";

            DashboardLogger.Info(LogSource, $"Load result: success={success}");

            if (success)
            {
                await Task.Delay(300);
                await DiscoverWidgetsAsync();
            }
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error: {ex.Message}";
            DashboardLogger.Error(LogSource, $"LoadWidget error for '{manifestPath}'", ex);
        }
        finally
        {
            IsBusy = false;
        }
    }

    [RelayCommand]
    private async Task UnloadWidgetAsync(string? widgetId)
    {
        if (string.IsNullOrWhiteSpace(widgetId)) return;

        IsBusy = true;
        StatusMessage = $"Unloading {widgetId}...";
        DashboardLogger.Info(LogSource, $"Unloading widget: {widgetId}");
        try
        {
            string result = await _ipc.UnloadWidgetAsync(widgetId);
            bool success = result.Contains("\"status\":\"ok\"") || result.Contains("\"status\": \"ok\"");
            if (success)
            {
                StatusMessage = $"✓ Widget unloaded: {widgetId}";
                var widget = Widgets.FirstOrDefault(w => w.Id == widgetId);
                if (widget is not null)
                    Widgets.Remove(widget);

                await Task.Delay(300);
                await DiscoverWidgetsAsync();
            }
            else
            {
                StatusMessage = $"✗ Unload failed: {result}";
            }

            DashboardLogger.Info(LogSource, $"Unload result: success={success}");
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error: {ex.Message}";
            DashboardLogger.Error(LogSource, $"UnloadWidget error for '{widgetId}'", ex);
        }
        finally
        {
            IsBusy = false;
        }
    }

    [RelayCommand]
    private async Task ReloadAllAsync()
    {
        IsBusy = true;
        StatusMessage = "Reloading all widgets...";
        DashboardLogger.Info(LogSource, "Reloading all widgets");
        try
        {
            await _ipc.ReloadAllAsync();
            StatusMessage = "✓ All widgets reloaded.";
            await Task.Delay(500);
            await DiscoverWidgetsAsync();
        }
        finally
        {
            IsBusy = false;
        }
    }

    [RelayCommand]
    private async Task ToggleWidgetLockAsync(string? widgetId)
    {
        string target = string.IsNullOrWhiteSpace(widgetId) ? "perf_monitor_widget" : widgetId;
        StatusMessage = $"Toggling lock for '{target}'...";
        DashboardLogger.Debug(LogSource, $"Toggling lock: {target}");
        try
        {
            await _settingsService.ToggleLockAsync(target);
            StatusMessage = $"✓ Lock state toggled for '{target}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error toggling lock: {ex.Message}";
            DashboardLogger.Error(LogSource, $"ToggleLock error for '{target}'", ex);
        }
    }

    [RelayCommand]
    private async Task SetOpacityAsync((string widgetId, double opacity) args)
    {
        DashboardLogger.Debug(LogSource, $"SetOpacity: {args.widgetId} → {args.opacity:F2}");
        try
        {
            await _settingsService.SetOpacityAsync(args.widgetId, args.opacity);
            StatusMessage = $"✓ Opacity updated to {args.opacity:P0} for '{args.widgetId}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error updating opacity: {ex.Message}";
            DashboardLogger.Error(LogSource, $"SetOpacity error for '{args.widgetId}'", ex);
        }
    }

    [RelayCommand]
    private async Task ToggleEnableDisableAsync(string? widgetId)
    {
        if (string.IsNullOrWhiteSpace(widgetId)) return;
        DashboardLogger.Debug(LogSource, $"ToggleEnableDisable: {widgetId}");
        try
        {
            var opts = _settingsService.Load(widgetId);
            bool next = !opts.Enabled;
            await _settingsService.SetEnabledAsync(widgetId, next);
            StatusMessage = $"✓ Widget '{widgetId}' is now {(next ? "enabled" : "disabled")}.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error toggling widget state: {ex.Message}";
            DashboardLogger.Error(LogSource, $"ToggleEnable error for '{widgetId}'", ex);
        }
    }

    [RelayCommand]
    private async Task QuickSwapPositionAsync((string fromId, string toId) args)
    {
        DashboardLogger.Info(LogSource, $"QuickSwapPosition: {args.fromId} ↔ {args.toId}");
        try
        {
            await _ipc.QuickSwapWidgetAsync(args.fromId, args.toId, "position");
            StatusMessage = $"✓ Swapped desktop position between '{args.fromId}' and '{args.toId}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error swapping positions: {ex.Message}";
            DashboardLogger.Error(LogSource, "QuickSwapPosition error", ex);
        }
    }

    [RelayCommand]
    private async Task QuickSwapConfigAsync((string fromId, string toId) args)
    {
        DashboardLogger.Info(LogSource, $"QuickSwapConfig: {args.fromId} ↔ {args.toId}");
        try
        {
            await _ipc.QuickSwapWidgetAsync(args.fromId, args.toId, "configuration");
            StatusMessage = $"✓ Swapped configuration between '{args.fromId}' and '{args.toId}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error swapping configurations: {ex.Message}";
            DashboardLogger.Error(LogSource, "QuickSwapConfig error", ex);
        }
    }

    [RelayCommand]
    private async Task ResetWidgetConfigAsync(string? widgetId)
    {
        if (string.IsNullOrWhiteSpace(widgetId)) return;
        DashboardLogger.Info(LogSource, $"ResetWidgetConfig: {widgetId}");
        try
        {
            await _settingsService.ResetAsync(widgetId);
            StatusMessage = $"✓ Configuration reset to defaults for '{widgetId}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error resetting config: {ex.Message}";
            DashboardLogger.Error(LogSource, $"ResetConfig error for '{widgetId}'", ex);
        }
    }

    [RelayCommand]
    private async Task ResetWidgetPositionAsync(string? widgetId)
    {
        string target = string.IsNullOrWhiteSpace(widgetId) ? "perf_monitor_widget" : widgetId;
        StatusMessage = $"Resetting position for '{target}'...";
        DashboardLogger.Debug(LogSource, $"ResetWidgetPosition: {target}");
        try
        {
            await _ipc.SetWidgetPositionAsync(target, 100, 100);
            StatusMessage = $"✓ Position reset to default (100, 100) for '{target}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error resetting position: {ex.Message}";
            DashboardLogger.Error(LogSource, $"ResetPosition error for '{target}'", ex);
        }
    }

    [RelayCommand]
    private async Task ToggleDesktopWidgetAsync()
    {
        StatusMessage = "Toggling desktop overlay widget...";
        DashboardLogger.Info(LogSource, "ToggleDesktopWidget");
        try
        {
            await _ipc.ToggleDesktopWidgetAsync();
            StatusMessage = "✓ Desktop overlay widget toggled.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error toggling overlay: {ex.Message}";
            DashboardLogger.Error(LogSource, "ToggleDesktopWidget error", ex);
        }
    }

    private static string FormatWidgetName(string id)
    {
        var parts = id.Split('.');
        var name = parts.Length > 0 ? parts[^1] : id;
        return name.Replace("_", " ")
            .Split(' ')
            .Select(w => w.Length > 0 ? char.ToUpper(w[0]) + w[1..] : w)
            .Aggregate((a, b) => $"{a} {b}");
    }
}
