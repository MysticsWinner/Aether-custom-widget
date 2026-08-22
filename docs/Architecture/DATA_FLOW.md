# End-to-End Data Flow Architecture

**Purpose**: Documents telemetry collection, IPC command dispatch, and render command pipelines in Aether.  
**Audience**: Engine Developers, SDK Developers.  
**Prerequisites**: [System_Architecture.md](System_Architecture.md).  
**Related Documents**: [IPC.md](IPC.md), [Rendering.md](Rendering.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Specification  
**Owner**: Core Engine Team  

---

## 1. Telemetry Data Pipeline ("Collect Once, Publish Everywhere")

```
[ Win32 APIs / Hardware Drivers ]
               │
               ▼ (Sampled once per 10ms cycle)
      [ TelemetryService ]
               │
               ▼ (Atomic write)
    [ SharedTelemetryCache ]
               │
      ┌────────┴────────┐ (Lock-Free Read)
      ▼                 ▼
[ Widget 1 ]      [ Widget 2 ]
```

---

## 2. IPC Command Dispatch Pipeline

```
[ GUI / TUI Dashboard ]
          │ (JSON over Named Pipe: \\.\pipe\CustomWidgetEngineControlPipe)
          ▼
  [ IpcServer Dispatch ]
          │
          ├─► LoadWidget ────► [ SubsystemManager ] ──► Spawns AppContainer
          ├─► SetThemeMode ──► [ ThemeEngine ] ───────► Broadcasts CoreEvent
          └─► GetStatus ─────► [ HealthMonitor ] ─────► Returns JSON Response
```

---

## Future Work
- Add binary Protobuf IPC serialization option for zero-copy IPC payloads.

## Known Issues
- None.

## References
- [crates/ipc_protocol/src/messages.rs](file:///d:/Code/Aether-custom-widget/crates/ipc_protocol/src/messages.rs)

## Related Documents
- [IPC.md](IPC.md)
- [Rendering.md](Rendering.md)
