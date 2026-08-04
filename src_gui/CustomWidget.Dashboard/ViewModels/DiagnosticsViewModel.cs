// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CustomWidget.Dashboard.Models;
using CustomWidget.Dashboard.Services;

namespace CustomWidget.Dashboard.ViewModels;

/// <summary>
/// ViewModel for the Diagnostics page — live log viewer, IPC console, and error inspector.
/// </summary>
public partial class DiagnosticsViewModel : ObservableObject
{
    private readonly LogCollectorService _logCollector;
    private readonly AetherIpcService _ipc;

    [ObservableProperty] private string _commandText = "\"GetStatus\"";
    [ObservableProperty] private string _responseText = "";
    [ObservableProperty] private bool _isBusy;
    [ObservableProperty] private string _selectedLogLevel = "All";
    [ObservableProperty] private int _warnCount;
    [ObservableProperty] private int _errorCount;
    [ObservableProperty] private string _errorBadgeText = "";

    /// <summary>
    /// All log entries (unfiltered). Bound to the log viewer ListView.
    /// </summary>
    public ObservableCollection<LogEntry> AllEntries => _logCollector.Entries;

    /// <summary>
    /// Available log level filter options.
    /// </summary>
    public string[] LogLevels { get; } = ["All", "TRACE", "DEBUG", "INFO", "WARN", "ERROR"];

    public DiagnosticsViewModel(LogCollectorService logCollector, AetherIpcService ipc)
    {
        _logCollector = logCollector;
        _ipc = ipc;

        _logCollector.OnNewEntry += OnNewLogEntry;
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
