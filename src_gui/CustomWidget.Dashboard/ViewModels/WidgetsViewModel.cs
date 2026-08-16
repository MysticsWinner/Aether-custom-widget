// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.Services;

namespace CustomWidget.Dashboard.ViewModels;

/// <summary>
/// ViewModel for the Widgets management page — lists discovered plugin manifests and running widgets.
/// Dynamically queries the Core Engine IPC daemon for real filesystem discovery and status.
/// </summary>
public partial class WidgetsViewModel : ObservableObject
{
    private readonly AetherIpcService _ipc;
    private readonly TelemetryPollerService _poller;
    private readonly WidgetSettingsService _settingsService;

    [ObservableProperty] private bool _isBusy;
    [ObservableProperty] private string _statusMessage = "";
    [ObservableProperty] private bool _isConnected;
    [ObservableProperty] private string _searchQuery = "";

    public ObservableCollection<WidgetInfo> Widgets { get; } = new();
    public ObservableCollection<WidgetInfo> DiscoveredWidgets { get; } = new();

    public WidgetsViewModel(AetherIpcService ipc, TelemetryPollerService poller, WidgetSettingsService settingsService)
    {
        _ipc = ipc;
        _poller = poller;
        _settingsService = settingsService;

        _poller.OnNewSample += _ =>
        {
            IsConnected = _ipc.IsConnected;
            RefreshRunningWidgets();
        };

        _ = DiscoverWidgetsAsync();
    }

    partial void OnSearchQueryChanged(string value)
    {
        // Triggers UI refresh for filtered view
        _ = DiscoverWidgetsAsync();
    }

    [RelayCommand]
    public async Task DiscoverWidgetsAsync()
    {
        IsBusy = true;
        StatusMessage = "Scanning filesystem for widget.toml manifests...";
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
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Discovery failed: {ex.Message}";
        }
        finally
        {
            IsBusy = false;
        }
    }

    private void RefreshRunningWidgets()
    {
        if (_poller.LastStatus is not { } status) return;

        var activeIds = status.ActiveWidgets.ToHashSet();

        // Update running widgets list
        var currentIds = Widgets.Select(w => w.Id).ToHashSet();
        if (!currentIds.SetEquals(activeIds))
        {
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
        try
        {
            string result = await _ipc.LoadWidgetAsync(manifestPath);
            bool success = result.Contains("\"status\":\"ok\"") || result.Contains("\"status\": \"ok\"");
            StatusMessage = success
                ? $"✓ Widget loaded: {manifestPath}"
                : $"✗ Load failed: {result}";

            if (success)
            {
                await Task.Delay(300);
                await DiscoverWidgetsAsync();
            }
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error: {ex.Message}";
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
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error: {ex.Message}";
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
        try
        {
            await _settingsService.ToggleLockAsync(target);
            StatusMessage = $"✓ Lock state toggled for '{target}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error toggling lock: {ex.Message}";
        }
    }

    [RelayCommand]
    private async Task SetOpacityAsync((string widgetId, double opacity) args)
    {
        try
        {
            await _settingsService.SetOpacityAsync(args.widgetId, args.opacity);
            StatusMessage = $"✓ Opacity updated to {args.opacity:P0} for '{args.widgetId}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error updating opacity: {ex.Message}";
        }
    }

    [RelayCommand]
    private async Task ToggleEnableDisableAsync(string? widgetId)
    {
        if (string.IsNullOrWhiteSpace(widgetId)) return;
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
        }
    }

    [RelayCommand]
    private async Task QuickSwapPositionAsync((string fromId, string toId) args)
    {
        try
        {
            await _ipc.QuickSwapWidgetAsync(args.fromId, args.toId, "position");
            StatusMessage = $"✓ Swapped desktop position between '{args.fromId}' and '{args.toId}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error swapping positions: {ex.Message}";
        }
    }

    [RelayCommand]
    private async Task QuickSwapConfigAsync((string fromId, string toId) args)
    {
        try
        {
            await _ipc.QuickSwapWidgetAsync(args.fromId, args.toId, "configuration");
            StatusMessage = $"✓ Swapped configuration between '{args.fromId}' and '{args.toId}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error swapping configurations: {ex.Message}";
        }
    }

    [RelayCommand]
    private async Task ResetWidgetConfigAsync(string? widgetId)
    {
        if (string.IsNullOrWhiteSpace(widgetId)) return;
        try
        {
            await _settingsService.ResetAsync(widgetId);
            StatusMessage = $"✓ Configuration reset to defaults for '{widgetId}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error resetting config: {ex.Message}";
        }
    }

    [RelayCommand]
    private async Task ResetWidgetPositionAsync(string? widgetId)
    {
        string target = string.IsNullOrWhiteSpace(widgetId) ? "perf_monitor_widget" : widgetId;
        StatusMessage = $"Resetting position for '{target}'...";
        try
        {
            await _ipc.SetWidgetPositionAsync(target, 100, 100);
            StatusMessage = $"✓ Position reset to default (100, 100) for '{target}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error resetting position: {ex.Message}";
        }
    }

    [RelayCommand]
    private async Task ToggleDesktopWidgetAsync()
    {
        StatusMessage = "Toggling desktop overlay widget...";
        try
        {
            await _ipc.ToggleDesktopWidgetAsync();
            StatusMessage = "✓ Desktop overlay widget toggled.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"✗ Error toggling overlay: {ex.Message}";
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
