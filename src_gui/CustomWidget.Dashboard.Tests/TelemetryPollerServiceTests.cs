// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public class TelemetryPollerServiceTests
{
    private AetherIpcService _ipc = null!;
    private TelemetryPollerService _poller = null!;

    [TestInitialize]
    public void Setup()
    {
        _ipc = new AetherIpcService();
        _poller = new TelemetryPollerService(_ipc);
    }

    [TestMethod]
    public void Test_TelemetryPollerService_DefaultPollInterval_Is500ms()
    {
        Assert.AreEqual(500, _poller.PollIntervalMs);
    }

    [TestMethod]
    public void Test_TelemetryPollerService_SetWindowVisibility_AdjustsInterval()
    {
        // When minimized/hidden: throttle to 5000ms
        _poller.SetWindowVisibility(false);
        Assert.AreEqual(5000, _poller.PollIntervalMs);

        // When restored/visible: return to normal 500ms
        _poller.SetWindowVisibility(true);
        Assert.AreEqual(500, _poller.PollIntervalMs);
    }

    [TestMethod]
    public void Test_TelemetryPollerService_SetPowerState_AdjustsInterval()
    {
        // When running on battery: throttle to 2000ms
        _poller.SetPowerState(true);
        Assert.AreEqual(2000, _poller.PollIntervalMs);

        // When connected to AC power: return to 500ms
        _poller.SetPowerState(false);
        Assert.AreEqual(500, _poller.PollIntervalMs);
    }

    [TestMethod]
    public void Test_TelemetryPollerService_SetThrottleState_AdjustsInterval()
    {
        _poller.SetThrottleState(true);
        Assert.AreEqual(2000, _poller.PollIntervalMs);

        _poller.SetThrottleState(false);
        Assert.AreEqual(500, _poller.PollIntervalMs);
    }

    [TestMethod]
    public void Test_TelemetryPollerService_MaxHistorySize_EnforcesBoundaries()
    {
        _poller.MaxHistorySize = 60;
        Assert.AreEqual(60, _poller.MaxHistorySize);
    }
}
