// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public class MemoryManagerServiceTests
{
    private AetherIpcService _ipc = null!;
    private TelemetryPollerService _poller = null!;
    private ProcessManagerService _processManager = null!;
    private LogCollectorService _logCollector = null!;
    private MemoryManagerService _memoryManager = null!;

    [TestInitialize]
    public void Setup()
    {
        _ipc = new AetherIpcService();
        _poller = new TelemetryPollerService(_ipc);
        _processManager = new ProcessManagerService();
        _logCollector = new LogCollectorService(_processManager);
        _memoryManager = new MemoryManagerService(_processManager, _poller, _logCollector);
    }

    [TestMethod]
    public void Test_MemoryManagerService_OptimizeMemory_ExecutesWithoutException()
    {
        _memoryManager.OptimizeMemory();
        Assert.IsTrue(true, "OptimizeMemory should execute GC collect and working set trim cleanly.");
    }

    [TestMethod]
    public void Test_MemoryManagerService_DevMode_And_AggressiveGc_Flags()
    {
        _memoryManager.IsDevelopmentMode = true;
        Assert.IsTrue(_memoryManager.IsDevelopmentMode);
        Assert.AreEqual(TimeSpan.FromSeconds(15), _memoryManager.GcInterval);

        _memoryManager.IsAggressiveGcEnabled = true;
        Assert.IsTrue(_memoryManager.IsAggressiveGcEnabled);

        _memoryManager.IsDevelopmentMode = false;
        Assert.IsFalse(_memoryManager.IsDevelopmentMode);
        Assert.AreEqual(TimeSpan.FromMinutes(5), _memoryManager.GcInterval);
    }

    [TestMethod]
    public void Test_MemoryManagerService_OptimizeMemory_Aggressive_CompactsLoh_And_RecordsStats()
    {
        _memoryManager.IsDevelopmentMode = true;
        _memoryManager.OptimizeMemory(aggressive: true, compactLoh: true);

        var stats = _memoryManager.GetLastStats();
        Assert.IsNotNull(stats);
        Assert.IsTrue(stats.WasAggressive);
        Assert.IsTrue(stats.WasLohCompacted);
        Assert.IsTrue(stats.IsDevMode);
        Assert.IsTrue(stats.BeforeBytes >= 0);
        Assert.IsTrue(stats.AfterBytes >= 0);
    }

    [TestMethod]
    public async Task Test_MemoryManagerService_TriggerEngineGarbageCollection_WithStub()
    {
        string result = await _memoryManager.TriggerEngineGarbageCollectionAsync(aggressive: true, trimWorkingSet: true);
        Assert.IsNotNull(result);
    }

    [TestMethod]
    public async Task Test_MemoryManagerService_ShutdownAndCleanAllDependencies_ExecutesCleanly()
    {
        await _memoryManager.ShutdownAndCleanAllDependenciesAsync();
        Assert.IsFalse(_processManager.IsEngineRunning, "Engine processes should be stopped after shutdown.");
    }
}

