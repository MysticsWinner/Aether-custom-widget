// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.VisualStudio.TestTools.UnitTesting;

namespace CustomWidget.Dashboard.Tests;

[TestClass]
public class ProfilesViewModelTests
{
    private AetherIpcService _ipc = null!;
    private ProfilesViewModel _viewModel = null!;

    [TestInitialize]
    public void Setup()
    {
        _ipc = new AetherIpcService();
        _viewModel = new ProfilesViewModel(_ipc);
    }

    [TestMethod]
    public async Task Test_ProfilesViewModel_Initialization_PopulatesProfiles()
    {
        await _viewModel.LoadProfilesCommand.ExecuteAsync(null);
        Assert.IsTrue(_viewModel.Profiles.Count > 0, "Profiles collection should be populated.");
    }

    [TestMethod]
    public async Task Test_ProfilesViewModel_ActivateProfile_UpdatesActiveState()
    {
        await _viewModel.LoadProfilesCommand.ExecuteAsync(null);
        var targetProfile = _viewModel.Profiles[1]; // profile.gaming

        await _viewModel.ActivateProfileCommand.ExecuteAsync(targetProfile);

        Assert.AreEqual(targetProfile.Id, _viewModel.ActiveProfileId, "ActiveProfileId should match activated profile ID.");
        Assert.IsTrue(targetProfile.IsActive, "Target profile IsActive flag should be true.");
        Assert.IsFalse(_viewModel.Profiles[0].IsActive, "Previous profile IsActive flag should be false.");
        Assert.AreEqual("✓ Active Profile", targetProfile.ButtonText);
    }
}
