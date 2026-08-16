// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Text.Json;
using CustomWidget.Dashboard.Models;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// Manages per-widget settings files stored under %LOCALAPPDATA%\Aether\widget_settings\&lt;widget_id&gt;.json.
/// Provides read/write access to <see cref="WidgetDisplayOptions"/> and synchronises changes
/// with the Core Engine via <see cref="AetherIpcService"/> IPC calls.
/// </summary>
public sealed class WidgetSettingsService
{
    private readonly AetherIpcService _ipc;
    private static readonly string _settingsRoot = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "Aether", "widget_settings");

    private static readonly JsonSerializerOptions _jsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        WriteIndented = true,
    };

    public WidgetSettingsService(AetherIpcService ipc)
    {
        _ipc = ipc;
        Directory.CreateDirectory(_settingsRoot);
    }

    // ── Read ───────────────────────────────────────────────────────────────────

    /// <summary>
    /// Loads the display options for <paramref name="widgetId"/> from disk.
    /// Returns defaults if no file exists.
    /// </summary>
    public WidgetDisplayOptions Load(string widgetId)
    {
        string path = SettingsPath(widgetId);
        if (!File.Exists(path))
            return new WidgetDisplayOptions { WidgetId = widgetId };

        try
        {
            string json = File.ReadAllText(path);
            return JsonSerializer.Deserialize<WidgetDisplayOptions>(json, _jsonOptions)
                   ?? new WidgetDisplayOptions { WidgetId = widgetId };
        }
        catch
        {
            return new WidgetDisplayOptions { WidgetId = widgetId };
        }
    }

    // ── Write ──────────────────────────────────────────────────────────────────

    /// <summary>
    /// Persists <paramref name="options"/> to disk and sends an IPC
    /// <c>UpdateWidgetDisplayOptions</c> to the Core Engine.
    /// </summary>
    public async Task SaveAsync(WidgetDisplayOptions options)
    {
        try
        {
            string json = JsonSerializer.Serialize(options, _jsonOptions);
            File.WriteAllText(SettingsPath(options.WidgetId), json);
        }
        catch { /* disk failure — best-effort */ }

        await _ipc.UpdateWidgetDisplayOptionsAsync(
            options.WidgetId,
            options.Opacity,
            options.Scale,
            options.Locked,
            options.Enabled);
    }

    /// <summary>
    /// Sets the opacity for a widget and synchronises with the engine.
    /// </summary>
    public async Task SetOpacityAsync(string widgetId, double opacity)
    {
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
        await SaveAsync(opts);
    }

    /// <summary>
    /// Enables or disables a widget and synchronises with the engine.
    /// </summary>
    public async Task SetEnabledAsync(string widgetId, bool enabled)
    {
        var opts = Load(widgetId);
        opts.Enabled = enabled;
        await SaveAsync(opts);
    }

    /// <summary>
    /// Resets a widget's settings to defaults on disk and via IPC.
    /// </summary>
    public async Task ResetAsync(string widgetId)
    {
        try { File.Delete(SettingsPath(widgetId)); }
        catch { }

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
