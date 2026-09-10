// Copyright (c) Aether Platform. Licensed under the MIT License.

using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace CustomWidget.Dashboard.Pages;

public sealed partial class SecurityPage : Page
{
    private const string LogSource = "SecurityPage";
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

        this.Unloaded += (_, _) => DashboardLogger.Debug(LogSource, "SecurityPage unloaded");
        DashboardLogger.Debug(LogSource, "SecurityPage loaded");
    }

    private void RefreshBtn_Click(object sender, RoutedEventArgs e)
    {
        DashboardLogger.Info(LogSource, "Refresh Security Audit clicked");
        _ = _vm.LoadSecurityStatusCommand.ExecuteAsync(null);
    }
}
