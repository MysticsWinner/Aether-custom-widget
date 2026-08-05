// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Text.Json;
using CustomWidget.Dashboard.Models;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// High-level IPC service wrapping <see cref="IPCClient.NamedPipeClient"/>.
/// Provides typed methods for every <c>ControlCommand</c> and tracks connection state.
/// All telemetry data is real — sourced from the Rust daemon's <c>SharedTelemetryCache</c>.
/// </summary>
public sealed class AetherIpcService
{
    private readonly IPCClient.NamedPipeClient _pipe = new();
    private bool _isConnected;
    private string _lastEngineVersion = "";

    /// <summary>
    /// True if the last IPC call succeeded (engine is reachable).
    /// </summary>
    public bool IsConnected => _isConnected;

    /// <summary>
    /// Last known engine version string from a successful GetStatus response.
    /// </summary>
    public string LastEngineVersion => _lastEngineVersion;

    /// <summary>
    /// Sends a <c>GetStatus</c> command and deserializes the real telemetry response.
    /// </summary>
    public async Task<EngineStatus?> GetStatusAsync()
    {
        try
        {
            var cmd = new GetStatusCommand();
            string response = await _pipe.SendCommandAsync(cmd.ToJson());

            if (string.IsNullOrWhiteSpace(response))
            {
                _isConnected = false;
                return null;
            }

            var status = JsonSerializer.Deserialize<EngineStatus>(response);
            if (status is not null && status.Status != "error")
            {
                _isConnected = true;
                if (!string.IsNullOrEmpty(status.EngineVersion))
                    _lastEngineVersion = status.EngineVersion;
                return status;
            }

            _isConnected = false;
            return null;
        }
        catch
        {
            _isConnected = false;
            return null;
        }
    }

    /// <summary>
    /// Sends a <c>Ping</c> command and returns true if the engine responds with <c>Pong</c>.
    /// </summary>
    public async Task<bool> PingAsync()
    {
        try
        {
            var cmd = new PingCommand();
            string response = await _pipe.SendCommandAsync(cmd.ToJson());
            _isConnected = response.Contains("pong", StringComparison.OrdinalIgnoreCase);
            return _isConnected;
        }
        catch
        {
            _isConnected = false;
            return false;
        }
    }

    /// <summary>
    /// Sends a raw JSON command string and returns the raw response.
    /// Used by the Diagnostics IPC console.
    /// </summary>
    public async Task<string> SendRawCommandAsync(string commandJson)
    {
        try
        {
            string response = await _pipe.SendCommandAsync(commandJson);
            _isConnected = !response.Contains("\"status\": \"error\"");
            return response;
        }
        catch (Exception ex)
        {
            _isConnected = false;
            return $"{{\"status\": \"error\", \"message\": \"{ex.Message}\"}}";
        }
    }

    /// <summary>
    /// Sends <c>LoadWidget</c> with the specified manifest path.
    /// </summary>
    public async Task<string> LoadWidgetAsync(string manifestPath)
    {
        var cmd = new LoadWidgetCommand { ManifestPath = manifestPath };
        return await SendRawCommandAsync(cmd.ToJson());
    }

    /// <summary>
    /// Sends <c>UnloadWidget</c> with the specified widget ID.
    /// </summary>
    public async Task<string> UnloadWidgetAsync(string widgetId)
    {
        var cmd = new UnloadWidgetCommand { WidgetId = widgetId };
        return await SendRawCommandAsync(cmd.ToJson());
    }

    /// <summary>
    /// Sends <c>SetThemeMode</c> with the specified mode ("light", "dark", "system").
    /// </summary>
    public async Task<string> SetThemeModeAsync(string mode)
    {
        var cmd = new SetThemeModeCommand { Mode = mode };
        return await SendRawCommandAsync(cmd.ToJson());
    }

    /// <summary>
    /// Sends <c>ReloadAll</c> to reload all loaded widgets.
    /// </summary>
    public async Task<string> ReloadAllAsync()
    {
        var cmd = new ReloadAllCommand();
        return await SendRawCommandAsync(cmd.ToJson());
    }

    /// <summary>
    /// Sends <c>ToggleDesktopWidget</c> to toggle the transparent desktop overlay window.
    /// </summary>
    public async Task<string> ToggleDesktopWidgetAsync()
    {
        return await SendRawCommandAsync("\"ToggleDesktopWidget\"");
    }

    /// <summary>
    /// Sends <c>SetWidgetPosition</c> to set specific (X, Y) coordinates for a widget.
    /// </summary>
    public async Task<string> SetWidgetPositionAsync(string widgetId, int x, int y)
    {
        var payload = JsonSerializer.Serialize(new { widget_id = widgetId, x, y });
        var cmdJson = $"{{\"type\": \"SetWidgetPosition\", \"payload\": {payload}}}";
        return await SendRawCommandAsync(cmdJson);
    }

    /// <summary>
    /// Sends <c>SetWidgetLock</c> to lock or unlock widget drag movement.
    /// </summary>
    public async Task<string> SetWidgetLockAsync(string widgetId, bool locked)
    {
        var payload = JsonSerializer.Serialize(new { widget_id = widgetId, locked });
        var cmdJson = $"{{\"type\": \"SetWidgetLock\", \"payload\": {payload}}}";
        return await SendRawCommandAsync(cmdJson);
    }

    /// <summary>
    /// Sends <c>ToggleWidgetLock</c> to flip widget position lock state.
    /// </summary>
    public async Task<string> ToggleWidgetLockAsync(string widgetId)
    {
        var payload = JsonSerializer.Serialize(new { widget_id = widgetId });
        var cmdJson = $"{{\"type\": \"ToggleWidgetLock\", \"payload\": {payload}}}";
        return await SendRawCommandAsync(cmdJson);
    }
}
