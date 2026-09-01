// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.IO;
using System.IO.Pipes;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

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
    private const string PipeName = "CustomWidgetEngineControlPipe";
    public const int DefaultConnectTimeoutMs = 2000;
    private const int ChunkBufferSize = 8192;

    /// <summary>
    /// Sends a JSON command to the Aether core engine and returns the full JSON response string.
    /// Opens a stateless pipe connection per call.
    /// </summary>
    /// <param name="commandJson">JSON command to write.</param>
    /// <param name="ct">Optional cancellation token.</param>
    /// <param name="timeoutMs">Timeout in milliseconds for pipe connection.</param>
    /// <returns>JSON response from the engine, or an error JSON object on failure.</returns>
    public async Task<string> SendCommandAsync(string commandJson, CancellationToken ct = default, int timeoutMs = DefaultConnectTimeoutMs)
    {
        if (string.IsNullOrWhiteSpace(commandJson))
            return BuildErrorJson("Command string is empty.");

        try
        {
            using var pipeStream = new NamedPipeClientStream(
                serverName: ".",
                pipeName: PipeName,
                direction: PipeDirection.InOut,
                options: PipeOptions.Asynchronous);

            using var timeoutCts = new CancellationTokenSource(timeoutMs);
            using var linkedCts = CancellationTokenSource.CreateLinkedTokenSource(ct, timeoutCts.Token);

            await pipeStream.ConnectAsync(timeoutMs, linkedCts.Token).ConfigureAwait(false);

            // Write command
            byte[] commandBytes = Encoding.UTF8.GetBytes(commandJson);
            await pipeStream.WriteAsync(commandBytes, linkedCts.Token).ConfigureAwait(false);
            await pipeStream.FlushAsync(linkedCts.Token).ConfigureAwait(false);

            // Read response (supporting arbitrarily large chunked JSON payloads)
            using var ms = new MemoryStream();
            byte[] buffer = new byte[ChunkBufferSize];

            int bytesRead = await pipeStream.ReadAsync(buffer, linkedCts.Token).ConfigureAwait(false);
            if (bytesRead > 0)
            {
                ms.Write(buffer, 0, bytesRead);
            }

            return Encoding.UTF8.GetString(ms.ToArray());
        }
        catch (OperationCanceledException) when (ct.IsCancellationRequested)
        {
            return BuildErrorJson("IPC request was cancelled by user.");
        }
        catch (OperationCanceledException)
        {
            return BuildErrorJson("Connection timed out — is the core engine running?");
        }
        catch (TimeoutException)
        {
            return BuildErrorJson("Connection timed out — is the core engine running?");
        }
        catch (IOException ex) when (ex.Message.Contains("pipe", StringComparison.OrdinalIgnoreCase))
        {
            return BuildErrorJson($"Pipe I/O error: {ex.Message}");
        }
        catch (Exception ex)
        {
            return BuildErrorJson(ex.Message);
        }
    }

    public static string BuildErrorJson(string message)
    {
        string escaped = message.Replace("\\", "\\\\").Replace("\"", "\\\"").Replace("\n", "\\n").Replace("\r", "");
        return $"{{\"status\": \"error\", \"message\": \"{escaped}\"}}";
    }
}
