using System;
using System.Threading.Tasks;

namespace CustomWidget.Dashboard.Services.Interfaces;

/// <summary>
/// Service abstraction for monitoring memory footprint, garbage collection, and working-set trimming.
/// </summary>
public interface IMemoryManagerService
{
    /// <summary>
    /// Flag indicating whether the application is running in development mode.
    /// In development mode, GC runs on a rapid cadence (15s) with full LOH compaction.
    /// </summary>
    bool IsDevelopmentMode { get; set; }

    /// <summary>
    /// Flag indicating whether aggressive garbage collection (forced collections & working set trimming) is active.
    /// </summary>
    bool IsAggressiveGcEnabled { get; set; }

    /// <summary>
    /// Periodic garbage collection interval.
    /// </summary>
    TimeSpan GcInterval { get; set; }

    /// <summary>
    /// Returns the metrics from the most recent garbage collection pass.
    /// </summary>
    GarbageCollectionStats GetLastStats();

    /// <summary>
    /// Forces immediate garbage collection and trims physical working set memory.
    /// </summary>
    void OptimizeMemory(bool aggressive = false, bool compactLoh = false);

    /// <summary>
    /// Alias for OptimizeMemory.
    /// </summary>
    void TrimWorkingSet();

    /// <summary>
    /// Dispatches a garbage collection command to the Core Engine daemon over IPC.
    /// </summary>
    Task<string> TriggerEngineGarbageCollectionAsync(bool aggressive = false, bool trimWorkingSet = true);

    /// <summary>
    /// Gracefully stops all engine dependencies and releases all allocated memory on exit.
    /// </summary>
    Task ShutdownAndCleanAllDependenciesAsync();
}

/// <summary>
/// Diagnostic report for a single garbage collection and working set trimming cycle.
/// </summary>
public record GarbageCollectionStats(
    long BeforeBytes,
    long AfterBytes,
    long ReclaimedBytes,
    int Gen0Count,
    int Gen1Count,
    int Gen2Count,
    bool WasAggressive,
    bool WasLohCompacted,
    bool IsDevMode,
    DateTime Timestamp
);

