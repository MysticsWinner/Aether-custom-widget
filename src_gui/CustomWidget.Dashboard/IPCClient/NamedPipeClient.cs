// Copyright (c) Aether Platform. Licensed under the MIT License.

using System;
using System.IO;
using System.IO.Pipes;
using System.Text;
using System.Threading.Tasks;

namespace CustomWidget.Dashboard.IPCClient;

/// <summary>
/// Named Pipe IPC Client connecting the WinUI 3 Management Dashboard
/// to the Aether Core Engine Daemon via <c>\\.\pipe\CustomWidgetEngineControlPipe</c>.
///
/// Each call opens a fresh pipe connection (matching the Rust server's per-connection model).
/// All telemetry data returned is REAL — sourced from the daemon's SharedTelemetryCache.
/// </summary>
public sealed class NamedPipeClient
{
    private const string PipeName = "CustomWidgetEngineControlPipe";
    private const int ConnectTimeoutMs = 2000;
    private const int ResponseBufferSize = 16384; // 16 KB — large enough for status + subsystem list

    /// <summary>
    /// Sends a JSON command to the Aether core engine and returns the JSON response.
    /// Opens a new pipe connection per call (stateless request-response).
    /// </summary>
    /// <param name="commandJson">
    /// JSON-encoded <c>ControlCommand</c> string — e.g. <c>"GetStatus"</c> or
    /// <c>{"LoadWidget":{"manifest_path":"..."}}</c>.
    /// </param>
    /// <returns>JSON response from the engine, or an error JSON object on failure.</returns>
    public async Task<string> SendCommandAsync(string commandJson)
    {
        try
        {
            using var pipeStream = new NamedPipeClientStream(
                serverName: ".",
                pipeName: PipeName,
                direction: PipeDirection.InOut,
                options: PipeOptions.Asynchronous);

            await pipeStream.ConnectAsync(ConnectTimeoutMs);

            // Write command
            byte[] commandBytes = Encoding.UTF8.GetBytes(commandJson);
            await pipeStream.WriteAsync(commandBytes, 0, commandBytes.Length);
            await pipeStream.FlushAsync();

            // Read response (may arrive in multiple chunks)
            using var ms = new MemoryStream();
            byte[] buffer = new byte[ResponseBufferSize];
            int bytesRead = await pipeStream.ReadAsync(buffer, 0, buffer.Length);

            if (bytesRead > 0)
            {
                ms.Write(buffer, 0, bytesRead);
            }

            return Encoding.UTF8.GetString(ms.ToArray());
        }
        catch (TimeoutException)
        {
            return BuildErrorJson("Connection timed out — is the core engine running?");
        }
        catch (IOException ex) when (ex.Message.Contains("pipe"))
        {
            return BuildErrorJson($"Pipe I/O error: {ex.Message}");
        }
        catch (Exception ex)
        {
            return BuildErrorJson(ex.Message);
        }
    }

    private static string BuildErrorJson(string message)
    {
        // Escape any quotes in the message for valid JSON
        string escaped = message.Replace("\\", "\\\\").Replace("\"", "\\\"");
        return $"{{\"status\": \"error\", \"message\": \"{escaped}\"}}";
    }
}
