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

    [TestMethod]
    public void Test_AetherIpcService_InitialConnectionState_IsFalse()
    {
        Assert.IsFalse(_ipc.IsConnected, "Default connection state should be false before communication.");
    }

    [TestMethod]
    public async Task Test_AetherIpcService_OfflineGetStatus_ReturnsNullGracefully()
    {
        using var cts = new System.Threading.CancellationTokenSource(200);
        var status = await _ipc.GetStatusAsync(cts.Token);
        Assert.IsNull(status, "Offline engine should result in null status.");
        Assert.IsFalse(_ipc.IsConnected, "Offline engine should report IsConnected = false.");
    }

    [TestMethod]
    public void Test_NamedPipeClient_BuildErrorJson_ProducesValidJsonStructure()
    {
        string error = CustomWidget.Dashboard.IPCClient.NamedPipeClient.BuildErrorJson("Test error message");
        Assert.IsTrue(error.Contains("\"status\": \"error\""));
        Assert.IsTrue(error.Contains("Test error message"));
    }

    [TestMethod]
    public async Task Test_NamedPipeClient_Cancellation_ReturnsCancelledJson()
    {
        var client = new CustomWidget.Dashboard.IPCClient.NamedPipeClient();
        using var cts = new System.Threading.CancellationTokenSource();
        cts.Cancel(); // Cancel immediately
        string res = await client.SendCommandAsync("{\"Ping\":{}}", cts.Token);
        Assert.IsTrue(res.Contains("cancelled"), "Should return cancelled message.");
    }

    [TestMethod]
    public void Test_EngineStatus_Deserialization_IncludesNetworkMetrics()
    {
        string json = "{\"status\":\"ok\",\"cpu_pct\":15.2,\"net_recv_bytes_per_sec\":1048576,\"net_sent_bytes_per_sec\":524288}";
        var status = System.Text.Json.JsonSerializer.Deserialize<CustomWidget.Dashboard.Models.EngineStatus>(json);
        Assert.IsNotNull(status);
        Assert.AreEqual(1048576UL, status.NetRecvBytesPerSec);
        Assert.AreEqual(524288UL, status.NetSentBytesPerSec);
    }

    [TestMethod]
    public void Test_TelemetrySample_FromStatus_PreservesNetworkThroughput()
    {
        var status = new CustomWidget.Dashboard.Models.EngineStatus
        {
            Status = "ok",
            CpuPct = 25.0f,
            NetRecvBytesPerSec = 4096,
            NetSentBytesPerSec = 8192
        };
        var sample = CustomWidget.Dashboard.Models.TelemetrySample.FromStatus(status);
        Assert.AreEqual(4096UL, sample.NetRecvBytesPerSec);
        Assert.AreEqual(8192UL, sample.NetSentBytesPerSec);
    }
}
