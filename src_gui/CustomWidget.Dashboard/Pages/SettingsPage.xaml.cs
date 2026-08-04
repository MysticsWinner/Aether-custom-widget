// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace CustomWidget.Dashboard.Pages;

/// <summary>
/// Settings page — application configuration and engine preferences.
/// </summary>
public sealed partial class SettingsPage : Page
{
    private readonly SettingsViewModel _vm;

    public SettingsPage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<SettingsViewModel>();

        // Initialize UI state from ViewModel
        ThemeCombo.SelectedIndex = _vm.SelectedThemeIndex;
        IntervalSlider.Value = _vm.PollingIntervalMs;
        IntervalText.Text = $"{_vm.PollingIntervalMs} ms";
        AutoStartToggle.IsOn = _vm.AutoStartEngine;
        CloudSyncToggle.IsOn = _vm.CloudSyncEnabled;
        AiToggle.IsOn = _vm.AiFeaturesEnabled;
        EngineVersionText.Text = _vm.EngineVersion;
    }

    private void ThemeCombo_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (ThemeCombo.SelectedIndex >= 0)
        {
            _vm.SelectedThemeIndex = ThemeCombo.SelectedIndex;
        }
    }

    private void IntervalSlider_ValueChanged(object sender, Microsoft.UI.Xaml.Controls.Primitives.RangeBaseValueChangedEventArgs e)
    {
        int val = (int)e.NewValue;
        _vm.PollingIntervalMs = val;
        if (IntervalText is not null)
            IntervalText.Text = $"{val} ms";
    }

    private void AutoStartToggle_Toggled(object sender, RoutedEventArgs e)
    {
        _vm.AutoStartEngine = AutoStartToggle.IsOn;
    }

    private void CloudSyncToggle_Toggled(object sender, RoutedEventArgs e)
    {
        _vm.CloudSyncEnabled = CloudSyncToggle.IsOn;
    }

    private void AiToggle_Toggled(object sender, RoutedEventArgs e)
    {
        _vm.AiFeaturesEnabled = AiToggle.IsOn;
    }

    private void ResetBtn_Click(object sender, RoutedEventArgs e)
    {
        _vm.ResetDefaultsCommand.Execute(null);

        // Sync UI
        ThemeCombo.SelectedIndex = _vm.SelectedThemeIndex;
        IntervalSlider.Value = _vm.PollingIntervalMs;
        AutoStartToggle.IsOn = _vm.AutoStartEngine;
        CloudSyncToggle.IsOn = _vm.CloudSyncEnabled;
        AiToggle.IsOn = _vm.AiFeaturesEnabled;
        StatusText.Text = _vm.StatusMessage;
    }
}
