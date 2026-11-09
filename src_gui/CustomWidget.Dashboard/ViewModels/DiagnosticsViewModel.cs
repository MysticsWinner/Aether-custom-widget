using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.Services;
using CustomWidget.Dashboard.Services.Interfaces;

namespace CustomWidget.Dashboard.ViewModels;

/// <summary>
/// ViewModel for the Diagnostics page — live log viewer, IPC console, and error inspector.
/// </summary>
public partial class DiagnosticsViewModel : ObservableObject
{
    private const string LogSource = "DiagnosticsViewModel";
    private readonly ILogCollectorService _logCollector;
    private readonly IAetherIpcService _ipc;
    private readonly IMemoryManagerService? _memoryManager;

    [ObservableProperty] private string _commandText = "\"GetStatus\"";
    [ObservableProperty] private string _responseText = "";
    [ObservableProperty] private bool _isBusy;
    [ObservableProperty] private string _selectedLogLevel = "All";
    [ObservableProperty] private int _warnCount;
    [ObservableProperty] private int _errorCount;
    [ObservableProperty] private string _errorBadgeText = "";
    [ObservableProperty] private bool _isDevelopmentMode;
    [ObservableProperty] private bool _isAggressiveGc;
    [ObservableProperty] private string _gcStatusMessage = "GC ready";

    /// <summary>
    /// All log entries (unfiltered). Bound to the log viewer ListView.
    /// </summary>
    public ObservableCollection<LogEntry> AllEntries => _logCollector.Logs;

    /// <summary>
    /// Available log level filter options.
    /// </summary>
    public string[] LogLevels { get; } = ["All", "TRACE", "DEBUG", "INFO", "WARN", "ERROR"];

    public DiagnosticsViewModel(
        ILogCollectorService logCollector,
        IAetherIpcService ipc,
        IMemoryManagerService? memoryManager = null)
    {
        _logCollector = logCollector;
        _ipc = ipc;
        _memoryManager = memoryManager;

        if (_memoryManager != null)
        {
            _isDevelopmentMode = _memoryManager.IsDevelopmentMode;
            _isAggressiveGc = _memoryManager.IsAggressiveGcEnabled;
        }

        _logCollector.OnNewLog += OnNewLogEntry;
        DashboardLogger.Debug(LogSource, $"DiagnosticsViewModel initialized (DevMode={IsDevelopmentMode}, AggressiveGC={IsAggressiveGc})");
    }


    private void OnNewLogEntry(LogEntry entry)
    {
        WarnCount = _logCollector.WarnCount;
        ErrorCount = _logCollector.ErrorCount;
        ErrorBadgeText = (WarnCount + ErrorCount) > 0
            ? $"{WarnCount + ErrorCount}"
            : "";
    }

    /// <summary>
    /// Sends the raw JSON command from the IPC console text box to the engine.
    /// </summary>
    [RelayCommand]
    private async Task SendCommandAsync()
    {
        if (string.IsNullOrWhiteSpace(CommandText)) return;

        IsBusy = true;
        ResponseText = "Sending...";
        try
        {
            string response = await _ipc.SendRawCommandAsync(CommandText.Trim());
            DashboardLogger.Info(LogSource, $"IPC console: sent '{CommandText.Trim().Truncate(80)}', received {response.Length} chars");

            // Pretty-print the JSON response
            try
            {
                var parsed = System.Text.Json.JsonDocument.Parse(response);
                ResponseText = System.Text.Json.JsonSerializer.Serialize(
                    parsed, new System.Text.Json.JsonSerializerOptions { WriteIndented = true });
            }
            catch
            {
                ResponseText = response;
            }

            // Log the IPC exchange
            _logCollector.AddManualEntry("INFO", "ipc_console",
                $"Sent: {CommandText.Trim()} → {response.Truncate(200)}");
        }
        catch (Exception ex)
        {
            ResponseText = $"Error: {ex.Message}";
            _logCollector.AddManualEntry("ERROR", "ipc_console", ex.Message);
        }
        finally
        {
            IsBusy = false;
        }
    }

