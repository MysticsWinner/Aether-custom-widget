// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Text.Json.Serialization;

namespace CustomWidget.Dashboard.Models;

/// <summary>
/// Mirrors the Rust <c>ipc_server::StatusResponse</c> returned by the <c>GetStatus</c> IPC command.
/// All telemetry values are real, sourced from the <c>SharedTelemetryCache</c>.
/// </summary>
public sealed class EngineStatus
{
    [JsonPropertyName("status")]
    public string Status { get; set; } = "";

    [JsonPropertyName("cpu_pct")]
    public float CpuPct { get; set; }

    [JsonPropertyName("gpu_pct")]
    public float GpuPct { get; set; }

    [JsonPropertyName("memory_used_mb")]
    public float MemoryUsedMb { get; set; }

    [JsonPropertyName("memory_total_mb")]
    public float MemoryTotalMb { get; set; }

    [JsonPropertyName("memory_free_mb")]
    public float MemoryFreeMb { get; set; }

    [JsonPropertyName("active_widgets")]
    public string[] ActiveWidgets { get; set; } = [];

    [JsonPropertyName("engine_version")]
    public string EngineVersion { get; set; } = "";

    // ── Extended fields (from future GetSubsystemHealth / GetDiagnostics) ──

    [JsonPropertyName("subsystems")]
    public SubsystemEntry[]? Subsystems { get; set; }

    [JsonPropertyName("pid")]
    public uint? Pid { get; set; }

    [JsonPropertyName("uptime_secs")]
    public ulong? UptimeSecs { get; set; }

    [JsonPropertyName("tick_count")]
    public ulong? TickCount { get; set; }

    /// <summary>
    /// Convenience: memory usage as a percentage (0.0–100.0).
    /// </summary>
    [JsonIgnore]
    public float MemoryPct => MemoryTotalMb > 0 ? (MemoryUsedMb / MemoryTotalMb) * 100f : 0f;
}

/// <summary>
/// Subsystem health entry within an extended status response.
/// </summary>
public sealed class SubsystemEntry
{
    [JsonPropertyName("name")]
    public string Name { get; set; } = "";

    [JsonPropertyName("health")]
    public string Health { get; set; } = "Healthy";
}
