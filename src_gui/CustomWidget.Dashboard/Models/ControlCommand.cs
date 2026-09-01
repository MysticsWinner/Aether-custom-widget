// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace CustomWidget.Dashboard.Models;

/// <summary>
/// C# mirror of the Rust <c>ControlCommand</c> enum (externally tagged union in serde_json).
/// Used to serialize IPC commands sent from the dashboard to the core engine.
/// </summary>
public abstract class ControlCommandBase
{
    /// <summary>
    /// Serializes the command to the exact JSON format expected by the Rust serde deserializer.
    /// Rust's serde uses externally tagged enum format by default.
    /// </summary>
    public abstract string ToJson();
}

public sealed class PingCommand : ControlCommandBase
{
    public override string ToJson() => "\"Ping\"";
}

public sealed class GetStatusCommand : ControlCommandBase
{
    public override string ToJson() => "\"GetStatus\"";
}

public sealed class ReloadAllCommand : ControlCommandBase
{
    public override string ToJson() => "\"ReloadAll\"";
}

public sealed class ToggleDesktopWidgetCommand : ControlCommandBase
{
    public override string ToJson() => "\"ToggleDesktopWidget\"";
}

public sealed class GetSubsystemHealthCommand : ControlCommandBase
{
    public override string ToJson() => "\"GetSubsystemHealth\"";
}

public sealed class GetDiagnosticsCommand : ControlCommandBase
{
    public override string ToJson() => "\"GetDiagnostics\"";
}

public sealed class GetHealthReportCommand : ControlCommandBase
{
    public override string ToJson() => "\"GetHealthReport\"";
}

public sealed class GetWatchdogStatusCommand : ControlCommandBase
{
    public override string ToJson() => "\"GetWatchdogStatus\"";
}

public sealed class GetLaunchModeCommand : ControlCommandBase
{
    public override string ToJson() => "\"GetLaunchMode\"";
}

public sealed class ExitSafeModeCommand : ControlCommandBase
{
    public override string ToJson() => "\"ExitSafeMode\"";
}

public sealed class GetQuarantineListCommand : ControlCommandBase
{
    public override string ToJson() => "\"GetQuarantineList\"";
}

public sealed class ListSnapshotsCommand : ControlCommandBase
{
    public override string ToJson() => "\"ListSnapshots\"";
}

public sealed class ListWidgetsCommand : ControlCommandBase
{
    public override string ToJson() => "\"ListWidgets\"";
}

public sealed class GetAuditLogsCommand : ControlCommandBase
{
    public override string ToJson() => "\"GetAuditLogs\"";
}

public sealed class VerifyAuditChainCommand : ControlCommandBase
{
    public override string ToJson() => "\"VerifyAuditChain\"";
}

public sealed class LoadWidgetCommand : ControlCommandBase
{
    [JsonPropertyName("manifest_path")]
    public string ManifestPath { get; set; } = "";

    public override string ToJson()
        => JsonSerializer.Serialize(new { LoadWidget = new { manifest_path = ManifestPath } });
}

public sealed class UnloadWidgetCommand : ControlCommandBase
{
    [JsonPropertyName("widget_id")]
    public string WidgetId { get; set; } = "";

    public override string ToJson()
        => JsonSerializer.Serialize(new { UnloadWidget = new { widget_id = WidgetId } });
}

public sealed class SetThemeModeCommand : ControlCommandBase
{
    [JsonPropertyName("mode")]
    public string Mode { get; set; } = "dark";

    public override string ToJson()
        => JsonSerializer.Serialize(new { SetThemeMode = new { mode = Mode } });
}

public sealed class SetWidgetPositionCommand : ControlCommandBase
{
    public string WidgetId { get; set; } = "";
    public int X { get; set; }
    public int Y { get; set; }

    public override string ToJson()
        => JsonSerializer.Serialize(new { SetWidgetPosition = new { widget_id = WidgetId, x = X, y = Y } });
}

public sealed class SetWidgetLockCommand : ControlCommandBase
{
    public string WidgetId { get; set; } = "";
    public bool Locked { get; set; }

