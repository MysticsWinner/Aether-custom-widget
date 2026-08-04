// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.Services;

namespace CustomWidget.Dashboard.ViewModels;

/// <summary>
/// ViewModel for the Widgets management page — lists installed widgets and provides load/unload controls.
/// Widget data comes from real IPC GetStatus responses.
/// </summary>
public partial class WidgetsViewModel : ObservableObject
{
    private readonly AetherIpcService _ipc;
    private readonly TelemetryPollerService _poller;

    [ObservableProperty] private bool _isBusy;
    [ObservableProperty] private string _statusMessage = "";
    [ObservableProperty] private bool _isConnected;

    public ObservableCollection<WidgetInfo> Widgets { get; } = new();

    public WidgetsViewModel(AetherIpcService ipc, TelemetryPollerService poller)
    {
        _ipc = ipc;
        _poller = poller;

        _poller.OnNewSample += _ =>
        {
            IsConnected = _ipc.IsConnected;
            RefreshWidgetList();
        };
    }

    private void RefreshWidgetList()
    {
        if (_poller.LastStatus is not { } status) return;

        // Only update if the list actually changed
        var currentIds = Widgets.Select(w => w.Id).ToHashSet();
        var newIds = status.ActiveWidgets.ToHashSet();

        if (currentIds.SetEquals(newIds)) return;

        Widgets.Clear();
        foreach (var widgetId in status.ActiveWidgets)
        {
            Widgets.Add(new WidgetInfo
            {
                Id = widgetId,
                Name = FormatWidgetName(widgetId),
                State = "Loaded",
            });
        }
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
            StatusMessage = result.Contains("\"status\":\"ok\"") || result.Contains("\"status\": \"ok\"")
                ? $"✓ Widget loaded: {manifestPath}"
                : $"✗ Load failed: {result}";
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
            StatusMessage = result.Contains("\"status\":\"ok\"") || result.Contains("\"status\": \"ok\"")
                ? $"✓ Widget unloaded: {widgetId}"
                : $"✗ Unload failed: {result}";
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
        }
        finally
        {
            IsBusy = false;
        }
    }

    private static string FormatWidgetName(string id)
    {
        // Convert "aether.builtin.perf_monitor" → "Perf Monitor"
        var parts = id.Split('.');
        var name = parts.Length > 0 ? parts[^1] : id;
        return name.Replace("_", " ")
            .Split(' ')
            .Select(w => w.Length > 0 ? char.ToUpper(w[0]) + w[1..] : w)
            .Aggregate((a, b) => $"{a} {b}");
    }
}
