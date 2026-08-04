// Copyright (c) Aether Platform. Licensed under the MIT License.

using System.Diagnostics;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// Manages the lifecycle of the Aether core engine daemon process.
/// Can start, stop, and monitor the <c>core_engine</c> Rust binary.
/// </summary>
public sealed class ProcessManagerService
{
    private Process? _engineProcess;
    private readonly string _workspaceRoot;

    /// <summary>
    /// Fired when the engine process exits (normally or due to crash).
    /// </summary>
    public event Action<int>? OnEngineExited;

    /// <summary>
    /// Fired when new output is received from the engine process.
    /// </summary>
    public event Action<string>? OnEngineOutput;

    /// <summary>
    /// True if the engine process is currently running (either launched by us or externally).
    /// </summary>
    public bool IsEngineRunning
    {
        get
        {
            // Check our managed process first
            if (_engineProcess is not null && !_engineProcess.HasExited)
                return true;

            // Check for externally-launched engine processes
            return Process.GetProcessesByName("core_engine").Length > 0;
        }
    }

    /// <summary>
    /// PID of the running engine process, or null.
    /// </summary>
    public int? EnginePid
    {
        get
        {
            if (_engineProcess is not null && !_engineProcess.HasExited)
                return _engineProcess.Id;

            var procs = Process.GetProcessesByName("core_engine");
            return procs.Length > 0 ? procs[0].Id : null;
        }
    }

    public ProcessManagerService()
    {
        // Find the workspace root by walking up from the exe/assembly location
        // In development: the project is at src_gui\CustomWidget.Dashboard within the workspace
        var assemblyDir = AppContext.BaseDirectory;
        var dir = new DirectoryInfo(assemblyDir);

        // Walk up until we find Cargo.toml (workspace root marker)
        while (dir is not null && !File.Exists(Path.Combine(dir.FullName, "Cargo.toml")))
        {
            dir = dir.Parent;
        }

        _workspaceRoot = dir?.FullName ?? Path.GetFullPath(Path.Combine(assemblyDir, "..", "..", "..", "..", ".."));
    }

    /// <summary>
    /// Starts the core engine daemon via <c>cargo run -p core_engine</c>.
    /// </summary>
    public async Task<bool> StartEngineAsync()
    {
        if (IsEngineRunning)
            return true;

        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = "cargo",
                Arguments = "run -p core_engine",
                WorkingDirectory = _workspaceRoot,
                UseShellExecute = false,
                CreateNoWindow = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
            };

            _engineProcess = Process.Start(psi);

            if (_engineProcess is null)
                return false;

            _engineProcess.EnableRaisingEvents = true;
            _engineProcess.Exited += (_, _) =>
            {
                OnEngineExited?.Invoke(_engineProcess?.ExitCode ?? -1);
            };

            // Read output asynchronously
            _ = Task.Run(async () =>
            {
                try
                {
                    while (_engineProcess is not null && !_engineProcess.HasExited)
                    {
                        var line = await _engineProcess.StandardOutput.ReadLineAsync();
                        if (line is not null)
                            OnEngineOutput?.Invoke(line);
                    }
                }
                catch { /* Process exited */ }
            });

            _ = Task.Run(async () =>
            {
                try
                {
                    while (_engineProcess is not null && !_engineProcess.HasExited)
                    {
                        var line = await _engineProcess.StandardError.ReadLineAsync();
                        if (line is not null)
                            OnEngineOutput?.Invoke(line);
                    }
                }
                catch { /* Process exited */ }
            });

            // Give the engine a moment to start the IPC pipe
            await Task.Delay(2000);
            return IsEngineRunning;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Stops the core engine daemon gracefully.
    /// </summary>
    public async Task StopEngineAsync()
    {
        // Try to stop our managed process
        if (_engineProcess is not null && !_engineProcess.HasExited)
        {
            try
            {
                // Send Ctrl+C signal via GenerateConsoleCtrlEvent is complex on Windows;
                // fallback to Kill for reliability
                _engineProcess.Kill(entireProcessTree: true);
                await _engineProcess.WaitForExitAsync();
            }
            catch { /* Already exited */ }
            finally
            {
                _engineProcess = null;
            }
            return;
        }

        // Kill any externally-launched core_engine processes
        foreach (var proc in Process.GetProcessesByName("core_engine"))
        {
            try
            {
                proc.Kill(entireProcessTree: true);
                await proc.WaitForExitAsync();
            }
            catch { /* Access denied or already exited */ }
        }
    }

    /// <summary>
    /// Restarts the engine: stop, wait, then start.
    /// </summary>
    public async Task<bool> RestartEngineAsync()
    {
        await StopEngineAsync();
        await Task.Delay(1000); // Wait for pipe cleanup
        return await StartEngineAsync();
    }
}
