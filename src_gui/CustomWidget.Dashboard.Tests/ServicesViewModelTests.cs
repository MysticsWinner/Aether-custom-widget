// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public class ServicesViewModelTests
{
    private ProcessManagerService _processManager = null!;
    private AetherIpcService _ipc = null!;
    private TelemetryPollerService _poller = null!;
    private ServicesViewModel _viewModel = null!;

    [TestInitialize]
    public void Setup()
    {
        _processManager = new ProcessManagerService();
        _ipc = new AetherIpcService();
        _poller = new TelemetryPollerService(_ipc);
        _viewModel = new ServicesViewModel(_processManager, _ipc, _poller);
    }

    [TestMethod]
    public async Task Test_ServicesViewModel_RefreshSubsystemsAsync_Connected_PopulatesList()
    {
        await _viewModel.RefreshSubsystemsAsync(true);
        Assert.IsTrue(_viewModel.Subsystems.Count >= 9, "Subsystems collection should contain all architectural subsystems when connected.");
    }

    [TestMethod]
    public async Task Test_ServicesViewModel_RefreshSubsystemsAsync_Disconnected_ClearsList()
    {
        await _viewModel.RefreshSubsystemsAsync(true);
        Assert.IsTrue(_viewModel.Subsystems.Count > 0);

        await _viewModel.RefreshSubsystemsAsync(false);
        Assert.AreEqual(0, _viewModel.Subsystems.Count, "Subsystems collection should be cleared when disconnected.");
    }

    [TestMethod]
    public void Test_ServicesViewModel_InitialEngineStatusText()
    {
        Assert.IsNotNull(_viewModel.EngineStatusText);
        Assert.IsNotNull(_viewModel.EnginePidText);
    }
}
