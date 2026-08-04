// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Text.Json;
using System.Text.Json.Serialization;

namespace CustomWidget.Dashboard.Models;

/// <summary>
/// C# mirror of the Rust <c>ControlCommand</c> enum (JSON-tagged union).
/// Used to serialize IPC commands sent from the dashboard to the core engine.
/// </summary>
[JsonDerivedType(typeof(PingCommand), "Ping")]
[JsonDerivedType(typeof(GetStatusCommand), "GetStatus")]
[JsonDerivedType(typeof(ReloadAllCommand), "ReloadAll")]
[JsonDerivedType(typeof(LoadWidgetCommand), "LoadWidget")]
[JsonDerivedType(typeof(UnloadWidgetCommand), "UnloadWidget")]
[JsonDerivedType(typeof(SetThemeModeCommand), "SetThemeMode")]
public abstract class ControlCommandBase
{
    /// <summary>
    /// Serializes the command to the JSON format expected by the Rust serde deserializer.
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

public sealed class LoadWidgetCommand : ControlCommandBase
{
    public string ManifestPath { get; set; } = "";

    public override string ToJson()
    {
        var payload = new { LoadWidget = new { manifest_path = ManifestPath } };
        return JsonSerializer.Serialize(payload);
    }
}

public sealed class UnloadWidgetCommand : ControlCommandBase
{
    public string WidgetId { get; set; } = "";

    public override string ToJson()
    {
        var payload = new { UnloadWidget = new { widget_id = WidgetId } };
        return JsonSerializer.Serialize(payload);
    }
}

public sealed class SetThemeModeCommand : ControlCommandBase
{
    public string Mode { get; set; } = "dark";

    public override string ToJson()
    {
        var payload = new { SetThemeMode = new { mode = Mode } };
        return JsonSerializer.Serialize(payload);
    }
}

/// <summary>
/// IPC commands for the extended protocol (Phase I).
/// </summary>
public sealed class GetSubsystemHealthCommand : ControlCommandBase
{
    public override string ToJson() => "\"GetSubsystemHealth\"";
}

public sealed class GetDiagnosticsCommand : ControlCommandBase
{
    public override string ToJson() => "\"GetDiagnostics\"";
}
