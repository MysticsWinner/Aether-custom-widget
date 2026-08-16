// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.IO;
using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.ViewModels;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.UI.Xaml;

namespace CustomWidget.Dashboard;

/// <summary>
/// Aether Studio — WinUI 3 Management Dashboard application entry point.
/// Configures dependency injection, theme, and launches the main window.
/// </summary>
public partial class App : Application
{
    private Window? _mainWindow;

    public Window? MainWindow => _mainWindow;

    /// <summary>
    /// Global service provider for dependency injection.
    /// </summary>
    public static IServiceProvider Services { get; private set; } = null!;

    /// <summary>
    /// Convenience accessor for the running App instance.
    /// </summary>
    public static new App Current => (App)Application.Current;

    public App()
    {
        this.UnhandledException += OnUnhandledException;
        AppDomain.CurrentDomain.UnhandledException += OnDomainUnhandledException;

        try
        {
            this.InitializeComponent();
            Services = ConfigureServices();
        }
        catch (Exception ex)
        {
            LogCrash("App_Constructor", ex);
            throw;
        }
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        try
        {
            _mainWindow = new MainWindow();
            _mainWindow.Activate();
        }
        catch (Exception ex)
        {
            LogCrash("OnLaunched", ex);
            throw;
        }
    }

    private void OnUnhandledException(object sender, Microsoft.UI.Xaml.UnhandledExceptionEventArgs e)
    {
        LogCrash("Xaml_UnhandledException", e.Exception);
    }

    private void OnDomainUnhandledException(object sender, System.UnhandledExceptionEventArgs e)
    {
        if (e.ExceptionObject is Exception ex)
            LogCrash("Domain_UnhandledException", ex);
    }

    public static void LogCrash(string source, Exception ex)
    {
        try
        {
            string log = $"[{DateTime.Now:yyyy-MM-dd HH:mm:ss}] Crash in {source}:\n{ex.TypeAndMessage()}\n{ex.StackTrace}\n\n";
            Console.WriteLine(log);
            System.Diagnostics.Debug.WriteLine(log);
            string path = Path.Combine(AppContext.BaseDirectory, "crash.log");
            File.AppendAllText(path, log);
        }
        catch { }
    }

    /// <summary>
    /// Registers all services, ViewModels, and infrastructure into the DI container.
    /// </summary>
    private static IServiceProvider ConfigureServices()
    {
        var services = new ServiceCollection();

        // ── Core Services (singletons — shared across the app lifetime) ──
        services.AddSingleton<AetherIpcService>();
        services.AddSingleton<TelemetryPollerService>();
        services.AddSingleton<ProcessManagerService>();
        services.AddSingleton<LogCollectorService>();
        services.AddSingleton<MemoryManagerService>();
        services.AddSingleton<WidgetSettingsService>();

        // ── ViewModels (transient — new instance per page navigation) ──
        services.AddTransient<OverviewViewModel>();
        services.AddTransient<WidgetsViewModel>();
        services.AddTransient<MarketplaceViewModel>();
        services.AddTransient<SnapshotsViewModel>();
        services.AddTransient<SecurityViewModel>();
        services.AddTransient<ServicesViewModel>();
        services.AddTransient<PerformanceViewModel>();
        services.AddTransient<DiagnosticsViewModel>();
        services.AddTransient<SettingsViewModel>();
        services.AddTransient<AboutViewModel>();
        services.AddTransient<DesignTokensViewModel>();
        services.AddTransient<ProfilesViewModel>();
        services.AddTransient<AiComposerViewModel>();

        var provider = services.BuildServiceProvider();

        // Attach fallback ProcessExit handler for process termination
        AppDomain.CurrentDomain.ProcessExit += (_, _) =>
        {
            try
            {
                var mem = provider.GetService<MemoryManagerService>();
                mem?.ShutdownAndCleanAllDependenciesAsync().GetAwaiter().GetResult();
            }
            catch { }
        };

        return provider;
    }
}

internal static class ExceptionExtensions
{
    public static string TypeAndMessage(this Exception ex)
        => $"{ex.GetType().FullName}: {ex.Message}" + (ex.InnerException is not null ? $" ---> {ex.InnerException.TypeAndMessage()}" : "");
}