    public override string ToJson()
        => JsonSerializer.Serialize(new { SetWidgetLock = new { widget_id = WidgetId, locked = Locked } });
}

public sealed class ToggleWidgetLockCommand : ControlCommandBase
{
    public string WidgetId { get; set; } = "";

    public override string ToJson()
        => JsonSerializer.Serialize(new { ToggleWidgetLock = new { widget_id = WidgetId } });
}

public sealed class CreateSnapshotCommand : ControlCommandBase
{
    public string Name { get; set; } = "";

    public override string ToJson()
        => JsonSerializer.Serialize(new { CreateSnapshot = new { name = Name } });
}

public sealed class RestoreSnapshotCommand : ControlCommandBase
{
    public string SnapshotId { get; set; } = "";

    public override string ToJson()
        => JsonSerializer.Serialize(new { RestoreSnapshot = new { snapshot_id = SnapshotId } });
}

public sealed class DeleteSnapshotCommand : ControlCommandBase
{
    public string SnapshotId { get; set; } = "";

    public override string ToJson()
        => JsonSerializer.Serialize(new { DeleteSnapshot = new { snapshot_id = SnapshotId } });
}

public sealed class ExportSnapshotCommand : ControlCommandBase
{
    public string SnapshotId { get; set; } = "";
    public string Path { get; set; } = "";

    public override string ToJson()
        => JsonSerializer.Serialize(new { ExportSnapshot = new { snapshot_id = SnapshotId, path = Path } });
}

public sealed class ImportSnapshotCommand : ControlCommandBase
{
    public string Path { get; set; } = "";

    public override string ToJson()
        => JsonSerializer.Serialize(new { ImportSnapshot = new { path = Path } });
}

public sealed class DiscoverWidgetsCommand : ControlCommandBase
{
    public List<string>? SearchPaths { get; set; }

    public override string ToJson()
        => JsonSerializer.Serialize(new { DiscoverWidgets = new { search_paths = SearchPaths } });
}

public sealed class SearchMarketplaceCommand : ControlCommandBase
{
    public string Query { get; set; } = "";
    public string Category { get; set; } = "all";

    public override string ToJson()
        => JsonSerializer.Serialize(new { SearchMarketplace = new { query = Query, category = Category } });
}

public sealed class UpdateWidgetDisplayOptionsCommand : ControlCommandBase
{
    public string WidgetId { get; set; } = "";
    public double? Opacity { get; set; }
    public double? Scale { get; set; }
    public bool? Locked { get; set; }
    public bool? Enabled { get; set; }

    public override string ToJson()
        => JsonSerializer.Serialize(new { UpdateWidgetDisplayOptions = new { widget_id = WidgetId, opacity = Opacity, scale = Scale, locked = Locked, enabled = Enabled } });
}

public sealed class QuickSwapWidgetCommand : ControlCommandBase
{
    public string FromId { get; set; } = "";
    public string ToId { get; set; } = "";
    public string Mode { get; set; } = "position";

    public override string ToJson()
        => JsonSerializer.Serialize(new { QuickSwapWidget = new { from_id = FromId, to_id = ToId, mode = Mode } });
}

public sealed class EnableWidgetCommand : ControlCommandBase
{
    public string WidgetId { get; set; } = "";

    public override string ToJson()
        => JsonSerializer.Serialize(new { EnableWidget = new { widget_id = WidgetId } });
}

public sealed class DisableWidgetCommand : ControlCommandBase
{
    public string WidgetId { get; set; } = "";

    public override string ToJson()
        => JsonSerializer.Serialize(new { DisableWidget = new { widget_id = WidgetId } });
}

public sealed class SetWidgetOpacityCommand : ControlCommandBase
{
    public string WidgetId { get; set; } = "";
    public float Opacity { get; set; } = 1.0f;

    public override string ToJson()
        => JsonSerializer.Serialize(new { SetWidgetOpacity = new { widget_id = WidgetId, opacity = Opacity } });
}

public sealed class ResetWidgetConfigCommand : ControlCommandBase
{
    public string WidgetId { get; set; } = "";

    public override string ToJson()
        => JsonSerializer.Serialize(new { ResetWidgetConfig = new { widget_id = WidgetId } });
}
