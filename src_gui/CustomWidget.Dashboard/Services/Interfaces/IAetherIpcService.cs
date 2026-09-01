// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;
using CustomWidget.Dashboard.Models;

namespace CustomWidget.Dashboard.Services.Interfaces;

/// <summary>
/// Service abstraction for IPC communication with the Aether Core Engine.
/// </summary>
public interface IAetherIpcService
{
    bool IsConnected { get; }
    string LastEngineVersion { get; }

    Task<EngineStatus?> GetStatusAsync(CancellationToken ct = default);
    Task<bool> PingAsync(CancellationToken ct = default);
    Task<string> SendRawCommandAsync(string commandJson, CancellationToken ct = default);

    Task<string> LoadWidgetAsync(string manifestPath, CancellationToken ct = default);
    Task<string> UnloadWidgetAsync(string widgetId, CancellationToken ct = default);
    Task<string> SetThemeModeAsync(string mode, CancellationToken ct = default);
    Task<string> ReloadAllAsync(CancellationToken ct = default);
    Task<string> ToggleDesktopWidgetAsync(CancellationToken ct = default);

    Task<string> SetWidgetPositionAsync(string widgetId, int x, int y, CancellationToken ct = default);
    Task<string> SetWidgetLockAsync(string widgetId, bool locked, CancellationToken ct = default);
    Task<string> ToggleWidgetLockAsync(string widgetId, CancellationToken ct = default);

    Task<List<WidgetInfo>> DiscoverWidgetsAsync(List<string>? searchPaths = null, CancellationToken ct = default);
    Task<string> SearchMarketplaceAsync(string query, string? category = null, CancellationToken ct = default);

    Task<string> CreateSnapshotAsync(string name, CancellationToken ct = default);
    Task<string> ListSnapshotsAsync(CancellationToken ct = default);
    Task<string> RestoreSnapshotAsync(string snapshotId, CancellationToken ct = default);
    Task<string> DeleteSnapshotAsync(string snapshotId, CancellationToken ct = default);
    Task<string> ExportSnapshotAsync(string snapshotId, string path, CancellationToken ct = default);
    Task<string> ImportSnapshotAsync(string path, CancellationToken ct = default);

    Task<string> GetSecurityAuditLogsAsync(CancellationToken ct = default);
    Task<string> GetDiagnosticsAsync(CancellationToken ct = default);
    Task<string> GetSubsystemHealthAsync(CancellationToken ct = default);
    Task<string> GetCrashHistoryAsync(string? widgetId = null, CancellationToken ct = default);
    Task<string> GetLaunchModeAsync(CancellationToken ct = default);
    Task<string> ExitSafeModeAsync(CancellationToken ct = default);
    Task<string> GetQuarantineListAsync(CancellationToken ct = default);
    Task<string> ReleaseQuarantineAsync(string widgetId, CancellationToken ct = default);

    Task<string> ListWidgetsAsync(CancellationToken ct = default);
    Task<string> UpdateWidgetDisplayOptionsAsync(string widgetId, double? opacity = null, double? scale = null, bool? locked = null, bool? enabled = null, CancellationToken ct = default);
    Task<string> QuickSwapWidgetAsync(string fromId, string toId, string mode = "position", CancellationToken ct = default);
    Task<string> EnableWidgetAsync(string widgetId, CancellationToken ct = default);
    Task<string> DisableWidgetAsync(string widgetId, CancellationToken ct = default);
    Task<string> SetWidgetOpacityAsync(string widgetId, double opacity, CancellationToken ct = default);
    Task<string> ResetWidgetConfigAsync(string widgetId, CancellationToken ct = default);
}
