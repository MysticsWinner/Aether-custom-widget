// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Services;

namespace CustomWidget.Dashboard.ViewModels;

/// <summary>
/// ViewModel for the Settings page — application configuration and engine preferences.
/// </summary>
public partial class SettingsViewModel : ObservableObject
{
    private readonly AetherIpcService _ipc;
    private readonly TelemetryPollerService _poller;
    private readonly string _settingsFilePath = Path.Combine(AppContext.BaseDirectory, "settings.json");

    [ObservableProperty] private int _selectedThemeIndex; // 0=Dark, 1=Light, 2=System
    [ObservableProperty] private int _pollingIntervalMs = 500;
    [ObservableProperty] private bool _autoStartEngine = false;
    [ObservableProperty] private bool _cloudSyncEnabled = false;
    [ObservableProperty] private bool _aiFeaturesEnabled = false;
    [ObservableProperty] private string _engineVersion = "—";
    [ObservableProperty] private string _statusMessage = "";

    public string[] ThemeOptions { get; } = ["Dark", "Light", "System"];

    public SettingsViewModel(AetherIpcService ipc, TelemetryPollerService poller)
    {
        _ipc = ipc;
        _poller = poller;

        LoadSettings();

        _engineVersion = string.IsNullOrEmpty(ipc.LastEngineVersion)
            ? "—"
            : $"v{ipc.LastEngineVersion}";
    }

    private void LoadSettings()
    {
        try
        {
            if (File.Exists(_settingsFilePath))
            {
                string json = File.ReadAllText(_settingsFilePath);
                var dict = JsonSerializer.Deserialize<Dictionary<string, string>>(json);
                if (dict != null)
                {
                    if (dict.TryGetValue(nameof(SelectedThemeIndex), out var themeStr) && int.TryParse(themeStr, out var themeVal))
                        SelectedThemeIndex = themeVal;
                    if (dict.TryGetValue(nameof(PollingIntervalMs), out var pollStr) && int.TryParse(pollStr, out var pollVal))
                        PollingIntervalMs = pollVal;
                    if (dict.TryGetValue(nameof(AutoStartEngine), out var autoStr) && bool.TryParse(autoStr, out var autoVal))
                        AutoStartEngine = autoVal;
                    if (dict.TryGetValue(nameof(CloudSyncEnabled), out var cloudStr) && bool.TryParse(cloudStr, out var cloudVal))
                        CloudSyncEnabled = cloudVal;
                    if (dict.TryGetValue(nameof(AiFeaturesEnabled), out var aiStr) && bool.TryParse(aiStr, out var aiVal))
                        AiFeaturesEnabled = aiVal;
                }
            }
        }
        catch { }
    }

    private void SaveSettings()
    {
        try
        {
            var dict = new Dictionary<string, string>
            {
                { nameof(SelectedThemeIndex), SelectedThemeIndex.ToString() },
                { nameof(PollingIntervalMs), PollingIntervalMs.ToString() },
                { nameof(AutoStartEngine), AutoStartEngine.ToString() },
                { nameof(CloudSyncEnabled), CloudSyncEnabled.ToString() },
                { nameof(AiFeaturesEnabled), AiFeaturesEnabled.ToString() }
            };
            string json = JsonSerializer.Serialize(dict);
            File.WriteAllText(_settingsFilePath, json);
        }
        catch { }
    }

    partial void OnSelectedThemeIndexChanged(int value)
    {
        SaveSettings();
        string mode = value switch
        {
            0 => "dark",
            1 => "light",
            2 => "system",
            _ => "dark",
        };

        _ = ApplyThemeAsync(mode);
    }

    partial void OnPollingIntervalMsChanged(int value)
    {
        _poller.PollIntervalMs = Math.Clamp(value, 100, 5000);
        SaveSettings();
    }

    partial void OnAutoStartEngineChanged(bool value)
    {
        SaveSettings();
    }

    partial void OnCloudSyncEnabledChanged(bool value)
    {
        SaveSettings();
    }

    partial void OnAiFeaturesEnabledChanged(bool value)
    {
        SaveSettings();
    }

    private async Task ApplyThemeAsync(string mode)
    {
        try
        {
            // Apply theme live to WinUI 3 Dashboard Window
            var elementTheme = mode switch
            {
                "light" => Microsoft.UI.Xaml.ElementTheme.Light,
                "system" => Microsoft.UI.Xaml.ElementTheme.Default,
                _ => Microsoft.UI.Xaml.ElementTheme.Dark,
            };

            if (App.Current.MainWindow is MainWindow window)
            {
                window.SetAppTheme(elementTheme);
            }

            // Sync theme with Core Engine Daemon via IPC
            await _ipc.SetThemeModeAsync(mode);
            StatusMessage = $"Theme updated to '{mode}'.";
        }
        catch (Exception ex)
        {
            StatusMessage = $"Failed to set theme: {ex.Message}";
        }
    }

    [RelayCommand]
    private void ResetDefaults()
    {
        SelectedThemeIndex = 0; // Dark
        PollingIntervalMs = 500;
        AutoStartEngine = false;
        CloudSyncEnabled = false;
        AiFeaturesEnabled = false;
        StatusMessage = "Settings reset to defaults.";
        SaveSettings();
    }
}
