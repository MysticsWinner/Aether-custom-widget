// Copyright (c) Aether Platform. Licensed under the MIT License.

namespace CustomWidget.Dashboard.Models;

/// <summary>
/// Subsystem health information for display in the Services page.
/// </summary>
public sealed class SubsystemInfo
{
    /// <summary>
    /// Subsystem identifier — e.g. "gpu_render_engine", "telemetry_subsystem".
    /// </summary>
    public string Name { get; init; } = "";

    /// <summary>
    /// Health status: "Healthy", "Degraded", or "Failed".
    /// </summary>
    public string Health { get; init; } = "Healthy";

    /// <summary>
    /// Human-readable description of the subsystem.
    /// </summary>
    public string Description { get; init; } = "";

    /// <summary>
    /// Display-friendly name derived from the identifier.
    /// </summary>
    public string DisplayName => Name.Replace("_", " ")
        .Replace("subsystem", "")
        .Trim();
}
