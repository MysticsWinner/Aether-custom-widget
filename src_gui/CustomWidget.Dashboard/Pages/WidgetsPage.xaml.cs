// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Windows.Storage.Pickers;

namespace CustomWidget.Dashboard.Pages;

/// <summary>
/// Widgets management page — load/unload widgets via IPC.
/// </summary>
public sealed partial class WidgetsPage : Page
{
    private readonly WidgetsViewModel _vm;
    private readonly DispatcherTimer _refreshTimer;

    public WidgetsPage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<WidgetsViewModel>();

        WidgetList.ItemsSource = _vm.Widgets;

        _refreshTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
        _refreshTimer.Tick += RefreshUI;
        _refreshTimer.Start();

        this.Unloaded += (_, _) => _refreshTimer.Stop();
    }

    private void RefreshUI(object? sender, object e)
    {
        StatusText.Text = _vm.StatusMessage;
        EmptyState.Visibility = _vm.Widgets.Count == 0 ? Visibility.Visible : Visibility.Collapsed;
        WidgetList.Visibility = _vm.Widgets.Count > 0 ? Visibility.Visible : Visibility.Collapsed;
    }

    private async void LoadBtn_Click(object sender, RoutedEventArgs e)
    {
        try
        {
            var picker = new FileOpenPicker();
            picker.FileTypeFilter.Add(".toml");
            picker.SuggestedStartLocation = PickerLocationId.DocumentsLibrary;

            // Get the window handle for the picker
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
            // Fallback: use a simple input dialog if the picker fails
            var dialog = new ContentDialog
            {
                Title = "Load Widget",
                Content = new TextBox { PlaceholderText = "Enter widget.toml path...", Name = "PathBox" },
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
}
