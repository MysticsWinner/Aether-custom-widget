# Inter-Process Communication (IPC) Protocol

**Purpose**: Specifications for Aether's Win32 Named Pipe IPC protocol and shared ring buffer.  
**Audience**: GUI Developers, CLI Developers, Core Engine Team.  
**Prerequisites**: [Data_Flow.md](Data_Flow.md).  
**Related Documents**: [System_Architecture.md](System_Architecture.md), [Dashboard.md](../05_GUI/Dashboard.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Protocol Specification  
**Owner**: Network & IPC Team  

---

## 1. IPC Pipe Address

- **Named Pipe URI**: `\\.\pipe\CustomWidgetEngineControlPipe`
- **Protocol**: JSON text messages terminated by newline (`\n`).

---

## 2. Control Commands (`ControlCommand`)

- `Ping` / `Pong`
- `GetStatus`
- `LoadWidget { manifest_path }`
- `UnloadWidget { widget_id }`
- `SetThemeMode { mode }`
- `SetWidgetRenderConfig { widget_id, config_json }`
- `GetWidgetRenderConfig { widget_id }`

---

## Future Work
- Add named pipe security DACLs to restrict IPC access to Administrator / current user SID.

## Known Issues
- None.

## References
- [crates/ipc_protocol/src/messages.rs](file:///d:/Code/Aether-custom-widget/crates/ipc_protocol/src/messages.rs)

## Related Documents
- [System_Architecture.md](System_Architecture.md)
