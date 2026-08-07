# Architecture Overview

**Purpose**: High-level executive overview of Aether's system architecture, subsystem orchestrator, event bus, and rendering pipeline.  
**Audience**: Architects, Lead Developers, Technical Stakeholders.  
**Prerequisites**: [Root README](../../README.md), [Detailed_Project_Report.md](Detailed_Project_Report.md).  
**Related Documents**: [System_Architecture.md](../01_Architecture/System_Architecture.md), [Data_Flow.md](../01_Architecture/Data_Flow.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Reference  
**Owner**: Core Architecture Team  

---

## 1. High-Level Blueprint

```
 ┌─────────────────────────────────────────────────────────────────────────────┐
 │                         Aether User Interface Layer                         │
 │   ┌───────────────────────────────┐     ┌───────────────────────────────┐   │
 │   │ WinUI 3 Dashboard (C# .NET 8) │     │ Ratatui TUI Dashboard (Rust)  │   │
 │   └───────────────┬───────────────┘     └───────────────┬───────────────┘   │
 └───────────────────┼─────────────────────────────────────┼───────────────────┘
                     │  IPC Win32 Named Pipe               │
 ┌───────────────────▼─────────────────────────────────────▼───────────────────┐
 │                        Aether Core Daemon Runtime                           │
 │   ┌─────────────────────────────────────────────────────────────────────┐   │
 │   │ Async Engine Host Daemon (tokio 1.38 + tracing structured logging)   │   │
 │   └──────┬────────────────────┬────────────────────┬──────────────┬─────┘   │
 │          │ Telemetry          │ DirectComposition  │ Security     │ IPC     │
 │   ┌──────▼──────┐      ┌──────▼──────┐      ┌──────▼──────┐┌──────▼──────┐  │
 │   │ Telemetry   │      │ Render      │      │ Capability  ││ IPC Named   │  │
 │   │ Subsystem   │      │ Subsystem   │      │ Broker      ││ Pipe Server │  │
 │   └──────┬──────┘      └──────┬──────┘      └──────┬──────┘└─────────────┘  │
 │          │                     │                   │                        │
 │   ┌──────▼──────┐      ┌──────▼──────┐      ┌──────▼──────┐                 │
 │   │ Shared Cache│      │ Direct2D    │      │ AppContainer│                 │
 │   │ (Lock-Free) │      │ Composition │      │ Sandbox     │                 │
 │   └─────────────┘      └─────────────┘      └─────────────┘                 │
 └─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Core Architectural Principles

1. **Collect Once, Publish Everywhere**:
   Hardware metrics are sampled once per 10ms cycle by `system_providers` and cached in lock-free `SharedTelemetryCache`. Widgets query metrics zero-cost without direct Win32 API calls.
2. **Zero-Trust AppContainer Sandboxing**:
   All 3rd-party widget plugins execute inside Windows AppContainer sandboxes supervised by `plugin_runtime` and `capability_broker`.
3. **Retained-Mode Dirty Region Rendering**:
   Widgets emit `DrawCommand` batches. `DirtyRegionTracker` calculates minimal dirty bounding boxes to update DWM surfaces with zero tear and < 0.32ms frame times.

---

## Future Work
- Abstract display surface layer behind cross-platform traits.

## Known Issues
- None.

## References
- [Detailed_Project_Report.md](Detailed_Project_Report.md)
- [System_Architecture.md](../01_Architecture/System_Architecture.md)

## Related Documents
- [Root README](../../README.md)
- [Project_Status.md](Project_Status.md)
