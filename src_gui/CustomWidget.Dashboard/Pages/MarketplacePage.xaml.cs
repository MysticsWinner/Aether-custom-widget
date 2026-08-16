// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace CustomWidget.Dashboard.Pages;

public sealed partial class MarketplacePage : Page
{
    private readonly MarketplaceViewModel _vm;

    public MarketplacePage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<MarketplaceViewModel>();
        CatalogList.ItemsSource = _vm.FilteredPackages;

        _vm.PropertyChanged += (s, e) =>
        {
            if (e.PropertyName == nameof(_vm.StatusMessage))
            {
                StatusText.Text = _vm.StatusMessage;
            }
        };
    }

    private void RefreshBtn_Click(object sender, RoutedEventArgs e)
    {
        _ = _vm.LoadCatalogCommand.ExecuteAsync(null);
    }

    private void SearchBox_TextChanged(object sender, TextChangedEventArgs e)
    {
        _vm.SearchQuery = SearchBox.Text;
    }

    private void CategoryCombo_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (CategoryCombo.SelectedItem is ComboBoxItem item)
        {
            _vm.SelectedCategory = item.Content?.ToString() ?? "All Categories";
        }
    }

    private async void InstallBtn_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button btn && btn.Tag is MarketplacePackageItem package)
        {
            if (package.IsInstalled)
            {
                await _vm.UninstallPackageCommand.ExecuteAsync(package);
            }
            else
            {
                await _vm.InstallPackageCommand.ExecuteAsync(package);
            }
        }
    }
}
