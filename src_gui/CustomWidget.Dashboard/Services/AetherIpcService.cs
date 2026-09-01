// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.Collections.Generic;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.Services.Interfaces;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// High-level IPC service wrapping <see cref="IPCClient.NamedPipeClient"/>.
/// Provides typed methods for every <c>ControlCommand</c> and tracks connection state.
/// All telemetry data is real — sourced from the Rust daemon's <c>SharedTelemetryCache</c>.
/// </summary>
public sealed class AetherIpcService : IAetherIpcService
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
    public async Task<EngineStatus?> GetStatusAsync(CancellationToken ct = default)
    {
        try
        {
            var cmd = new GetStatusCommand();
            string response = await _pipe.SendCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);

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
    public async Task<bool> PingAsync(CancellationToken ct = default)
    {
        try
        {
            var cmd = new PingCommand();
            string response = await _pipe.SendCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
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
    public async Task<string> SendRawCommandAsync(string commandJson, CancellationToken ct = default)
    {
        try
        {
            string response = await _pipe.SendCommandAsync(commandJson, ct).ConfigureAwait(false);
            _isConnected = !response.Contains("\"status\": \"error\"", StringComparison.OrdinalIgnoreCase);
            return response;
        }
        catch (Exception ex)
        {
            _isConnected = false;
            return $"{{\"status\": \"error\", \"message\": \"{ex.Message}\"}}";
        }
    }

    public async Task<string> LoadWidgetAsync(string manifestPath, CancellationToken ct = default)
    {
        var cmd = new LoadWidgetCommand { ManifestPath = manifestPath };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> UnloadWidgetAsync(string widgetId, CancellationToken ct = default)
    {
        var cmd = new UnloadWidgetCommand { WidgetId = widgetId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> SetThemeModeAsync(string mode, CancellationToken ct = default)
    {
        var cmd = new SetThemeModeCommand { Mode = mode };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ReloadAllAsync(CancellationToken ct = default)
    {
        var cmd = new ReloadAllCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ToggleDesktopWidgetAsync(CancellationToken ct = default)
    {
        var cmd = new ToggleDesktopWidgetCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> SetWidgetPositionAsync(string widgetId, int x, int y, CancellationToken ct = default)
    {
        var cmd = new SetWidgetPositionCommand { WidgetId = widgetId, X = x, Y = y };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> SetWidgetLockAsync(string widgetId, bool locked, CancellationToken ct = default)
    {
        var cmd = new SetWidgetLockCommand { WidgetId = widgetId, Locked = locked };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ToggleWidgetLockAsync(string widgetId, CancellationToken ct = default)
    {
        var cmd = new ToggleWidgetLockCommand { WidgetId = widgetId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<List<WidgetInfo>> DiscoverWidgetsAsync(List<string>? searchPaths = null, CancellationToken ct = default)
    {
        try
        {
            var cmd = new DiscoverWidgetsCommand { SearchPaths = searchPaths };
            string responseJson = await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);

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

    public async Task<string> SearchMarketplaceAsync(string query, string? category = null, CancellationToken ct = default)
    {
        var cmd = new SearchMarketplaceCommand { Query = query, Category = category ?? "all" };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> CreateSnapshotAsync(string name, CancellationToken ct = default)
    {
        var cmd = new CreateSnapshotCommand { Name = name };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ListSnapshotsAsync(CancellationToken ct = default)
    {
        var cmd = new ListSnapshotsCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> RestoreSnapshotAsync(string snapshotId, CancellationToken ct = default)
    {
        var cmd = new RestoreSnapshotCommand { SnapshotId = snapshotId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> DeleteSnapshotAsync(string snapshotId, CancellationToken ct = default)
    {
        var cmd = new DeleteSnapshotCommand { SnapshotId = snapshotId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ExportSnapshotAsync(string snapshotId, string path, CancellationToken ct = default)
    {
        var cmd = new ExportSnapshotCommand { SnapshotId = snapshotId, Path = path };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ImportSnapshotAsync(string path, CancellationToken ct = default)
    {
        var cmd = new ImportSnapshotCommand { Path = path };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> GetSecurityAuditLogsAsync(CancellationToken ct = default)
    {
        var cmd = new GetAuditLogsCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> GetDiagnosticsAsync(CancellationToken ct = default)
    {
        var cmd = new GetDiagnosticsCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> GetSubsystemHealthAsync(CancellationToken ct = default)
    {
        var cmd = new GetSubsystemHealthCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> GetCrashHistoryAsync(string? widgetId = null, CancellationToken ct = default)
    {
        var payload = JsonSerializer.Serialize(new { GetCrashHistory = new { widget_id = widgetId } });
        return await SendRawCommandAsync(payload, ct).ConfigureAwait(false);
    }

    public async Task<string> GetLaunchModeAsync(CancellationToken ct = default)
    {
        var cmd = new GetLaunchModeCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ExitSafeModeAsync(CancellationToken ct = default)
    {
        var cmd = new ExitSafeModeCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> GetQuarantineListAsync(CancellationToken ct = default)
    {
        var cmd = new GetQuarantineListCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ReleaseQuarantineAsync(string widgetId, CancellationToken ct = default)
    {
        var payload = JsonSerializer.Serialize(new { ReleaseQuarantine = new { widget_id = widgetId } });
        return await SendRawCommandAsync(payload, ct).ConfigureAwait(false);
    }

    public async Task<string> ListWidgetsAsync(CancellationToken ct = default)
    {
        var cmd = new ListWidgetsCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> UpdateWidgetDisplayOptionsAsync(
        string widgetId, double? opacity = null, double? scale = null,
        bool? locked = null, bool? enabled = null, CancellationToken ct = default)
    {
        var cmd = new UpdateWidgetDisplayOptionsCommand
        {
            WidgetId = widgetId,
            Opacity = opacity,
            Scale = scale,
            Locked = locked,
            Enabled = enabled
        };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> QuickSwapWidgetAsync(string fromId, string toId, string mode = "position", CancellationToken ct = default)
    {
        var cmd = new QuickSwapWidgetCommand { FromId = fromId, ToId = toId, Mode = mode };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> EnableWidgetAsync(string widgetId, CancellationToken ct = default)
    {
        var cmd = new EnableWidgetCommand { WidgetId = widgetId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> DisableWidgetAsync(string widgetId, CancellationToken ct = default)
    {
        var cmd = new DisableWidgetCommand { WidgetId = widgetId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> SetWidgetOpacityAsync(string widgetId, double opacity, CancellationToken ct = default)
    {
        var cmd = new SetWidgetOpacityCommand
        {
            WidgetId = widgetId,
            Opacity = (float)Math.Clamp(opacity, 0.0, 1.0)
        };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ResetWidgetConfigAsync(string widgetId, CancellationToken ct = default)
    {
        var cmd = new ResetWidgetConfigCommand { WidgetId = widgetId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }
}