    [RelayCommand]
    private void ClearLogs()
    {
        _logCollector.Clear();
        WarnCount = 0;
        ErrorCount = 0;
        ErrorBadgeText = "";
        DashboardLogger.Info(LogSource, "Logs cleared from diagnostics view");
    }

    /// <summary>
    /// Executes immediate garbage collection across the dashboard process and instructs core_engine to collect.
    /// </summary>
    [RelayCommand]
    private async Task CollectGarbageAsync()
    {
        IsBusy = true;
        GcStatusMessage = "Running GC pass...";
        try
        {
            if (_memoryManager != null)
            {
                _memoryManager.OptimizeMemory(aggressive: true, compactLoh: true);
                var stats = _memoryManager.GetLastStats();
                string engineResp = await _memoryManager.TriggerEngineGarbageCollectionAsync(aggressive: true, trimWorkingSet: true);
                GcStatusMessage = $"Reclaimed {stats.ReclaimedBytes / 1024}KB in GUI. Engine GC: {engineResp.Truncate(60)}";
                DashboardLogger.Info(LogSource, $"Manual GC completed: {GcStatusMessage}");
            }
            else
            {
                GC.Collect(2, GCCollectionMode.Forced, true, true);
                GC.WaitForPendingFinalizers();
                GcStatusMessage = "Direct GC completed";
            }
        }
        catch (Exception ex)
        {
            GcStatusMessage = $"GC error: {ex.Message}";
            DashboardLogger.Warn(LogSource, "GC failed", ex);
        }
        finally
        {
            IsBusy = false;
        }
    }

    /// <summary>
    /// Toggles development mode garbage collection policy.
    /// </summary>
    [RelayCommand]
    private async Task ToggleDevModeAsync()
    {
        IsDevelopmentMode = !IsDevelopmentMode;
        if (_memoryManager != null)
        {
            _memoryManager.IsDevelopmentMode = IsDevelopmentMode;
            // Inform engine daemon of updated policy
            string cmd = $"{{\"SetGarbageCollectionPolicy\":{{\"dev_mode\":{IsDevelopmentMode.ToString().ToLowerInvariant()},\"aggressive\":{IsAggressiveGc.ToString().ToLowerInvariant()},\"interval_secs\":{(IsDevelopmentMode ? 15 : 300)}}}}}";
            await _ipc.SendRawCommandAsync(cmd);
        }
        GcStatusMessage = $"Dev mode {(IsDevelopmentMode ? "enabled (15s GC)" : "disabled (5min GC)")}";
    }

    /// <summary>
    /// Toggles aggressive garbage collection flag.
    /// </summary>
    [RelayCommand]
    private async Task ToggleAggressiveGcAsync()
    {
        IsAggressiveGc = !IsAggressiveGc;
        if (_memoryManager != null)
        {
            _memoryManager.IsAggressiveGcEnabled = IsAggressiveGc;
            string cmd = $"{{\"SetGarbageCollectionPolicy\":{{\"dev_mode\":{IsDevelopmentMode.ToString().ToLowerInvariant()},\"aggressive\":{IsAggressiveGc.ToString().ToLowerInvariant()},\"interval_secs\":{(IsDevelopmentMode ? 15 : 300)}}}}}";
            await _ipc.SendRawCommandAsync(cmd);
        }
        GcStatusMessage = $"Aggressive GC {(IsAggressiveGc ? "active" : "inactive")}";
    }

    /// <summary>
    /// Returns all log entries as a single string for export.
    /// </summary>
    public string ExportLogsAsText()
    {
        return string.Join(Environment.NewLine,
            AllEntries.Select(e => $"[{e.Timestamp:yyyy-MM-dd HH:mm:ss.fff}] [{e.Level}] {e.Target}: {e.Message}"));
    }
}


/// <summary>
/// String extension for truncation.
/// </summary>
internal static class StringExtensions
{
    public static string Truncate(this string value, int maxLength)
        => value.Length <= maxLength ? value : value[..maxLength] + "…";
}
