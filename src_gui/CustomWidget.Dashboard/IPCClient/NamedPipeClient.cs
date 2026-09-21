// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.IO;
using System.IO.Pipes;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using CustomWidget.Dashboard.Services;

namespace CustomWidget.Dashboard.IPCClient;

/// <summary>
/// Named Pipe IPC Client connecting the WinUI 3 Management Dashboard
/// to the Aether Core Engine Daemon via <c>\\.\pipe\CustomWidgetEngineControlPipe</c>.
///
/// Features chunked memory-stream reading, cancellation token propagation,
/// dynamic timeout controls, and structured error generation.
/// </summary>
public sealed class NamedPipeClient
{
    private const string LogSource = "NamedPipeClient";
    private const string PipeName = "CustomWidgetEngineControlPipe";
    public const int DefaultConnectTimeoutMs = 500;
    private const int ChunkBufferSize = 8192;

    /// <summary>
    /// Sends a JSON command to the Aether core engine and returns the full JSON response string.
    /// Opens a stateless pipe connection per call with configurable retries and exponential backoff.
    /// </summary>
    /// <param name="commandJson">JSON command to write.</param>
    /// <param name="ct">Optional cancellation token.</param>
    /// <param name="timeoutMs">Timeout in milliseconds for pipe connection.</param>
    /// <param name="maxRetries">Maximum retry attempts on transient connection/IO failures.</param>
    /// <returns>JSON response from the engine, or an error JSON object on failure.</returns>
    public async Task<string> SendCommandAsync(string commandJson, CancellationToken ct = default, int timeoutMs = DefaultConnectTimeoutMs, int maxRetries = 2)
    {
        if (string.IsNullOrWhiteSpace(commandJson))
            return BuildErrorJson("Command string is empty.");

        int attempt = 0;
        while (true)
        {
            try
            {
                using var pipeStream = new NamedPipeClientStream(
                    serverName: ".",
                    pipeName: PipeName,
                    direction: PipeDirection.InOut,
                    options: PipeOptions.Asynchronous);

                using var timeoutCts = new CancellationTokenSource(timeoutMs);
                using var linkedCts = CancellationTokenSource.CreateLinkedTokenSource(ct, timeoutCts.Token);

                DashboardLogger.Trace(LogSource, $"Connecting to pipe '{PipeName}' (attempt {attempt + 1}/{maxRetries + 1}, timeout={timeoutMs}ms)");
                await pipeStream.ConnectAsync(timeoutMs, linkedCts.Token).ConfigureAwait(false);
                DashboardLogger.Trace(LogSource, "Pipe connected");

                // Write command
                byte[] commandBytes = Encoding.UTF8.GetBytes(commandJson);
                await pipeStream.WriteAsync(commandBytes, linkedCts.Token).ConfigureAwait(false);
                await pipeStream.FlushAsync(linkedCts.Token).ConfigureAwait(false);

                // Read response — loop until pipe closes or no more data (supporting arbitrarily large chunked JSON payloads)
                using var ms = new MemoryStream();
                byte[] buffer = new byte[ChunkBufferSize];

                int bytesRead;
                while ((bytesRead = await pipeStream.ReadAsync(buffer, linkedCts.Token).ConfigureAwait(false)) > 0)
                {
                    ms.Write(buffer, 0, bytesRead);
                }

                string response = Encoding.UTF8.GetString(ms.ToArray());
                DashboardLogger.Trace(LogSource, $"IPC exchange complete: sent {commandBytes.Length} bytes, received {ms.Length} bytes");
                return response;
            }
            catch (OperationCanceledException) when (ct.IsCancellationRequested)
            {
                DashboardLogger.Info(LogSource, "IPC request cancelled by user cancellation token.");
                return BuildErrorJson("IPC request was cancelled by user.");
            }
            catch (OperationCanceledException)
            {
                DashboardLogger.Debug(LogSource, "Pipe connection timed out");
                return BuildErrorJson("Connection timed out — is the core engine running?");
            }
            catch (TimeoutException)
            {
                DashboardLogger.Debug(LogSource, "Pipe connection timed out (TimeoutException)");
                return BuildErrorJson("Connection timed out — is the core engine running?");
            }
            catch (Exception retryEx) when (attempt < maxRetries && !ct.IsCancellationRequested && retryEx is not TimeoutException and not OperationCanceledException)
            {
                attempt++;
                int delayMs = 100 * attempt;
                DashboardLogger.Debug(LogSource, $"Pipe connection attempt {attempt} failed: {retryEx.Message}. Retrying in {delayMs}ms...");
                try
                {
                    await Task.Delay(delayMs, ct).ConfigureAwait(false);
                }
                catch (OperationCanceledException)
                {
                    return BuildErrorJson("IPC request was cancelled by user.");
                }
            }
            catch (IOException ex) when (ex.Message.Contains("pipe", StringComparison.OrdinalIgnoreCase))
            {
                DashboardLogger.Warn(LogSource, $"Pipe I/O error: {ex.Message}");
                return BuildErrorJson($"Pipe I/O error: {ex.Message}");
            }
            catch (Exception ex)
            {
                DashboardLogger.Error(LogSource, "Unexpected IPC error", ex);
                return BuildErrorJson(ex.Message);
            }
        }
    }

    public static string BuildErrorJson(string message)
    {
        string escaped = message.Replace("\\", "\\\\").Replace("\"", "\\\"").Replace("\n", "\\n").Replace("\r", "");
        return $"{{\"status\": \"error\", \"message\": \"{escaped}\"}}";
    }
}
