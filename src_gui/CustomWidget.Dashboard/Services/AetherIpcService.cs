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

    /// <summary>
    /// Sends <c>DiscoverWidgets</c> command to recursively scan widget directories on disk.
    /// Returns the full list of discovered widget plugins and their metadata.
    /// </summary>
    public async Task<List<WidgetInfo>> DiscoverWidgetsAsync(List<string>? searchPaths = null)
    {
        try
        {
            var cmd = new { DiscoverWidgets = new { search_paths = searchPaths } };
            string cmdJson = JsonSerializer.Serialize(cmd);
            string responseJson = await SendRawCommandAsync(cmdJson);

            if (string.IsNullOrWhiteSpace(responseJson) || responseJson.Contains("\"status\": \"error\""))
                return new List<WidgetInfo>();

            var resp = JsonSerializer.Deserialize<DiscoverWidgetsResponse>(responseJson);
            return resp?.DiscoveredWidgets ?? new List<WidgetInfo>();
        }
        catch
        {
            return new List<WidgetInfo>();
        }
    }

    /// <summary>
    /// Queries the marketplace catalog for widget packages matching search query and category filters.
    /// </summary>
    public async Task<string> SearchMarketplaceAsync(string query, string? category = null)
    {
        var cmd = new { SearchMarketplace = new { query, category = category ?? "all" } };
        string json = JsonSerializer.Serialize(cmd);
        return await SendRawCommandAsync(json);
    }

    /// <summary>
    /// Creates a transactional system configuration snapshot.
    /// </summary>
    public async Task<string> CreateSnapshotAsync(string name)
    {
        var cmd = new { CreateSnapshot = new { name } };
        string json = JsonSerializer.Serialize(cmd);
        return await SendRawCommandAsync(json);
    }

    /// <summary>
    /// Fetches all system configuration snapshots.
    /// </summary>
    public async Task<string> ListSnapshotsAsync()
    {
        var cmd = new { ListSnapshots = new { } };
        string json = JsonSerializer.Serialize(cmd);
        return await SendRawCommandAsync(json);
    }

    /// <summary>
    /// Restores a system configuration snapshot by snapshot ID.
    /// </summary>
    public async Task<string> RestoreSnapshotAsync(string snapshotId)
    {
        var cmd = new { RestoreSnapshot = new { id = snapshotId } };
        string json = JsonSerializer.Serialize(cmd);
        return await SendRawCommandAsync(json);
    }

    /// <summary>
    /// Deletes a system configuration snapshot by snapshot ID.
    /// </summary>
    public async Task<string> DeleteSnapshotAsync(string snapshotId)
    {
        var cmd = new { DeleteSnapshot = new { id = snapshotId } };
        string json = JsonSerializer.Serialize(cmd);
        return await SendRawCommandAsync(json);
    }

    /// <summary>
    /// Fetches security sandbox audit logs and process capability states.
    /// </summary>
    public async Task<string> GetSecurityAuditLogsAsync()
    {
        var cmd = new { GetSecurityAuditLogs = new { } };
        string json = JsonSerializer.Serialize(cmd);
        return await SendRawCommandAsync(json);
    }
}
