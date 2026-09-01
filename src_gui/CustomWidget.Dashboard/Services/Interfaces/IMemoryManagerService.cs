// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Threading.Tasks;

namespace CustomWidget.Dashboard.Services.Interfaces;

/// <summary>
/// Service abstraction for monitoring memory footprint and performing aggressive working-set trimming.
/// </summary>
public interface IMemoryManagerService
{
    void TrimWorkingSet();
    Task ShutdownAndCleanAllDependenciesAsync();
}
