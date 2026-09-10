// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace CustomWidget.Dashboard.Pages;

public sealed partial class SnapshotsPage : Page
{
    private const string LogSource = "SnapshotsPage";
    private readonly SnapshotsViewModel _vm;

    public SnapshotsPage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<SnapshotsViewModel>();
        SnapshotsList.ItemsSource = _vm.Snapshots;

        _vm.PropertyChanged += (s, e) =>
        {
            if (e.PropertyName == nameof(_vm.StatusMessage))
            {
                StatusText.Text = _vm.StatusMessage;
            }
        };

        this.Unloaded += (_, _) => DashboardLogger.Debug(LogSource, "SnapshotsPage unloaded");
        DashboardLogger.Debug(LogSource, "SnapshotsPage loaded");
    }

    private void RefreshBtn_Click(object sender, RoutedEventArgs e)
    {
        DashboardLogger.Info(LogSource, "Refresh Snapshots clicked");
        _ = _vm.LoadSnapshotsCommand.ExecuteAsync(null);
    }

    private async void CreateBtn_Click(object sender, RoutedEventArgs e)
    {
        DashboardLogger.Info(LogSource, $"Create Snapshot clicked: '{NameInput.Text}'");
        _vm.NewSnapshotName = NameInput.Text;
        await _vm.CreateSnapshotCommand.ExecuteAsync(null);
        NameInput.Text = "";
    }

    private async void RestoreBtn_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button btn && btn.Tag is SnapshotItem snap)
        {
            await _vm.RestoreSnapshotCommand.ExecuteAsync(snap);
        }
    }

    private async void DeleteBtn_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button btn && btn.Tag is SnapshotItem snap)
        {
            await _vm.DeleteSnapshotCommand.ExecuteAsync(snap);
        }
    }
}
