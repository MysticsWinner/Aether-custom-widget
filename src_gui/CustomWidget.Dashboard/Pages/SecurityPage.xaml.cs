// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace CustomWidget.Dashboard.Pages;

public sealed partial class SecurityPage : Page
{
    private readonly SecurityViewModel _vm;

    public SecurityPage()
    {
        this.InitializeComponent();
        _vm = App.Services.GetRequiredService<SecurityViewModel>();
        CapabilityList.ItemsSource = _vm.Capabilities;
        AuditList.ItemsSource = _vm.AuditLogs;

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
        _ = _vm.LoadSecurityStatusCommand.ExecuteAsync(null);
    }
}
