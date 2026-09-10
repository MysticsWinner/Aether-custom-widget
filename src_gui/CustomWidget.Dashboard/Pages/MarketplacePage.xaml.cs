// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace CustomWidget.Dashboard.Pages;

public sealed partial class MarketplacePage : Page
{
    private const string LogSource = "MarketplacePage";
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

        this.Unloaded += (_, _) => DashboardLogger.Debug(LogSource, "MarketplacePage unloaded");
        DashboardLogger.Debug(LogSource, "MarketplacePage loaded");
    }

    private void RefreshBtn_Click(object sender, RoutedEventArgs e)
    {
        DashboardLogger.Info(LogSource, "Refresh Catalog clicked");
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
            string category = item.Content?.ToString() ?? "All Categories";
            DashboardLogger.Debug(LogSource, $"Category filter changed to: {category}");
            _vm.SelectedCategory = category;
        }
    }

    private async void InstallBtn_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button btn && btn.Tag is MarketplacePackageItem package)
        {
            DashboardLogger.Info(LogSource, $"Package action clicked: {package.Name} (Installed={package.IsInstalled})");
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
