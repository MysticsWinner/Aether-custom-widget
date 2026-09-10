// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Diagnostics;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.Services.Interfaces;
using Windows.System;

namespace CustomWidget.Dashboard.ViewModels;

/// <summary>
/// ViewModel for the About page — platform metadata, authors, GitHub links, and licensing.
/// B1 Fix: Constructor now takes IAetherIpcService (interface) instead of concrete AetherIpcService.
/// </summary>
public partial class AboutViewModel : ObservableObject
{
    private const string LogSource = "AboutViewModel";
    private readonly IAetherIpcService _ipc;

    [ObservableProperty] private string _appVersion = "0.6.0";
    [ObservableProperty] private string _engineVersion = "0.6.0";
    [ObservableProperty] private string _phaseName = "16 — Production Release Candidate (Diagnostics & System Control)";
    [ObservableProperty] private string _targetOS = "Windows 11 (x64 / ARM64)";
    [ObservableProperty] private string _githubRepoUrl = "https://github.com/MysticsWinner/Aether-custom-widget";
    [ObservableProperty] private string _licenseName = "MIT / Apache-2.0 License";
    [ObservableProperty] private string _authorsText = "Next-Gen Desktop Customization Team, Google DeepMind Agentic Coding Team & Open Source Contributors";

    // B1 Fix: Changed from concrete AetherIpcService to IAetherIpcService interface
    public AboutViewModel(IAetherIpcService ipc)
    {
        _ipc = ipc;
        if (!string.IsNullOrEmpty(ipc.LastEngineVersion))
        {
            EngineVersion = $"v{ipc.LastEngineVersion}";
        }
        DashboardLogger.Debug(LogSource, $"AboutViewModel initialized (engineVersion={EngineVersion})");
    }

    [RelayCommand]
    private async Task OpenGithubAsync()
    {
        DashboardLogger.Info(LogSource, $"Opening GitHub URL: {GithubRepoUrl}");
        try
        {
            await Launcher.LaunchUriAsync(new Uri(GithubRepoUrl));
        }
        catch (Exception ex)
        {
            DashboardLogger.Error(LogSource, "Failed to open GitHub URL", ex);
        }
    }

    [RelayCommand]
    private async Task OpenDocsAsync()
    {
        string docsUrl = $"{GithubRepoUrl}#readme";
        DashboardLogger.Info(LogSource, $"Opening docs URL: {docsUrl}");
        try
        {
            await Launcher.LaunchUriAsync(new Uri(docsUrl));
        }
        catch (Exception ex)
        {
            DashboardLogger.Error(LogSource, "Failed to open docs URL", ex);
        }
    }
}
