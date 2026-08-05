# Aether — C# SDK & Dashboard Service APIs

**C# SDK Interfaces (`CustomWidget.SDK`) and WinUI Service Layer**

---

## 1. C# SDK Interfaces (`bindings/csharp/CustomWidget.SDK/IWidget.cs`)

C# widgets implement the `IWidget` interface:

```csharp
namespace CustomWidget.SDK;

public interface IWidget
{
    void OnLoad();
    void OnMount();
    void OnUpdate(TickContext context);
    void OnUnmount();
    void OnUnload();
    WidgetState State { get; }
}
```

---

## 2. Dashboard IPC Service API (`AetherIpcService.cs`)

The WinUI 3 management app communicates with the Rust engine via `AetherIpcService`:

```csharp
public class AetherIpcService
{
    public async Task<bool> PingAsync();
    public async Task<StatusPayload?> GetStatusAsync();
    public async Task<bool> LoadWidgetAsync(string manifestPath);
    public async Task<bool> UnloadWidgetAsync(string widgetId);
    public async Task<bool> SetThemeModeAsync(string mode);
}
```
