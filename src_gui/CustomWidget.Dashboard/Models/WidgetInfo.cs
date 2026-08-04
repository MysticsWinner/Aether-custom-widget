// Copyright (c) Aether Platform. Licensed under the MIT License.

namespace CustomWidget.Dashboard.Models;

/// <summary>
/// Widget metadata for display in the Widgets management page.
/// </summary>
public sealed class WidgetInfo
{
    /// <summary>
    /// Widget identifier — e.g. "aether.builtin.perf_monitor".
    /// </summary>
    public string Id { get; init; } = "";

    /// <summary>
    /// Human-readable widget name.
    /// </summary>
    public string Name { get; init; } = "";

    /// <summary>
    /// Current lifecycle state: "Loaded", "Mounted", "Unloaded", "Error".
    /// </summary>
    public string State { get; init; } = "Loaded";

    /// <summary>
    /// Path to the widget manifest (widget.toml).
    /// </summary>
    public string ManifestPath { get; init; } = "";

    /// <summary>
    /// True if the widget is currently active (loaded + mounted).
    /// </summary>
    public bool IsActive => State is "Loaded" or "Mounted";
}
