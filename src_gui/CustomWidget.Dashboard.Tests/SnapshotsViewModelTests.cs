// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public class SnapshotsViewModelTests
{
    private AetherIpcService _ipc = null!;
    private SnapshotsViewModel _viewModel = null!;

    [TestInitialize]
    public void Setup()
    {
        _ipc = new AetherIpcService();
        _viewModel = new SnapshotsViewModel(_ipc);
    }

    [TestMethod]
    public async Task Test_SnapshotsViewModel_Initialization_PopulatesSnapshots()
    {
        await _viewModel.LoadSnapshotsAsync();
        Assert.IsTrue(_viewModel.Snapshots.Count > 0, "System snapshots collection should be populated.");
    }

    [TestMethod]
    public void Test_SnapshotItem_CreatedAtTextFormat()
    {
        var now = new DateTime(2026, 8, 16, 12, 30, 45);
        var snapshot = new SnapshotItem { CreatedAt = now };

        Assert.AreEqual("Created: 2026-08-16 12:30:45", snapshot.CreatedAtText);
    }

    [TestMethod]
    public async Task Test_SnapshotsViewModel_CreateSnapshot_InsertsNewSnapshot()
    {
        await _viewModel.LoadSnapshotsAsync();
        int initialCount = _viewModel.Snapshots.Count;

        _viewModel.NewSnapshotName = "Test Custom Baseline";
        await _viewModel.CreateSnapshotCommand.ExecuteAsync(null);

        Assert.AreEqual(initialCount + 1, _viewModel.Snapshots.Count, "Snapshots list count should increase by 1 after creation.");
        Assert.AreEqual("Test Custom Baseline", _viewModel.Snapshots[0].Name, "Newly created snapshot should be at the top of the collection.");
    }

    [TestMethod]
    public async Task Test_SnapshotsViewModel_DeleteSnapshot_RemovesSnapshot()
    {
        await _viewModel.LoadSnapshotsAsync();
        int initialCount = _viewModel.Snapshots.Count;
        var snapToDelete = _viewModel.Snapshots[0];

        await _viewModel.DeleteSnapshotCommand.ExecuteAsync(snapToDelete);

        Assert.AreEqual(initialCount - 1, _viewModel.Snapshots.Count, "Snapshots list count should decrease by 1 after deletion.");
    }
}
