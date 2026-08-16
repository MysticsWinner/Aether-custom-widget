// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public class DesignTokensViewModelTests
{
    private AetherIpcService _ipc = null!;
    private DesignTokensViewModel _viewModel = null!;

    [TestInitialize]
    public void Setup()
    {
        _ipc = new AetherIpcService();
        _viewModel = new DesignTokensViewModel(_ipc);
    }

    [TestMethod]
    public async Task Test_DesignTokensViewModel_ResolveTokens_PopulatesTokenCategories()
    {
        await _viewModel.ResolveTokensCommand.ExecuteAsync(null);

        Assert.IsTrue(_viewModel.ColorTokens.Count > 0, "Color tokens collection should be populated.");
        Assert.IsTrue(_viewModel.TypographyTokens.Count > 0, "Typography tokens collection should be populated.");
        Assert.IsTrue(_viewModel.MaterialTokens.Count > 0, "Material tokens collection should be populated.");
        Assert.IsTrue(_viewModel.MotionTokens.Count > 0, "Motion tokens collection should be populated.");
    }

    [TestMethod]
    public void Test_DesignTokensViewModel_SelectAccentColor_UpdatesHex()
    {
        _viewModel.SelectAccentColor("#FF0055");
        Assert.AreEqual("#FF0055", _viewModel.ActiveAccentHex);
    }
}
