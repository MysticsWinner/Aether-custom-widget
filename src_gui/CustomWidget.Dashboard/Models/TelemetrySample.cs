// Copyright (c) Aether Platform. Licensed under the MIT License.

namespace CustomWidget.Dashboard.Models;

/// <summary>
/// A timestamped telemetry sample captured from the Aether core engine via IPC.
/// These are 100% real values from Windows APIs — no synthetic/fake data.
/// </summary>
public sealed class TelemetrySample
{
    public DateTime Timestamp { get; init; } = DateTime.Now;
    public float CpuPct { get; init; }
    public float GpuPct { get; init; }
    public float MemoryUsedMb { get; init; }
    public float MemoryTotalMb { get; init; }
    public float MemoryFreeMb { get; init; }
    public ulong NetRecvBytesPerSec { get; init; }
    public ulong NetSentBytesPerSec { get; init; }

    /// <summary>
    /// Memory usage as a percentage (0.0–100.0).
    /// </summary>
    public float MemoryPct => MemoryTotalMb > 0 ? (MemoryUsedMb / MemoryTotalMb) * 100f : 0f;

    /// <summary>
    /// Memory used in GB for display.
    /// </summary>
    public float MemoryUsedGb => MemoryUsedMb / 1024f;

    /// <summary>
    /// Memory total in GB for display.
    /// </summary>
    public float MemoryTotalGb => MemoryTotalMb / 1024f;

    /// <summary>
    /// Creates a TelemetrySample from an EngineStatus IPC response.
    /// </summary>
    public static TelemetrySample FromStatus(EngineStatus status) => new()
    {
        Timestamp = DateTime.Now,
        CpuPct = status.CpuPct,
        GpuPct = status.GpuPct,
        MemoryUsedMb = status.MemoryUsedMb,
        MemoryTotalMb = status.MemoryTotalMb,
        MemoryFreeMb = status.MemoryFreeMb,
    };
}
