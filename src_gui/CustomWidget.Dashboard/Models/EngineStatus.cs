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

    // ── Extended Hardware & Audio Telemetry Subsystems ──

    [JsonPropertyName("gpu_telemetry")]
    public GpuTelemetryDto? GpuTelemetry { get; set; }

    [JsonPropertyName("cpu_topology")]
    public CpuTopologyDto? CpuTopology { get; set; }

    [JsonPropertyName("audio_spectrum")]
    public AudioSpectrumDto? AudioSpectrum { get; set; }

    [JsonPropertyName("media_playback")]
    public MediaPlaybackDto? MediaPlayback { get; set; }

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

/// <summary>
/// Dedicated GPU telemetry DTO from native D3DKMT queries.
/// </summary>
public sealed class GpuTelemetryDto
{
    [JsonPropertyName("gpu_index")]
    public uint GpuIndex { get; set; }

    [JsonPropertyName("adapter_name")]
    public string AdapterName { get; set; } = "";

    [JsonPropertyName("utilization_3d_pct")]
    public float Utilization3dPct { get; set; }

    [JsonPropertyName("utilization_video_pct")]
    public float UtilizationVideoPct { get; set; }

    [JsonPropertyName("utilization_copy_pct")]
    public float UtilizationCopyPct { get; set; }

    [JsonPropertyName("vram_dedicated_used_mb")]
    public float VramDedicatedUsedMb { get; set; }

    [JsonPropertyName("vram_dedicated_total_mb")]
    public float VramDedicatedTotalMb { get; set; }

    [JsonPropertyName("vram_shared_used_mb")]
    public float VramSharedUsedMb { get; set; }

    [JsonPropertyName("temperature_c")]
    public float TemperatureC { get; set; }

    [JsonPropertyName("fan_speed_pct")]
    public float FanSpeedPct { get; set; }

    [JsonPropertyName("clock_core_mhz")]
    public uint ClockCoreMhz { get; set; }
}

/// <summary>
/// Per-core CPU topology and hybrid architecture DTO.
/// </summary>
public sealed class CpuTopologyDto
{
    [JsonPropertyName("physical_core_count")]
    public uint PhysicalCoreCount { get; set; }

    [JsonPropertyName("logical_core_count")]
    public uint LogicalCoreCount { get; set; }

    [JsonPropertyName("p_core_count")]
    public uint PCoreCount { get; set; }

    [JsonPropertyName("e_core_count")]
    public uint ECoreCount { get; set; }

    [JsonPropertyName("is_thermal_throttling")]
    public bool IsThermalThrottling { get; set; }

    [JsonPropertyName("per_core_usage_pct")]
    public float[] PerCoreUsagePct { get; set; } = [];
}

/// <summary>
/// 16-band WASAPI audio FFT frequency spectrum DTO.
/// </summary>
public sealed class AudioSpectrumDto
{
    [JsonPropertyName("is_active")]
    public bool IsActive { get; set; }

    [JsonPropertyName("peak_db_left")]
    public float PeakDbLeft { get; set; }

    [JsonPropertyName("peak_db_right")]
    public float PeakDbRight { get; set; }

    [JsonPropertyName("master_volume_pct")]
    public float MasterVolumePct { get; set; }

    [JsonPropertyName("fft_bins_16")]
    public float[] FftBins16 { get; set; } = [];
}

/// <summary>
/// Windows System Media Transport Controls (SMTC) playback DTO.
/// </summary>
public sealed class MediaPlaybackDto
{
    [JsonPropertyName("is_playing")]
    public bool IsPlaying { get; set; }

    [JsonPropertyName("title")]
    public string Title { get; set; } = "";

    [JsonPropertyName("artist")]
    public string Artist { get; set; } = "";

    [JsonPropertyName("album")]
    public string Album { get; set; } = "";

    [JsonPropertyName("playback_status")]
    public string PlaybackStatus { get; set; } = "";
}

