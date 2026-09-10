using System.Diagnostics;
using CustomWidget.Dashboard.Services.Interfaces;

namespace CustomWidget.Dashboard.Services;

/// <summary>
/// Manages the lifecycle of the Aether core engine daemon process.
/// Can start, stop, and monitor the <c>core_engine</c> Rust binary.
/// </summary>
public sealed class ProcessManagerService : IProcessManagerService
{
    private const string LogSource = "ProcessManager";

    private Process? _engineProcess;
    private readonly string _workspaceRoot;

    public string WorkspaceRoot => _workspaceRoot;

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
        
        try
        {
            Directory.CreateDirectory(Path.Combine(_workspaceRoot, "logs"));
        }
        catch (Exception ex)
        {
            DashboardLogger.Warn(LogSource, $"Failed to create logs directory: {ex.Message}");
        }

        DashboardLogger.Info(LogSource, $"ProcessManagerService initialized (workspaceRoot={_workspaceRoot})");
    }

    private void WriteToEngineLog(string line)
    {
        try
        {
            string path = Path.Combine(_workspaceRoot, "logs", "engine.log");
            File.AppendAllText(path, $"[{DateTime.Now:yyyy-MM-dd HH:mm:ss.fff}] {line}\n");
        }
        catch (Exception ex)
        {
            DashboardLogger.Warn(LogSource, $"Failed to write engine log: {ex.Message}");
        }
    }

    /// <summary>
    /// Starts the core engine daemon via <c>cargo run -p core_engine</c>.
    /// </summary>
    public async Task<bool> StartEngineAsync()
    {
        if (IsEngineRunning)
        {
            DashboardLogger.Info(LogSource, $"Engine already running (PID={EnginePid})");
            return true;
        }

        DashboardLogger.Info(LogSource, "Starting core engine daemon...");

        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = "cargo",
                Arguments = "run -p core_engine",
                WorkingDirectory = _workspaceRoot,
                UseShellExecute = false,
                CreateNoWindow = true,   // Hide the console window — daemon runs in background
                RedirectStandardOutput = true,
                RedirectStandardError = true,
            };

            _engineProcess = Process.Start(psi);

            if (_engineProcess is null)
            {
                DashboardLogger.Error(LogSource, "Process.Start returned null — failed to start engine");
                return false;
            }

            int pid = _engineProcess.Id;
            DashboardLogger.Info(LogSource, $"Engine process started (PID={pid})");

            _engineProcess.EnableRaisingEvents = true;
            _engineProcess.Exited += (_, _) =>
            {
                int exitCode = _engineProcess?.ExitCode ?? -1;
                DashboardLogger.Info(LogSource, $"Engine process exited (PID={pid}, exitCode={exitCode})");
                OnEngineExited?.Invoke(exitCode);
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
                        {
                            OnEngineOutput?.Invoke(line);
                            WriteToEngineLog(line);
                        }
                    }
                }
                catch (Exception ex)
                {
                    DashboardLogger.Debug(LogSource, $"Stdout reader exited: {ex.Message}");
                }
            });

            _ = Task.Run(async () =>
            {
                try
                {
                    while (_engineProcess is not null && !_engineProcess.HasExited)
                    {
                        var line = await _engineProcess.StandardError.ReadLineAsync();
                        if (line is not null)
                        {
                            OnEngineOutput?.Invoke(line);
                            WriteToEngineLog(line);
                        }
                    }
                }
                catch (Exception ex)
                {
                    DashboardLogger.Debug(LogSource, $"Stderr reader exited: {ex.Message}");
                }
            });

            // Give the engine a moment to start the IPC pipe
            await Task.Delay(2000);
            bool running = IsEngineRunning;
            DashboardLogger.Info(LogSource, $"Engine start result: running={running}");
            return running;
        }
        catch (Exception ex)
        {
            DashboardLogger.Error(LogSource, "Failed to start engine", ex);
            return false;
        }
    }

    /// <summary>
    /// Stops the core engine daemon gracefully and kills entire process tree.
    /// </summary>
    public async Task StopEngineAsync()
    {
        DashboardLogger.Info(LogSource, "Stopping engine...");

        if (_engineProcess is not null && !_engineProcess.HasExited)
        {
            try
            {
                int pid = _engineProcess.Id;
                _engineProcess.Kill(entireProcessTree: true);
                await _engineProcess.WaitForExitAsync();
                DashboardLogger.Info(LogSource, $"Managed engine process killed (PID={pid})");
            }
            catch (Exception ex)
            {
                DashboardLogger.Warn(LogSource, $"Error killing managed engine process: {ex.Message}");
            }
            finally
            {
                _engineProcess = null;
            }
        }

        // Kill any remaining core_engine processes on the system
        foreach (var proc in Process.GetProcessesByName("core_engine"))
        {
            try
            {
                int pid = proc.Id;
                proc.Kill(entireProcessTree: true);
                await proc.WaitForExitAsync();
                DashboardLogger.Info(LogSource, $"External engine process killed (PID={pid})");
            }
            catch (Exception ex)
            {
                DashboardLogger.Warn(LogSource, $"Error killing external engine process: {ex.Message}");
            }
        }

        DashboardLogger.Info(LogSource, "Engine stop completed");
    }

    /// <summary>
    /// Restarts the engine: stop, wait, then start.
    /// </summary>
    public async Task<bool> RestartEngineAsync()
    {
        DashboardLogger.Info(LogSource, "Restarting engine...");
        await StopEngineAsync();
        await Task.Delay(1000); // Wait for pipe cleanup
        return await StartEngineAsync();
    }

    /// <summary>
    /// B8 Fix: Dispose properly kills any engine process and logs it.
    /// </summary>
    public void Dispose()
    {
        try
        {
            if (_engineProcess is not null && !_engineProcess.HasExited)
            {
                int pid = _engineProcess.Id;
                _engineProcess.Kill(entireProcessTree: true);
                DashboardLogger.Info(LogSource, $"Engine process killed during Dispose (PID={pid})");
            }
        }
        catch (Exception ex)
        {
            DashboardLogger.Warn(LogSource, $"Error during Dispose: {ex.Message}");
        }
    }
}
