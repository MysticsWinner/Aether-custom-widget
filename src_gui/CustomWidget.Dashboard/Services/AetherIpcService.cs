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
    private const string LogSource = "AetherIpcService";
    private readonly IPCClient.NamedPipeClient _pipe = new();
    private volatile bool _isConnected;
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
                SetConnectionState(false, "GetStatusAsync: empty response");
                return null;
            }

            // B16 Fix: Use JSON parsing instead of string-contains for reliable error detection
            if (IsErrorResponse(response))
            {
                SetConnectionState(false, "GetStatusAsync: error response");
                return null;
            }

            var status = JsonSerializer.Deserialize<EngineStatus>(response);
            if (status is not null && status.Status != "error")
            {
                SetConnectionState(true, "GetStatusAsync: valid status received");
                if (!string.IsNullOrEmpty(status.EngineVersion))
                    _lastEngineVersion = status.EngineVersion;
                return status;
            }

            SetConnectionState(false, "GetStatusAsync: null or error status deserialized");
            return null;
        }
        catch (Exception ex)
        {
            DashboardLogger.Debug(LogSource, $"GetStatusAsync failed: {ex.Message}");
            SetConnectionState(false, $"GetStatusAsync: exception — {ex.Message}");
            return null;
        }
    }

    /// <summary>
    /// Sends a <c>Ping</c> command and returns true if the engine responds with <c>Pong</c>.
    /// </summary>
    public async Task<bool> PingAsync(CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, "Sending Ping command...");
        try
        {
            var cmd = new PingCommand();
            string response = await _pipe.SendCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
            _isConnected = response.Contains("pong", StringComparison.OrdinalIgnoreCase);
            DashboardLogger.Debug(LogSource, $"Ping result: connected={_isConnected}");
            return _isConnected;
        }
        catch (Exception ex)
        {
            DashboardLogger.Warn(LogSource, $"Ping failed: {ex.Message}");
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
        DashboardLogger.Debug(LogSource, $"SendRaw: {Truncate(commandJson, 120)}");
        try
        {
            string response = await _pipe.SendCommandAsync(commandJson, ct).ConfigureAwait(false);
            // B16 Fix: Use JSON parsing instead of fragile string-contains for error detection
            _isConnected = !IsErrorResponse(response);
            DashboardLogger.Debug(LogSource, $"SendRaw response ({response.Length} chars): connected={_isConnected}");
            return response;
        }
        catch (Exception ex)
        {
            DashboardLogger.Error(LogSource, $"SendRaw failed for command: {Truncate(commandJson, 80)}", ex);
            _isConnected = false;
            return $"{{\"status\": \"error\", \"message\": \"{EscapeJsonString(ex.Message)}\"}}";
        }
    }

    public async Task<string> LoadWidgetAsync(string manifestPath, CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, $"LoadWidget: {manifestPath}");
        var cmd = new LoadWidgetCommand { ManifestPath = manifestPath };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> UnloadWidgetAsync(string widgetId, CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, $"UnloadWidget: {widgetId}");
        var cmd = new UnloadWidgetCommand { WidgetId = widgetId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> SetThemeModeAsync(string mode, CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, $"SetThemeMode: {mode}");
        var cmd = new SetThemeModeCommand { Mode = mode };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ReloadAllAsync(CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, "ReloadAll requested");
        var cmd = new ReloadAllCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ToggleDesktopWidgetAsync(CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, "ToggleDesktopWidget requested");
        var cmd = new ToggleDesktopWidgetCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> SetWidgetPositionAsync(string widgetId, int x, int y, CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, $"SetWidgetPosition: {widgetId} → ({x}, {y})");
        var cmd = new SetWidgetPositionCommand { WidgetId = widgetId, X = x, Y = y };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> SetWidgetLockAsync(string widgetId, bool locked, CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, $"SetWidgetLock: {widgetId} → locked={locked}");
        var cmd = new SetWidgetLockCommand { WidgetId = widgetId, Locked = locked };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ToggleWidgetLockAsync(string widgetId, CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, $"ToggleWidgetLock: {widgetId}");
        var cmd = new ToggleWidgetLockCommand { WidgetId = widgetId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<List<WidgetInfo>> DiscoverWidgetsAsync(List<string>? searchPaths = null, CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, $"DiscoverWidgets: paths={searchPaths?.Count ?? 0}");
        try
        {
            var cmd = new DiscoverWidgetsCommand { SearchPaths = searchPaths };
            string responseJson = await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);

            if (string.IsNullOrWhiteSpace(responseJson) || IsErrorResponse(responseJson))
            {
                DashboardLogger.Debug(LogSource, "DiscoverWidgets: empty or error response");
                return new List<WidgetInfo>();
            }

            var resp = JsonSerializer.Deserialize<DiscoverWidgetsResponse>(responseJson);
            int count = resp?.DiscoveredWidgets?.Count ?? 0;
            DashboardLogger.Debug(LogSource, $"DiscoverWidgets: found {count} widgets");
            return resp?.DiscoveredWidgets ?? new List<WidgetInfo>();
        }
        catch (Exception ex)
        {
            DashboardLogger.Warn(LogSource, "DiscoverWidgets failed", ex);
            return new List<WidgetInfo>();
        }
    }

    public async Task<string> SearchMarketplaceAsync(string query, string? category = null, CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, $"SearchMarketplace: query='{query}' category='{category}'");
        var cmd = new SearchMarketplaceCommand { Query = query, Category = category ?? "all" };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> CreateSnapshotAsync(string name, CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, $"CreateSnapshot: '{name}'");
        var cmd = new CreateSnapshotCommand { Name = name };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ListSnapshotsAsync(CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, "ListSnapshots requested");
        var cmd = new ListSnapshotsCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> RestoreSnapshotAsync(string snapshotId, CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, $"RestoreSnapshot: {snapshotId}");
        var cmd = new RestoreSnapshotCommand { SnapshotId = snapshotId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> DeleteSnapshotAsync(string snapshotId, CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, $"DeleteSnapshot: {snapshotId}");
        var cmd = new DeleteSnapshotCommand { SnapshotId = snapshotId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ExportSnapshotAsync(string snapshotId, string path, CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, $"ExportSnapshot: {snapshotId} → {path}");
        var cmd = new ExportSnapshotCommand { SnapshotId = snapshotId, Path = path };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ImportSnapshotAsync(string path, CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, $"ImportSnapshot: {path}");
        var cmd = new ImportSnapshotCommand { Path = path };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> GetSecurityAuditLogsAsync(CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, "GetSecurityAuditLogs requested");
        var cmd = new GetAuditLogsCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> GetDiagnosticsAsync(CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, "GetDiagnostics requested");
        var cmd = new GetDiagnosticsCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> GetSubsystemHealthAsync(CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, "GetSubsystemHealth requested");
        var cmd = new GetSubsystemHealthCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> GetCrashHistoryAsync(string? widgetId = null, CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, $"GetCrashHistory: widgetId={widgetId ?? "all"}");
        var payload = JsonSerializer.Serialize(new { GetCrashHistory = new { widget_id = widgetId } });
        return await SendRawCommandAsync(payload, ct).ConfigureAwait(false);
    }

    public async Task<string> GetLaunchModeAsync(CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, "GetLaunchMode requested");
        var cmd = new GetLaunchModeCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ExitSafeModeAsync(CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, "ExitSafeMode requested");
        var cmd = new ExitSafeModeCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> GetQuarantineListAsync(CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, "GetQuarantineList requested");
        var cmd = new GetQuarantineListCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ReleaseQuarantineAsync(string widgetId, CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, $"ReleaseQuarantine: {widgetId}");
        var payload = JsonSerializer.Serialize(new { ReleaseQuarantine = new { widget_id = widgetId } });
        return await SendRawCommandAsync(payload, ct).ConfigureAwait(false);
    }

    public async Task<string> ListWidgetsAsync(CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, "ListWidgets requested");
        var cmd = new ListWidgetsCommand();
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> UpdateWidgetDisplayOptionsAsync(
        string widgetId, double? opacity = null, double? scale = null,
        bool? locked = null, bool? enabled = null, CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, $"UpdateWidgetDisplayOptions: {widgetId} opacity={opacity} scale={scale} locked={locked} enabled={enabled}");
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
        DashboardLogger.Info(LogSource, $"QuickSwapWidget: {fromId} ↔ {toId} mode={mode}");
        var cmd = new QuickSwapWidgetCommand { FromId = fromId, ToId = toId, Mode = mode };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> EnableWidgetAsync(string widgetId, CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, $"EnableWidget: {widgetId}");
        var cmd = new EnableWidgetCommand { WidgetId = widgetId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> DisableWidgetAsync(string widgetId, CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, $"DisableWidget: {widgetId}");
        var cmd = new DisableWidgetCommand { WidgetId = widgetId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> SetWidgetOpacityAsync(string widgetId, double opacity, CancellationToken ct = default)
    {
        DashboardLogger.Debug(LogSource, $"SetWidgetOpacity: {widgetId} → {opacity:F2}");
        var cmd = new SetWidgetOpacityCommand
        {
            WidgetId = widgetId,
            Opacity = (float)Math.Clamp(opacity, 0.0, 1.0)
        };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    public async Task<string> ResetWidgetConfigAsync(string widgetId, CancellationToken ct = default)
    {
        DashboardLogger.Info(LogSource, $"ResetWidgetConfig: {widgetId}");
        var cmd = new ResetWidgetConfigCommand { WidgetId = widgetId };
        return await SendRawCommandAsync(cmd.ToJson(), ct).ConfigureAwait(false);
    }

    // ── Helpers ──────────────────────────────────────────────────────────────

    private void SetConnectionState(bool connected, string reason)
    {
        bool changed = _isConnected != connected;
        _isConnected = connected;
        if (changed)
        {
            DashboardLogger.Info(LogSource, $"Connection state → {(connected ? "CONNECTED" : "DISCONNECTED")} ({reason})");
        }
    }

    /// <summary>
    /// B16 Fix: Reliably detect error responses using JSON parsing instead of fragile string-contains.
    /// Falls back to string check if JSON parsing fails (e.g. for non-JSON responses).
    /// </summary>
    private static bool IsErrorResponse(string response)
    {
        if (string.IsNullOrWhiteSpace(response)) return true;

        try
        {
            using var doc = JsonDocument.Parse(response);
            if (doc.RootElement.TryGetProperty("status", out var statusProp))
            {
                string? status = statusProp.GetString();
                return string.Equals(status, "error", StringComparison.OrdinalIgnoreCase);
            }
            return false;
        }
        catch
        {
            // Fallback for non-JSON responses
            return response.Contains("\"status\"", StringComparison.OrdinalIgnoreCase) &&
                   response.Contains("\"error\"", StringComparison.OrdinalIgnoreCase);
        }
    }

    private static string Truncate(string value, int maxLength)
        => value.Length <= maxLength ? value : value[..maxLength] + "…";

    private static string EscapeJsonString(string value)
        => value.Replace("\\", "\\\\").Replace("\"", "\\\"").Replace("\n", "\\n").Replace("\r", "");
}
