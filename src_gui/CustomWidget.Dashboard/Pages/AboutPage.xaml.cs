// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml.Controls;

namespace CustomWidget.Dashboard.Pages;

/// <summary>
/// About Page — displays application metadata, credits, GitHub repo links, and license information.
/// </summary>
public sealed partial class AboutPage : Page
{
    public AboutViewModel ViewModel { get; }

    public AboutPage()
    {
        this.InitializeComponent();
        ViewModel = App.Services.GetRequiredService<AboutViewModel>();
    }
}
