// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml.Controls;

namespace CustomWidget.Dashboard.Pages;

public sealed partial class DesignTokensPage : Page
{
    private const string LogSource = "DesignTokensPage";
    private readonly DesignTokensViewModel _vm;

    public DesignTokensPage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<DesignTokensViewModel>();

        ColorTokensList.ItemsSource = _vm.ColorTokens;
        TypographyTokensList.ItemsSource = _vm.TypographyTokens;
        MaterialTokensList.ItemsSource = _vm.MaterialTokens;
        MotionTokensList.ItemsSource = _vm.MotionTokens;

        _vm.PropertyChanged += (s, e) =>
        {
            if (e.PropertyName == nameof(_vm.StatusMessage))
                StatusText.Text = _vm.StatusMessage;
            if (e.PropertyName == nameof(_vm.ActiveAccentHex))
                AccentText.Text = $"Windows 11 System Accent: {_vm.ActiveAccentHex}";
            if (e.PropertyName == nameof(_vm.ContrastRatioText))
                StatusBadge.Text = _vm.ContrastRatioText;
        };

        this.Unloaded += (_, _) => DashboardLogger.Debug(LogSource, "DesignTokensPage unloaded");
        DashboardLogger.Debug(LogSource, "DesignTokensPage loaded");
    }

    private void ResolveBtn_Click(object sender, Microsoft.UI.Xaml.RoutedEventArgs e)
    {
        DashboardLogger.Info(LogSource, "Resolve Tokens button clicked");
        _ = _vm.ResolveTokensCommand.ExecuteAsync(null);
    }
}
