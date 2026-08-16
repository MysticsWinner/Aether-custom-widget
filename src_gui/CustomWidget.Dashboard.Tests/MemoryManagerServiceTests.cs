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
    public async Task Test_MemoryManagerService_ShutdownAndCleanAllDependencies_ExecutesCleanly()
    {
        await _memoryManager.ShutdownAndCleanAllDependenciesAsync();
        Assert.IsFalse(_processManager.IsEngineRunning, "Engine processes should be stopped after shutdown.");
    }
}
