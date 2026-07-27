using System;
using System.IO;
using System.IO.Pipes;
using System.Text;
using System.Threading.Tasks;

namespace CustomWidget.Dashboard.IPCClient
{
    /// <summary>
    /// Decoupled Named Pipe IPC Client connecting WinUI 3 Management Dashboard
    /// to the high-performance Headless Core Engine Daemon.
    /// </summary>
    public class NamedPipeClient
    {
        private const string PipeName = "CustomWidgetEngineControlPipe";

        public async Task<string> SendCommandAsync(string commandJson)
        {
            try
            {
                using var pipeStream = new NamedPipeClientStream(".", PipeName, PipeDirection.InOut, PipeOptions.Asynchronous);
                await pipeStream.ConnectAsync(2000); // 2 sec timeout

                byte[] buffer = Encoding.UTF8.GetBytes(commandJson);
                await pipeStream.WriteAsync(buffer, 0, buffer.Length);

                byte[] responseBuffer = new byte[4096];
                int bytesRead = await pipeStream.ReadAsync(responseBuffer, 0, responseBuffer.Length);

                return Encoding.UTF8.GetString(responseBuffer, 0, bytesRead);
            }
            catch (Exception ex)
            {
                return $"{{\"status\": \"error\", \"message\": \"{ex.Message}\"}}";
            }
        }
    }
}
