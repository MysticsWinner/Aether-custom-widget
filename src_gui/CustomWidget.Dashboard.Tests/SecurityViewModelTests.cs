// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public class SecurityViewModelTests
{
    private AetherIpcService _ipc = null!;
    private SecurityViewModel _viewModel = null!;

    [TestInitialize]
    public void Setup()
    {
        _ipc = new AetherIpcService();
        _viewModel = new SecurityViewModel(_ipc);
    }

    [TestMethod]
    public async Task Test_SecurityViewModel_Initialization_PopulatesCapabilitiesAndAudits()
    {
        await _viewModel.LoadSecurityStatusAsync();
        Assert.IsTrue(_viewModel.Capabilities.Count > 0, "Capability manifest tokens list should be populated.");
        Assert.IsTrue(_viewModel.AuditLogs.Count > 0, "Security audit log stream should be populated.");
    }

    [TestMethod]
    public void Test_CapabilityTokenItem_CategoryTextFormat()
    {
        var item = new CapabilityTokenItem { Category = "Hardware Telemetry" };
        Assert.AreEqual("(Hardware Telemetry)", item.CategoryText);
    }
}
