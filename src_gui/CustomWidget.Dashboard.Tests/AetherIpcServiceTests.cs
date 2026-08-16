// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public class AetherIpcServiceTests
{
    private AetherIpcService _ipc = null!;

    [TestInitialize]
    public void Setup()
    {
        _ipc = new AetherIpcService();
    }

    [TestMethod]
    public async Task Test_AetherIpcService_SearchMarketplaceAsync_ExecutesWithoutException()
    {
        string result = await _ipc.SearchMarketplaceAsync("monitoring", "all");
        Assert.IsNotNull(result, "SearchMarketplace response should not be null.");
    }

    [TestMethod]
    public async Task Test_AetherIpcService_ListSnapshotsAsync_ExecutesWithoutException()
    {
        string result = await _ipc.ListSnapshotsAsync();
        Assert.IsNotNull(result, "ListSnapshots response should not be null.");
    }

    [TestMethod]
    public async Task Test_AetherIpcService_GetSecurityAuditLogsAsync_ExecutesWithoutException()
    {
        string result = await _ipc.GetSecurityAuditLogsAsync();
        Assert.IsNotNull(result, "GetSecurityAuditLogs response should not be null.");
    }
}
