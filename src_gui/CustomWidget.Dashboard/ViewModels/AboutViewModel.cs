// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Diagnostics;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Services;
using Windows.System;

namespace CustomWidget.Dashboard.ViewModels;

/// <summary>
/// ViewModel for the About page — platform metadata, authors, GitHub links, and licensing.
/// </summary>
public partial class AboutViewModel : ObservableObject
{
    private readonly AetherIpcService _ipc;

    [ObservableProperty] private string _appVersion = "0.5.0";
    [ObservableProperty] private string _engineVersion = "0.5.0";
    [ObservableProperty] private string _phaseName = "15 — Production Release Candidate";
    [ObservableProperty] private string _targetOS = "Windows 11 (x64 / ARM64)";
    [ObservableProperty] private string _githubRepoUrl = "https://github.com/MysticsWinner/Aether-custom-widget";
    [ObservableProperty] private string _licenseName = "MIT / Apache-2.0 License";
    [ObservableProperty] private string _authorsText = "Next-Gen Desktop Customization Team, Google DeepMind Agentic Coding Team & Open Source Contributors";

    public AboutViewModel(AetherIpcService ipc)
    {
        _ipc = ipc;
        if (!string.IsNullOrEmpty(ipc.LastEngineVersion))
        {
            EngineVersion = $"v{ipc.LastEngineVersion}";
        }
    }

    [RelayCommand]
    private async Task OpenGithubAsync()
    {
        try
        {
            await Launcher.LaunchUriAsync(new Uri(GithubRepoUrl));
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Failed to open URL: {ex.Message}");
        }
    }

    [RelayCommand]
    private async Task OpenDocsAsync()
    {
        try
        {
            await Launcher.LaunchUriAsync(new Uri($"{GithubRepoUrl}#readme"));
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Failed to open docs: {ex.Message}");
        }
    }
}
