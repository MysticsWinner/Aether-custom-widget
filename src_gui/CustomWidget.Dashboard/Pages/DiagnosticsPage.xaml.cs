// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace CustomWidget.Dashboard.Pages;

/// <summary>
/// Diagnostics page — live log viewer, IPC command console, and error inspector.
/// </summary>
public sealed partial class DiagnosticsPage : Page
{
    private readonly DiagnosticsViewModel _vm;
    private readonly DispatcherTimer _refreshTimer;

    public DiagnosticsPage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<DiagnosticsViewModel>();

        LogListView.ItemsSource = _vm.AllEntries;

        _refreshTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(500) };
        _refreshTimer.Tick += RefreshUI;
        _refreshTimer.Start();

        this.Unloaded += (_, _) => _refreshTimer.Stop();
    }

    private void RefreshUI(object? sender, object e)
    {
        // Update error badge
        int total = _vm.WarnCount + _vm.ErrorCount;
        if (total > 0)
        {
            ErrorBadge.Visibility = Visibility.Visible;
            ErrorBadgeText.Text = total.ToString();
        }
        else
        {
            ErrorBadge.Visibility = Visibility.Collapsed;
        }

        // Auto-scroll to latest log entry
        if (_vm.AllEntries.Count > 0)
        {
            LogListView.ScrollIntoView(_vm.AllEntries[^1]);
        }
    }

    private void SendBtn_Click(object sender, RoutedEventArgs e)
    {
        _vm.CommandText = CommandInput.Text;
        _ = _vm.SendCommandCommand.ExecuteAsync(null);

        // Update response after a short delay
        var timer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(1000) };
        timer.Tick += (_, _) =>
        {
            timer.Stop();
            ResponseOutput.Text = _vm.ResponseText;
        };
        timer.Start();
    }

    private void ClearBtn_Click(object sender, RoutedEventArgs e)
    {
        _vm.ClearLogsCommand.Execute(null);
    }

    private void LevelFilter_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (LevelFilter.SelectedItem is ComboBoxItem item)
        {
            _vm.SelectedLogLevel = item.Content?.ToString() ?? "All";
        }
    }
}
