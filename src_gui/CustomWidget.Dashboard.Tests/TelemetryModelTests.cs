// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Text.Json;
using CustomWidget.Dashboard.Models;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public class TelemetryModelTests
{
    [TestMethod]
    public void Test_EngineStatus_Deserializes_GpuAndCpuTopology()
    {
        string json = """
        {
            "status": "Running",
            "cpu_pct": 24.5,
            "gpu_pct": 65.0,
            "memory_used_mb": 4096.0,
            "memory_total_mb": 16384.0,
            "memory_free_mb": 12288.0,
            "active_widgets": ["hardware_pro", "audio_visualizer"],
            "engine_version": "0.7.0",
            "gpu_telemetry": {
                "gpu_index": 0,
                "adapter_name": "NVIDIA GeForce RTX 4090",
                "utilization_3d_pct": 72.5,
                "utilization_video_pct": 0.0,
                "utilization_copy_pct": 5.0,
                "vram_dedicated_used_mb": 8192.0,
                "vram_dedicated_total_mb": 24576.0,
                "vram_shared_used_mb": 1024.0,
                "temperature_c": 58.0,
                "fan_speed_pct": 45.0,
                "clock_core_mhz": 2520
            },
            "cpu_topology": {
                "physical_core_count": 16,
                "logical_core_count": 24,
                "p_core_count": 8,
                "e_core_count": 16,
                "is_thermal_throttling": false,
                "per_core_usage_pct": [30.0, 45.0, 12.0, 80.0]
            }
        }
        """;

        EngineStatus? status = JsonSerializer.Deserialize<EngineStatus>(json);
        Assert.IsNotNull(status);
        Assert.AreEqual(24.5f, status.CpuPct);
        Assert.IsNotNull(status.GpuTelemetry);
        Assert.AreEqual("NVIDIA GeForce RTX 4090", status.GpuTelemetry.AdapterName);
        Assert.AreEqual(8192.0f, status.GpuTelemetry.VramDedicatedUsedMb);
        Assert.IsNotNull(status.CpuTopology);
        Assert.AreEqual(8u, status.CpuTopology.PCoreCount);
        Assert.AreEqual(16u, status.CpuTopology.ECoreCount);
        Assert.AreEqual(4, status.CpuTopology.PerCoreUsagePct.Length);

        TelemetrySample sample = TelemetrySample.FromStatus(status);
        Assert.IsNotNull(sample.GpuTelemetry);
        Assert.AreEqual(8192.0f, sample.GpuTelemetry.VramDedicatedUsedMb);
    }

    [TestMethod]
    public void Test_EngineStatus_Deserializes_AudioSpectrumAndMedia()
    {
        string json = """
        {
            "status": "Running",
            "cpu_pct": 10.0,
            "gpu_pct": 5.0,
            "memory_used_mb": 2048.0,
            "memory_total_mb": 16384.0,
            "memory_free_mb": 14336.0,
            "active_widgets": ["audio_visualizer"],
            "engine_version": "0.7.0",
            "audio_spectrum": {
                "is_active": true,
                "peak_db_left": -6.2,
                "peak_db_right": -5.8,
                "master_volume_pct": 80.0,
                "fft_bins_16": [0.1, 0.3, 0.7, 0.9, 0.8, 0.6, 0.4, 0.2, 0.1, 0.05, 0.02, 0.01, 0.0, 0.0, 0.0, 0.0]
            },
            "media_playback": {
                "is_playing": true,
                "title": "Midnight City",
                "artist": "M83",
                "album": "Hurry Up, We're Dreaming",
                "playback_status": "Playing"
            }
        }
        """;

        EngineStatus? status = JsonSerializer.Deserialize<EngineStatus>(json);
        Assert.IsNotNull(status);
        Assert.IsNotNull(status.AudioSpectrum);
        Assert.IsTrue(status.AudioSpectrum.IsActive);
        Assert.AreEqual(16, status.AudioSpectrum.FftBins16.Length);
        Assert.AreEqual(0.9f, status.AudioSpectrum.FftBins16[3]);

        Assert.IsNotNull(status.MediaPlayback);
        Assert.IsTrue(status.MediaPlayback.IsPlaying);
        Assert.AreEqual("Midnight City", status.MediaPlayback.Title);
        Assert.AreEqual("M83", status.MediaPlayback.Artist);

        TelemetrySample sample = TelemetrySample.FromStatus(status);
        Assert.IsNotNull(sample.AudioSpectrum);
        Assert.AreEqual("Midnight City", sample.MediaPlayback?.Title);
    }
}
