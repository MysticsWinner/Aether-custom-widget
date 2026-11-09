// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Threading.Tasks;
using CustomWidget.SDK;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public sealed class CSharpSdkTests
{
    private sealed class SampleManagedWidget : IWidget
    {
        public WidgetState State { get; private set; } = WidgetState.Unloaded;
        public string? LastEventTopic { get; private set; }

        public Task OnLoadAsync()
        {
            State = WidgetState.Loaded;
            return Task.CompletedTask;
        }

        public Task OnMountAsync()
        {
            State = WidgetState.Active;
            return Task.CompletedTask;
        }

        public void OnUpdate(in TickContext context, IRenderCanvas canvas)
        {
            canvas.Clear(Color.Rgba(15, 23, 42, 220));
            canvas.DrawRect(new RectF(0, 0, 320, 180), Color.Rgba(30, 41, 59, 240), 12.0f);
            canvas.DrawText($"Frame #{context.FrameIndex}", "Segoe UI", 14.0f, new RectF(16, 16, 200, 24), Color.Rgba(0, 212, 245, 255));
            canvas.Invalidate(new RectF(0, 0, 320, 180));
        }

        public Task OnUnmountAsync()
        {
            State = WidgetState.Unmounted;
            return Task.CompletedTask;
        }

        public Task OnUnloadAsync()
        {
            State = WidgetState.Unloaded;
            return Task.CompletedTask;
        }

        public void OnEvent(string topic, string payload)
        {
            LastEventTopic = $"{topic}:{payload}";
        }
    }

    [TestMethod]
    public async Task TestCSharpWidgetSdkLifecycleAndBatchRenderCanvas()
    {
        var widget = new SampleManagedWidget();
        Assert.AreEqual(WidgetState.Unloaded, widget.State);

        await widget.OnLoadAsync();
        Assert.AreEqual(WidgetState.Loaded, widget.State);

        await widget.OnMountAsync();
        Assert.AreEqual(WidgetState.Active, widget.State);

        var canvas = new BatchRenderCanvas();
        var ctx = new TickContext(1000, 16.6f, 42);
        widget.OnUpdate(in ctx, canvas);

        Assert.AreEqual(3, canvas.Commands.Count);
        Assert.AreEqual("Clear", canvas.Commands[0].Kind);
        Assert.AreEqual("DrawRect", canvas.Commands[1].Kind);
        Assert.AreEqual("DrawText", canvas.Commands[2].Kind);
        Assert.AreEqual("Frame #42", canvas.Commands[2].Text);
        Assert.AreEqual(1, canvas.InvalidatedRegions.Count);

        string json = canvas.ToJsonBatch();
        StringAssert.Contains(json, "Frame #42");

        widget.OnEvent("theme.changed", "cyberpunk");
        Assert.AreEqual("theme.changed:cyberpunk", widget.LastEventTopic);

        await widget.OnUnmountAsync();
        Assert.AreEqual(WidgetState.Unmounted, widget.State);

        await widget.OnUnloadAsync();
        Assert.AreEqual(WidgetState.Unloaded, widget.State);
    }

    [TestMethod]
    public void TestInMemorySettingsStoreAndWidgetStateEnumCount()
    {
        var store = new InMemorySettingsStore();
        Assert.IsNull(store.GetSetting("opacity"));

        store.SetSetting("Opacity", 0.92);
        Assert.AreEqual(0.92, store.GetSetting("opacity"));

        // TD-006: Ensure C# WidgetState enum matches all 13 Rust WidgetState variants
        var values = System.Enum.GetValues<WidgetState>();
        Assert.AreEqual(13, values.Length);
    }

    [TestMethod]
    public void TestCanvasWidgetBoxMagneticSnapResizeAndAlignmentGuides()
    {
        var (sx, sy) = CustomWidget.Dashboard.Models.CanvasWidgetBox.SnapToGrid(103, 199, 16);
        Assert.AreEqual(96, sx);
        Assert.AreEqual(192, sy);

        var box1 = new CustomWidget.Dashboard.Models.CanvasWidgetBox
        {
            WidgetId = "perf_monitor_widget",
            DisplayName = "Perf Monitor",
            X = 128,
            Y = 96,
            Width = 320,
            Height = 200
        };
        box1.ApplyResize(CustomWidget.Dashboard.Models.ResizeHandle.BottomRight, 48, 32);
        Assert.AreEqual(368, box1.Width);
        Assert.AreEqual(232, box1.Height);

        // Enforce min width (120) and min height (80)
        box1.ApplyResize(CustomWidget.Dashboard.Models.ResizeHandle.BottomRight, -1000, -1000);
        Assert.AreEqual(120, box1.Width);
        Assert.AreEqual(80, box1.Height);

        var peer = new CustomWidget.Dashboard.Models.CanvasWidgetBox
        {
            WidgetId = "weather_widget",
            DisplayName = "Weather",
            X = 130, // within 8px threshold of 128
            Y = 400,
            Width = 120,
            Height = 160
        };
        var guides = box1.ComputeSnapGuides(new[] { box1, peer });
        Assert.IsTrue(guides.Count >= 1);
        StringAssert.Contains(guides[0], "AlignLeft:weather_widget");
    }

    [TestMethod]
    public async Task TestMarketplaceReviewSubmissionUpdatesWeightedRating()
    {
        var pkg = new CustomWidget.Dashboard.ViewModels.MarketplacePackageItem
        {
            Id = "com.aether.weather-radar",
            Name = "Aero Weather Radar HD",
            Rating = 4.0,
            ReviewCount = 1,
            CrashFreeRatePct = 99.9
        };
        Assert.AreEqual("99.9% Crash-Free", pkg.CrashFreeText);

        var vm = new CustomWidget.Dashboard.ViewModels.MarketplaceViewModel(null!);
        await vm.SubmitReviewAsync((pkg, 5, "Awesome radar animations!"));
        Assert.AreEqual(2, pkg.ReviewCount);
        Assert.AreEqual(4.5, pkg.Rating);
    }
}

