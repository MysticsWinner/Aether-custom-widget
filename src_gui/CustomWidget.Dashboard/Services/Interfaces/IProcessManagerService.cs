// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.Threading.Tasks;

namespace CustomWidget.Dashboard.Services.Interfaces;

/// <summary>
/// Service abstraction for managing the Aether Core Engine process lifecycle.
/// </summary>
public interface IProcessManagerService : IDisposable
{
    string WorkspaceRoot { get; }
    bool IsEngineRunning { get; }
    int? EnginePid { get; }

    event Action<int>? OnEngineExited;
    event Action<string>? OnEngineOutput;

    Task<bool> StartEngineAsync();
    Task StopEngineAsync();
    Task<bool> RestartEngineAsync();
}
