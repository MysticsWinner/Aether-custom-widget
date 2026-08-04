// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.Services;
using LiveChartsCore;
using LiveChartsCore.Defaults;
using LiveChartsCore.SkiaSharpView;
using LiveChartsCore.SkiaSharpView.Painting;
using SkiaSharp;

namespace CustomWidget.Dashboard.ViewModels;

/// <summary>
/// ViewModel for the Performance page — real-time charts showing CPU, GPU, RAM, and network.
/// Uses LiveChartsCore with rolling 60-second windows backed by real IPC telemetry data.
/// </summary>
public partial class PerformanceViewModel : ObservableObject
{
    private readonly TelemetryPollerService _poller;
    private const int MaxPoints = 120; // 60 seconds at 500ms interval

    // ── Observable chart data collections ──
    private readonly ObservableCollection<ObservableValue> _cpuValues = new();
    private readonly ObservableCollection<ObservableValue> _gpuValues = new();
    private readonly ObservableCollection<ObservableValue> _ramValues = new();
    private readonly ObservableCollection<ObservableValue> _netRecvValues = new();
    private readonly ObservableCollection<ObservableValue> _netSentValues = new();

    // ── Summary stats ──
    [ObservableProperty] private string _cpuCurrent = "0.0%";
    [ObservableProperty] private string _cpuPeak = "0.0%";
    [ObservableProperty] private string _cpuAverage = "0.0%";

    [ObservableProperty] private string _gpuCurrent = "0.0%";
    [ObservableProperty] private string _gpuPeak = "0.0%";
    [ObservableProperty] private string _gpuAverage = "0.0%";

    [ObservableProperty] private string _ramCurrent = "0.0 GB";
    [ObservableProperty] private string _ramPeak = "0.0 GB";
    [ObservableProperty] private string _ramTotal = "0.0 GB";

    [ObservableProperty] private string _netRecvCurrent = "0 B/s";
    [ObservableProperty] private string _netSentCurrent = "0 B/s";

    // ── Chart Series (bound to CartesianChart.Series in XAML) ──

    public ISeries[] CpuSeries { get; }
    public ISeries[] GpuSeries { get; }
    public ISeries[] RamSeries { get; }
    public ISeries[] NetSeries { get; }

    // ── Axis configs ──

    public Axis[] PercentYAxes { get; } = [
        new Axis { MinLimit = 0, MaxLimit = 100, Name = "%",
            NamePaint = new SolidColorPaint(SKColors.Gray),
            LabelsPaint = new SolidColorPaint(SKColors.Gray) }
    ];

    public Axis[] RamYAxes { get; } = [
        new Axis { MinLimit = 0, Name = "GB",
            NamePaint = new SolidColorPaint(SKColors.Gray),
            LabelsPaint = new SolidColorPaint(SKColors.Gray) }
    ];

    public Axis[] NetYAxes { get; } = [
        new Axis { MinLimit = 0, Name = "KB/s",
            NamePaint = new SolidColorPaint(SKColors.Gray),
            LabelsPaint = new SolidColorPaint(SKColors.Gray) }
    ];

    public Axis[] HiddenXAxes { get; } = [
        new Axis { IsVisible = false }
    ];

    public PerformanceViewModel(TelemetryPollerService poller)
    {
        _poller = poller;

        CpuSeries = [CreateLineSeries(_cpuValues, new SKColor(0, 212, 255), "CPU %")];
        GpuSeries = [CreateLineSeries(_gpuValues, new SKColor(224, 64, 251), "GPU %")];
        RamSeries = [CreateLineSeries(_ramValues, new SKColor(105, 240, 174), "RAM GB")];
        NetSeries = [
            CreateLineSeries(_netRecvValues, new SKColor(255, 215, 64), "↓ Recv"),
            CreateLineSeries(_netSentValues, new SKColor(255, 138, 101), "↑ Sent"),
        ];

        _poller.OnNewSample += OnNewSample;
    }

    private void OnNewSample(TelemetrySample sample)
    {
        // Add new data points
        AddPoint(_cpuValues, sample.CpuPct);
        AddPoint(_gpuValues, sample.GpuPct);
        AddPoint(_ramValues, sample.MemoryUsedGb);
        AddPoint(_netRecvValues, sample.NetRecvBytesPerSec / 1024.0); // Convert to KB/s
        AddPoint(_netSentValues, sample.NetSentBytesPerSec / 1024.0);

        // Update summary stats
        UpdateCpuStats(sample);
        UpdateGpuStats(sample);
        UpdateRamStats(sample);
        UpdateNetStats(sample);
    }

    private static void AddPoint(ObservableCollection<ObservableValue> values, double newValue)
    {
        values.Add(new ObservableValue(newValue));
        while (values.Count > MaxPoints)
            values.RemoveAt(0);
    }

    private void UpdateCpuStats(TelemetrySample sample)
    {
        CpuCurrent = $"{sample.CpuPct:F1}%";
        float peak = _cpuValues.Count > 0 ? (float)_cpuValues.Max(v => v.Value ?? 0) : 0;
        float avg = _cpuValues.Count > 0 ? (float)_cpuValues.Average(v => v.Value ?? 0) : 0;
        CpuPeak = $"{peak:F1}%";
        CpuAverage = $"{avg:F1}%";
    }

    private void UpdateGpuStats(TelemetrySample sample)
    {
        GpuCurrent = $"{sample.GpuPct:F1}%";
        float peak = _gpuValues.Count > 0 ? (float)_gpuValues.Max(v => v.Value ?? 0) : 0;
        float avg = _gpuValues.Count > 0 ? (float)_gpuValues.Average(v => v.Value ?? 0) : 0;
        GpuPeak = $"{peak:F1}%";
        GpuAverage = $"{avg:F1}%";
    }

    private void UpdateRamStats(TelemetrySample sample)
    {
        RamCurrent = $"{sample.MemoryUsedGb:F1} GB";
        float peak = _ramValues.Count > 0 ? (float)_ramValues.Max(v => v.Value ?? 0) : 0;
        RamPeak = $"{peak:F1} GB";
        RamTotal = $"{sample.MemoryTotalGb:F1} GB";
    }

    private void UpdateNetStats(TelemetrySample sample)
    {
        NetRecvCurrent = FormatBytes(sample.NetRecvBytesPerSec);
        NetSentCurrent = FormatBytes(sample.NetSentBytesPerSec);
    }

    private static string FormatBytes(ulong bytes) => bytes switch
    {
        >= 1_048_576 => $"{bytes / 1_048_576.0:F1} MB/s",
        >= 1_024 => $"{bytes / 1_024.0:F1} KB/s",
        _ => $"{bytes} B/s",
    };

    private static LineSeries<ObservableValue> CreateLineSeries(
        ObservableCollection<ObservableValue> values, SKColor color, string name) => new()
    {
        Values = values,
        Name = name,
        Stroke = new SolidColorPaint(color, 2),
        GeometryStroke = null,
        GeometryFill = null,
        GeometrySize = 0,
        Fill = new SolidColorPaint(color.WithAlpha(30)),
        LineSmoothness = 0.65,
        AnimationsSpeed = TimeSpan.FromMilliseconds(150),
    };
}
