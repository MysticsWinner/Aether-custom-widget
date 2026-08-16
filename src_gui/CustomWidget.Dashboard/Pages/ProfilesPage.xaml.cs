// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace CustomWidget.Dashboard.Pages;

public sealed partial class ProfilesPage : Page
{
    private readonly ProfilesViewModel _vm;

    public ProfilesPage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<ProfilesViewModel>();
        ProfilesList.ItemsSource = _vm.Profiles;

        _vm.PropertyChanged += (s, e) =>
        {
            if (e.PropertyName == nameof(_vm.StatusMessage))
                StatusText.Text = _vm.StatusMessage;
        };
    }

    private void RefreshBtn_Click(object sender, RoutedEventArgs e)
    {
        _ = _vm.LoadProfilesCommand.ExecuteAsync(null);
    }

    private async void ActivateProfile_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button btn && btn.Tag is DesktopProfileItem profile)
        {
            await _vm.ActivateProfileCommand.ExecuteAsync(profile);
        }
    }
}
