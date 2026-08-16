// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Windows.Storage.Pickers;

namespace CustomWidget.Dashboard.Pages;

/// <summary>
/// Widgets management page — auto-discovers plugins from folders and provides 1-click loading/unloading.
/// </summary>
public sealed partial class WidgetsPage : Page
{
    private readonly WidgetsViewModel _vm;
    private readonly DispatcherTimer _refreshTimer;

    public WidgetsPage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<WidgetsViewModel>();

        DiscoveredList.ItemsSource = _vm.DiscoveredWidgets;
        RunningList.ItemsSource = _vm.Widgets;

        _refreshTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
        _refreshTimer.Tick += RefreshUI;
        _refreshTimer.Start();

        this.Unloaded += (_, _) => _refreshTimer.Stop();
    }

    private void RefreshUI(object? sender, object e)
    {
        StatusText.Text = _vm.StatusMessage;

        DiscoveredEmptyState.Visibility = _vm.DiscoveredWidgets.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
        DiscoveredList.Visibility = _vm.DiscoveredWidgets.Count > 0 ? Visibility.Visible : Visibility.Collapsed;

        RunningEmptyState.Visibility = _vm.Widgets.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
        RunningList.Visibility = _vm.Widgets.Count > 0 ? Visibility.Visible : Visibility.Collapsed;
    }

    private async void DiscoverBtn_Click(object sender, RoutedEventArgs e)
    {
        await _vm.DiscoverWidgetsAsync();
    }

    private void SearchBox_TextChanged(object sender, TextChangedEventArgs e)
    {
        _vm.SearchQuery = SearchBox.Text;
    }

    private async void ToggleLoad_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button btn && btn.Tag is WidgetInfo widget)
        {
            await _vm.ToggleWidgetLoadCommand.ExecuteAsync(widget);
        }
    }

    private async void LoadBtn_Click(object sender, RoutedEventArgs e)
    {
        try
        {
            var picker = new FileOpenPicker();
            picker.FileTypeFilter.Add(".toml");
            picker.SuggestedStartLocation = PickerLocationId.DocumentsLibrary;

            var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(App.Current
                .GetType().GetProperty("_mainWindow",
                    System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance)
                ?.GetValue(App.Current) as Window);

            if (hwnd != IntPtr.Zero)
            {
                WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);
            }

            var file = await picker.PickSingleFileAsync();
            if (file is not null)
            {
                await _vm.LoadWidgetCommand.ExecuteAsync(file.Path);
            }
        }
        catch
        {
            var dialog = new ContentDialog
            {
                Title = "Load Widget Manifest",
                Content = new TextBox { PlaceholderText = "Enter full path to widget.toml...", Name = "PathBox" },
                PrimaryButtonText = "Load",
                CloseButtonText = "Cancel",
                XamlRoot = this.XamlRoot,
            };

            if (await dialog.ShowAsync() == ContentDialogResult.Primary)
            {
                var textBox = dialog.Content as TextBox;
                if (!string.IsNullOrWhiteSpace(textBox?.Text))
                {
                    await _vm.LoadWidgetCommand.ExecuteAsync(textBox.Text);
                }
            }
        }
    }

    private void ReloadBtn_Click(object sender, RoutedEventArgs e)
        => _ = _vm.ReloadAllCommand.ExecuteAsync(null);

    private void UnloadBtn_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button btn && btn.Tag is string widgetId)
        {
            _ = _vm.UnloadWidgetCommand.ExecuteAsync(widgetId);
        }
    }

    private void LockBtn_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button btn && btn.Tag is string widgetId)
        {
            _ = _vm.ToggleWidgetLockCommand.ExecuteAsync(widgetId);
        }
    }

    private void ResetPosBtn_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button btn && btn.Tag is string widgetId)
        {
            _ = _vm.ResetWidgetPositionCommand.ExecuteAsync(widgetId);
        }
    }
}
