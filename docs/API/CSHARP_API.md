# C# .NET 8 Widget SDK & Dashboard Service APIs

**C# Widget Interface (`IWidget`), Named Pipe Client, and Dashboard MVVM Services**

---

## 1. C# Widget Plugin Interface (`IWidget`)

Located in `bindings/csharp/CustomWidget.SDK/IWidget.cs`:

```csharp
namespace CustomWidget.SDK;

public interface IWidget
{
    string Id { get; }
    string Name { get; }
    Version Version { get; }
    
    Task OnLoadAsync();
    Task OnMountAsync(IntPtr hostHwnd);
    Task OnUpdateAsync(TelemetrySnapshot snapshot);
    Task OnUnmountAsync();
    Task OnUnloadAsync();
}
```

---

## 2. Dashboard Service Architecture (`src_gui/CustomWidget.Dashboard/Services`)

The WinUI 3 Dashboard provides 6 background services:

| Service | File Path | Responsibility |
|:---|:---|:---|
| `AetherIpcService` | `Services/AetherIpcService.cs` | Async Named Pipe client, JSON command dispatch, auto-reconnect |
| `MemoryManagerService` | `Services/MemoryManagerService.cs` | Process working set tracking, GC triggers, and RAM budget enforcement |
| `ProcessManagerService` | `Services/ProcessManagerService.cs` | Host daemon lifecycle supervisor, process startup & shutdown |
| `TelemetryPollerService` | `Services/TelemetryPollerService.cs` | High-frequency telemetry polling loop with DispatcherQueue dispatch |
| `WidgetSettingsService` | `Services/WidgetSettingsService.cs` | Persistent JSON settings storage and per-widget options |
| `LogCollectorService` | `Services/LogCollectorService.cs` | Ring buffer log collector and diagnostic trace reader |
