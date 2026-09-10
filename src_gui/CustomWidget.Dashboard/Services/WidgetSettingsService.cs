// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Text.Json;
using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.Services.Interfaces;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// Manages per-widget settings files stored under %LOCALAPPDATA%\Aether\widget_settings\&lt;widget_id&gt;.json.
/// Provides read/write access to <see cref="WidgetDisplayOptions"/> and synchronises changes
/// with the Core Engine via <see cref="IAetherIpcService"/> IPC calls.
/// </summary>
public sealed class WidgetSettingsService : IWidgetSettingsService
{
    private const string LogSource = "WidgetSettings";

    private readonly IAetherIpcService _ipc;
    private static readonly string _settingsRoot = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "Aether", "widget_settings");

    private static readonly JsonSerializerOptions _jsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        WriteIndented = true,
    };

    public WidgetSettingsService(IAetherIpcService ipc)
    {
        _ipc = ipc;
        Directory.CreateDirectory(_settingsRoot);
        DashboardLogger.Info(LogSource, $"WidgetSettingsService initialized (settingsRoot={_settingsRoot})");
    }

    // ── Read ───────────────────────────────────────────────────────────────────

    /// <summary>
    /// Loads all persisted widget settings into a dictionary keyed by widget ID.
    /// </summary>
    public async Task<Dictionary<string, WidgetDisplayOptions>> LoadAllSettingsAsync()
    {
        var result = new Dictionary<string, WidgetDisplayOptions>();
        if (!Directory.Exists(_settingsRoot))
        {
            DashboardLogger.Debug(LogSource, "Settings root directory does not exist — returning empty");
            return result;
        }

        var files = Directory.GetFiles(_settingsRoot, "*.json");
        DashboardLogger.Debug(LogSource, $"LoadAllSettings: found {files.Length} settings file(s)");

        foreach (var file in files)
        {
            try
            {
                string json = await File.ReadAllTextAsync(file);
                var opts = JsonSerializer.Deserialize<WidgetDisplayOptions>(json, _jsonOptions);
                if (opts != null && !string.IsNullOrWhiteSpace(opts.WidgetId))
                {
                    result[opts.WidgetId] = opts;
                }
            }
            catch (Exception ex)
            {
                DashboardLogger.Warn(LogSource, $"Failed to load settings from '{Path.GetFileName(file)}': {ex.Message}");
            }
        }

        DashboardLogger.Info(LogSource, $"Loaded settings for {result.Count} widget(s)");
        return result;
    }

    /// <summary>
    /// Asynchronously retrieves display options for <paramref name="widgetId"/>.
    /// </summary>
    public Task<WidgetDisplayOptions> GetSettingsAsync(string widgetId)
    {
        return Task.FromResult(Load(widgetId));
    }

    /// <summary>
    /// Loads the display options for <paramref name="widgetId"/> from disk.
    /// Returns defaults if no file exists.
    /// </summary>
    public WidgetDisplayOptions Load(string widgetId)
    {
        string path = SettingsPath(widgetId);
        if (!File.Exists(path))
        {
            DashboardLogger.Debug(LogSource, $"No settings file for '{widgetId}' — using defaults");
            return new WidgetDisplayOptions { WidgetId = widgetId };
        }

        try
        {
            string json = File.ReadAllText(path);
            var opts = JsonSerializer.Deserialize<WidgetDisplayOptions>(json, _jsonOptions)
                   ?? new WidgetDisplayOptions { WidgetId = widgetId };
            DashboardLogger.Debug(LogSource, $"Loaded settings for '{widgetId}': opacity={opts.Opacity}, scale={opts.Scale}, locked={opts.Locked}, enabled={opts.Enabled}");
            return opts;
        }
        catch (Exception ex)
        {
            DashboardLogger.Warn(LogSource, $"Failed to parse settings for '{widgetId}': {ex.Message}");
            return new WidgetDisplayOptions { WidgetId = widgetId };
        }
    }

    // ── Write ──────────────────────────────────────────────────────────────────

    /// <summary>
    /// Persists <paramref name="options"/> for <paramref name="widgetId"/> to disk and synchronises via IPC.
    /// </summary>
    public async Task SaveSettingsAsync(string widgetId, WidgetDisplayOptions options)
    {
        options.WidgetId = widgetId;
        await SaveAsync(options);
    }

    /// <summary>
    /// Persists <paramref name="options"/> to disk and sends an IPC
    /// <c>UpdateWidgetDisplayOptions</c> to the Core Engine.
    /// </summary>
    public async Task SaveAsync(WidgetDisplayOptions options)
    {
        try
        {
            string json = JsonSerializer.Serialize(options, _jsonOptions);
            string path = SettingsPath(options.WidgetId);
            File.WriteAllText(path, json);
            DashboardLogger.Debug(LogSource, $"Settings saved for '{options.WidgetId}' → {Path.GetFileName(path)}");
        }
        catch (Exception ex)
        {
            DashboardLogger.Error(LogSource, $"Failed to save settings for '{options.WidgetId}'", ex);
        }

        await _ipc.UpdateWidgetDisplayOptionsAsync(
            options.WidgetId,
            options.Opacity,
            options.Scale,
            options.Locked,
            options.Enabled);
    }

    /// <summary>
    /// Updates the desktop screen coordinates (X, Y) for a widget and synchronises with the engine.
    /// </summary>
    public async Task SetPositionAsync(string widgetId, int x, int y)
    {
        DashboardLogger.Debug(LogSource, $"SetPosition: '{widgetId}' → ({x}, {y})");
        await _ipc.SetWidgetPositionAsync(widgetId, x, y);
    }

    /// <summary>
    /// Sets the position-lock state for a widget and synchronises with the engine.
    /// B17 Fix: Removed duplicate IPC call — SaveAsync already sends UpdateWidgetDisplayOptions.
    /// </summary>
    public async Task SetLockedAsync(string widgetId, bool locked)
    {
        DashboardLogger.Debug(LogSource, $"SetLocked: '{widgetId}' → locked={locked}");
        var opts = Load(widgetId);
        opts.Locked = locked;
        // B17 Fix: SaveAsync already sends IPC UpdateWidgetDisplayOptions — no need for separate SetWidgetLockAsync
        await SaveAsync(opts);
    }

    /// <summary>
    /// Sets the scale factor for a widget and synchronises with the engine.
    /// </summary>
    public async Task SetScaleAsync(string widgetId, double scale)
    {
        DashboardLogger.Debug(LogSource, $"SetScale: '{widgetId}' → scale={scale:F2}");
        var opts = Load(widgetId);
        opts.Scale = Math.Max(0.1, scale);
        await SaveAsync(opts);
    }

    /// <summary>
    /// Sets the opacity for a widget and synchronises with the engine.
    /// </summary>
    public async Task SetOpacityAsync(string widgetId, double opacity)
    {
        DashboardLogger.Debug(LogSource, $"SetOpacity: '{widgetId}' → opacity={opacity:F2}");
        var opts = Load(widgetId);
        opts.Opacity = Math.Clamp(opacity, 0.0, 1.0);
        await SaveAsync(opts);
    }

    /// <summary>
    /// Toggles the position-lock state for a widget and synchronises with the engine.
    /// </summary>
    public async Task ToggleLockAsync(string widgetId)
    {
        var opts = Load(widgetId);
        opts.Locked = !opts.Locked;
        DashboardLogger.Debug(LogSource, $"ToggleLock: '{widgetId}' → locked={opts.Locked}");
        await SaveAsync(opts);
    }

    /// <summary>
    /// Enables or disables a widget and synchronises with the engine.
    /// </summary>
    public async Task SetEnabledAsync(string widgetId, bool enabled)
    {
        DashboardLogger.Debug(LogSource, $"SetEnabled: '{widgetId}' → enabled={enabled}");
        var opts = Load(widgetId);
        opts.Enabled = enabled;
        await SaveAsync(opts);
    }

    /// <summary>
    /// Resets a widget's settings to defaults on disk and via IPC.
    /// </summary>
    public async Task ResetAsync(string widgetId)
    {
        DashboardLogger.Info(LogSource, $"Reset: '{widgetId}' — deleting settings file and sending IPC reset");
        try { File.Delete(SettingsPath(widgetId)); }
        catch (Exception ex)
        {
            DashboardLogger.Warn(LogSource, $"Failed to delete settings file for '{widgetId}': {ex.Message}");
        }

        await _ipc.ResetWidgetConfigAsync(widgetId);
    }

    /// <summary>
    /// Returns the absolute path of the settings JSON for a widget ID.
    /// </summary>
    private static string SettingsPath(string widgetId)
    {
        // Sanitise widget_id so it is safe as a filename
        string safe = string.Concat(widgetId.Select(c => char.IsLetterOrDigit(c) || c == '_' || c == '-' ? c : '_'));
        return Path.Combine(_settingsRoot, $"{safe}.json");
    }
}
