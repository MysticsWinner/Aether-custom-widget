// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Text.Json.Serialization;

namespace CustomWidget.Dashboard.Models;

/// <summary>
/// Detailed widget metadata for display in the Widgets management page & discovered plugins library.
/// </summary>
public sealed class WidgetInfo
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = "";

    [JsonPropertyName("name")]
    public string Name { get; set; } = "";

    [JsonPropertyName("author")]
    public string Author { get; set; } = "Community";

    [JsonPropertyName("version")]
    public string Version { get; set; } = "1.0.0";

    [JsonPropertyName("update_interval_ms")]
    public ulong UpdateIntervalMs { get; set; } = 500;

    [JsonPropertyName("manifest_path")]
    public string ManifestPath { get; set; } = "";

    [JsonPropertyName("folder_path")]
    public string FolderPath { get; set; } = "";

    [JsonPropertyName("is_loaded")]
    public bool IsLoaded { get; set; }

    [JsonPropertyName("is_locked")]
    public bool IsLocked { get; set; }

    [JsonPropertyName("position_x")]
    public int PositionX { get; set; } = 100;

    [JsonPropertyName("position_y")]
    public int PositionY { get; set; } = 100;

    [JsonPropertyName("target_fps")]
    public uint TargetFps { get; set; } = 60;

    [JsonPropertyName("description")]
    public string Description { get; set; } = "";

    public string State => IsLoaded ? "Loaded" : "Available";

    public bool IsActive => IsLoaded;
}

public sealed class DiscoverWidgetsResponse
{
    [JsonPropertyName("status")]
    public string Status { get; set; } = "";

    [JsonPropertyName("discovered_widgets")]
    public List<WidgetInfo> DiscoveredWidgets { get; set; } = new();
}
