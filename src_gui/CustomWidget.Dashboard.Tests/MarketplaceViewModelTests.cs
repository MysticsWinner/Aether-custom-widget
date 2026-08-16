// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public class MarketplaceViewModelTests
{
    private AetherIpcService _ipc = null!;
    private MarketplaceViewModel _viewModel = null!;

    [TestInitialize]
    public void Setup()
    {
        _ipc = new AetherIpcService();
        _viewModel = new MarketplaceViewModel(_ipc);
    }

    [TestMethod]
    public async Task Test_MarketplaceViewModel_Initialization_PopulatesCatalog()
    {
        await _viewModel.LoadCatalogAsync();
        Assert.IsTrue(_viewModel.FilteredPackages.Count > 0, "Marketplace filtered packages catalog should be populated.");
    }

    [TestMethod]
    public void Test_MarketplacePackageItem_FormattedProperties()
    {
        var package = new MarketplacePackageItem
        {
            Author = "Aether Core Team",
            Capabilities = "SystemMetrics",
            Downloads = 14200,
            Rating = 4.9
        };

        Assert.AreEqual("Publisher: Aether Core Team", package.AuthorText);
        Assert.AreEqual("Capabilities: SystemMetrics", package.CapabilitiesText);
        Assert.AreEqual("Downloads: 14,200", package.DownloadsText);
        Assert.AreEqual("4.9", package.RatingText);
        Assert.AreEqual("Install Package", package.ButtonText);
    }

    [TestMethod]
    public async Task Test_MarketplaceViewModel_SearchQuery_FiltersCatalog()
    {
        await _viewModel.LoadCatalogAsync();
        int total = _viewModel.FilteredPackages.Count;

        _viewModel.SearchQuery = "Cyberpunk";

        Assert.IsTrue(_viewModel.FilteredPackages.Count < total, "Filtered count should be less than total when search query is applied.");
        Assert.AreEqual("Cyberpunk Performance Matrix", _viewModel.FilteredPackages[0].Name);
    }

    [TestMethod]
    public async Task Test_MarketplaceViewModel_CategorySelection_FiltersCatalog()
    {
        await _viewModel.LoadCatalogAsync();

        _viewModel.SelectedCategory = "Media";

        Assert.AreEqual(1, _viewModel.FilteredPackages.Count, "Only 1 package should match Media category.");
        Assert.AreEqual("Spectra Audio Visualizer", _viewModel.FilteredPackages[0].Name);
    }

    [TestMethod]
    public async Task Test_MarketplaceViewModel_InstallPackage_UpdatesInstalledState()
    {
        await _viewModel.LoadCatalogAsync();
        var package = _viewModel.FilteredPackages[0];
        package.IsInstalled = false;

        await _viewModel.InstallPackageCommand.ExecuteAsync(package);

        Assert.IsTrue(package.IsInstalled, "Package IsInstalled flag should be true after installation.");
        Assert.AreEqual("✓ Installed", package.ButtonText);
        Assert.IsTrue(_viewModel.StatusMessage.Contains("Verified by Ed25519"), "Status message should confirm Ed25519 signature verification.");
    }

    [TestMethod]
    public async Task Test_MarketplaceViewModel_UninstallPackage_ResetsInstalledState()
    {
        await _viewModel.LoadCatalogAsync();
        var package = _viewModel.FilteredPackages[0];
        package.IsInstalled = true;

        await _viewModel.UninstallPackageCommand.ExecuteAsync(package);

        Assert.IsFalse(package.IsInstalled, "Package IsInstalled flag should be false after uninstallation.");
        Assert.AreEqual("Install Package", package.ButtonText);
    }
}
