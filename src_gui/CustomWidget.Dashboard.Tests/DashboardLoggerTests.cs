// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.Threading.Tasks;
using CustomWidget.Dashboard.Services;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public class DashboardLoggerTests
{
    [TestInitialize]
    public void Setup()
    {
        DashboardLogger.MinimumLevel = DashboardLogger.LogLevel.Trace;
        DashboardLogger.EnableFileLogging = false; // Disable file IO for pure unit test isolation
    }

    [TestMethod]
    public void Test_DashboardLogger_LogLevel_Ordering()
    {
        Assert.IsTrue(DashboardLogger.LogLevel.Trace < DashboardLogger.LogLevel.Debug);
        Assert.IsTrue(DashboardLogger.LogLevel.Debug < DashboardLogger.LogLevel.Info);
        Assert.IsTrue(DashboardLogger.LogLevel.Info < DashboardLogger.LogLevel.Warn);
        Assert.IsTrue(DashboardLogger.LogLevel.Warn < DashboardLogger.LogLevel.Error);
        Assert.IsTrue(DashboardLogger.LogLevel.Error < DashboardLogger.LogLevel.Fatal);
    }

    [TestMethod]
    public void Test_DashboardLogger_MinimumLevel_FilterProperty()
    {
        DashboardLogger.MinimumLevel = DashboardLogger.LogLevel.Warn;
        Assert.AreEqual(DashboardLogger.LogLevel.Warn, DashboardLogger.MinimumLevel);

        DashboardLogger.MinimumLevel = DashboardLogger.LogLevel.Debug;
        Assert.AreEqual(DashboardLogger.LogLevel.Debug, DashboardLogger.MinimumLevel);
    }

    [TestMethod]
    public void Test_DashboardLogger_ConvenienceMethods_ExecuteWithoutException()
    {
        // Must never throw regardless of arguments
        DashboardLogger.Trace("TestTrace", "Trace message");
        DashboardLogger.Debug("TestDebug", "Debug message");
        DashboardLogger.Info("TestInfo", "Info message");
        DashboardLogger.Warn("TestWarn", "Warn message", new InvalidOperationException("Mock warning"));
        DashboardLogger.Error("TestError", "Error message", new ApplicationException("Mock error"));
        DashboardLogger.Fatal("TestFatal", "Fatal crash message", new AccessViolationException("Mock fatal"));
    }

    [TestMethod]
    public void Test_DashboardLogger_NullException_DoesNotThrow()
    {
        DashboardLogger.Log(DashboardLogger.LogLevel.Info, "TestNull", "Message with null exception", null);
        DashboardLogger.Warn("TestNull", "Warn with null ex", null);
        DashboardLogger.Error("TestNull", "Error with null ex", null);
        DashboardLogger.Fatal("TestNull", "Fatal with null ex", null);
    }

    [TestMethod]
    public void Test_DashboardLogger_ConcurrentWrites_AreThreadSafe()
    {
        Parallel.For(0, 500, i =>
        {
            var level = (DashboardLogger.LogLevel)(i % 6);
            DashboardLogger.Log(level, $"ThreadSource_{i % 8}", $"Concurrent message iteration {i}");
        });

        // Ensure flush does not deadlock or throw
        DashboardLogger.Flush();
    }

    [TestMethod]
    public void Test_DashboardLogger_Flush_EmptyOrPopulated_ExecutesCleanly()
    {
        DashboardLogger.Flush();
        DashboardLogger.Info("FlushTest", "Message before flush");
        DashboardLogger.Flush();
    }
}
