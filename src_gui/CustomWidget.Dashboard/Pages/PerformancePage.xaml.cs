// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace CustomWidget.Dashboard.Pages;

/// <summary>
/// Performance page — real-time hardware telemetry charts backed by LiveChartsCore.
/// All data is REAL — sourced from the Rust daemon's SharedTelemetryCache.
/// </summary>
public sealed partial class PerformancePage : Page
{
    private readonly PerformanceViewModel _vm;
    private readonly DispatcherTimer _refreshTimer;

    public PerformancePage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<PerformanceViewModel>();

        // Bind chart series and axes
        CpuChart.Series = _vm.CpuSeries;
        CpuChart.YAxes = _vm.PercentYAxes;
        CpuChart.XAxes = _vm.HiddenXAxes;

        GpuChart.Series = _vm.GpuSeries;
        GpuChart.YAxes = _vm.PercentYAxes;
        GpuChart.XAxes = _vm.HiddenXAxes;

        RamChart.Series = _vm.RamSeries;
        RamChart.YAxes = _vm.RamYAxes;
        RamChart.XAxes = _vm.HiddenXAxes;

        NetChart.Series = _vm.NetSeries;
        NetChart.YAxes = _vm.NetYAxes;
        NetChart.XAxes = _vm.HiddenXAxes;

        // Update stat labels
        _refreshTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(500) };
        _refreshTimer.Tick += RefreshStats;
        _refreshTimer.Start();

        this.Unloaded += (_, _) => _refreshTimer.Stop();
    }

    private void RefreshStats(object? sender, object e)
    {
        CpuCurrentText.Text = _vm.CpuCurrent;
        CpuPeakText.Text = $"Peak: {_vm.CpuPeak}";
        CpuAvgText.Text = $"Avg: {_vm.CpuAverage}";

        GpuCurrentText.Text = _vm.GpuCurrent;
        GpuPeakText.Text = $"Peak: {_vm.GpuPeak}";
        GpuAvgText.Text = $"Avg: {_vm.GpuAverage}";

        RamCurrentText.Text = _vm.RamCurrent;
        RamPeakText.Text = $"Peak: {_vm.RamPeak}";
        RamTotalText.Text = $"Total: {_vm.RamTotal}";

        NetRecvText.Text = $"↓ {_vm.NetRecvCurrent}";
        NetSentText.Text = $"↑ {_vm.NetSentCurrent}";
    }
}
