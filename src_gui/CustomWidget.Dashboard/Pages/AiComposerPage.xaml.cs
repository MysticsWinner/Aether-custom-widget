// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;

namespace CustomWidget.Dashboard.Pages;

public sealed partial class AiComposerPage : Page
{
    private readonly AiComposerViewModel _vm;

    public AiComposerPage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<AiComposerViewModel>();

        _vm.PropertyChanged += (s, e) =>
        {
            if (e.PropertyName == nameof(_vm.StatusMessage))
                StatusText.Text = _vm.StatusMessage;
            if (e.PropertyName == nameof(_vm.SummaryText))
                SummaryText.Text = _vm.SummaryText;
            if (e.PropertyName == nameof(_vm.ThemeText))
                ThemeText.Text = _vm.ThemeText;
            if (e.PropertyName == nameof(_vm.MaterialText))
                MaterialText.Text = _vm.MaterialText;
            if (e.PropertyName == nameof(_vm.PerfText))
                PerfText.Text = _vm.PerfText;
            if (e.PropertyName == nameof(_vm.SecurityGateText))
                SecurityGateText.Text = _vm.SecurityGateText;
            if (e.PropertyName == nameof(_vm.IsDetailsVisible))
                DetailsPanel.Visibility = _vm.IsDetailsVisible ? Visibility.Visible : Visibility.Collapsed;
            if (e.PropertyName == nameof(_vm.PromptInput))
                PromptInput.Text = _vm.PromptInput;
        };
    }

    private void Synthesize_Click(object sender, RoutedEventArgs e)
    {
        _vm.PromptInput = PromptInput.Text;
        _ = _vm.SynthesizeCommand.ExecuteAsync(null);
    }

    private void ApplySetup_Click(object sender, RoutedEventArgs e)
    {
        _ = _vm.ApplySetupCommand.ExecuteAsync(null);
    }

    private void PromptInput_KeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (e.Key == Windows.System.VirtualKey.Enter)
        {
            _vm.PromptInput = PromptInput.Text;
            _ = _vm.SynthesizeCommand.ExecuteAsync(null);
        }
    }

    private void Preset_Click(object sender, RoutedEventArgs e)
    {
        if (sender is MenuFlyoutItem item && item.Tag is string prompt)
        {
            PromptInput.Text = prompt;
            _vm.SelectPresetPrompt(prompt);
        }
    }

    private void PresetChip_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button btn && btn.Tag is string prompt)
        {
            PromptInput.Text = prompt;
            _vm.SelectPresetPrompt(prompt);
        }
    }
}
