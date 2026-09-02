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

The WinUI 3 Dashboard provides 6 background services registered via Dependency Injection (`Microsoft.Extensions.DependencyInjection`) using both interface and concrete bindings:

| Service Interface | Concrete Implementation | Responsibility |
|:---|:---|:---|
| `IAetherIpcService` | `Services/AetherIpcService.cs` | Async Named Pipe client (`NamedPipeClient`), chunked memory-stream reader, JSON command dispatch, auto-reconnect |
| `IMemoryManagerService` | `Services/MemoryManagerService.cs` | Process working set tracking, optimized GC triggers, and RAM budget enforcement |
| `IProcessManagerService` | `Services/ProcessManagerService.cs` | Host daemon lifecycle supervisor, process startup & shutdown |
| `ITelemetryPollerService` | `Services/TelemetryPollerService.cs` | High-frequency telemetry polling loop with DispatcherQueue UI dispatch |
| `IWidgetSettingsService` | `Services/WidgetSettingsService.cs` | Persistent JSON settings storage and per-widget options |
| `ILogCollectorService` | `Services/LogCollectorService.cs` | Thread-safe ring buffer log collector and diagnostic trace reader with DispatcherQueue marshaling |

### 2.1 Dependency Injection Pattern
Services are registered in `App.xaml.cs` via:
```csharp
services.AddSingleton<ConcreteService>();
services.AddSingleton<IServiceInterface>(sp => sp.GetRequiredService<ConcreteService>());
```
Consumers inject the interface `IServiceInterface` across ViewModels and Pages.

