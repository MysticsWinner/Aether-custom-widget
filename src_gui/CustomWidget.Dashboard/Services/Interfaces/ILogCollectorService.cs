// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using CustomWidget.Dashboard.Models;

namespace CustomWidget.Dashboard.Services.Interfaces;

/// <summary>
/// Service abstraction for capturing, parsing, and streaming engine log entries in real-time.
/// </summary>
public interface ILogCollectorService
{
    ObservableCollection<LogEntry> Logs { get; }
    event Action<LogEntry>? OnNewLog;

    void AppendLog(string level, string message, string source);
    Task ClearLogsAsync();
}
