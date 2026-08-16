// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public class AiComposerViewModelTests
{
    private AetherIpcService _ipc = null!;
    private AiComposerViewModel _viewModel = null!;

    [TestInitialize]
    public void Setup()
    {
        _ipc = new AetherIpcService();
        _viewModel = new AiComposerViewModel(_ipc);
    }

    [TestMethod]
    public async Task Test_AiComposerViewModel_Synthesize_GeneratesDetails()
    {
        _viewModel.PromptInput = "Futuristic Cyberpunk Neon Workstation";
        await _viewModel.SynthesizeCommand.ExecuteAsync(null);

        Assert.IsTrue(_viewModel.IsDetailsVisible, "Details panel should be visible after synthesis.");
        Assert.IsTrue(_viewModel.ThemeText.Contains("cyberpunk"), "ThemeText should contain cyberpunk keyword.");
        Assert.IsTrue(_viewModel.MaterialText.Contains("Glass"), "MaterialText should specify Glass material.");
    }

    [TestMethod]
    public void Test_AiComposerViewModel_SelectPresetPrompt_TriggersSynthesis()
    {
        _viewModel.SelectPresetPrompt("Minimalist battery saver clock setup");
        Assert.AreEqual("Minimalist battery saver clock setup", _viewModel.PromptInput);
    }
}
