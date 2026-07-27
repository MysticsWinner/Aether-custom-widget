# Aether Architecture — Core Platform Blueprint

## Overview

The **Aether Platform** is a multi-tier, hardware-accelerated desktop customization engine built on Rust (`windows-rs`, `tokio`). It is designed around modularity, low memory utilization, zero-trust security, and reactive high-refresh rendering.

---

## Core System Ecosystem

- **Aether Runtime**: Headless core service daemon managing subsystem lifecycles and event dispatching.
- **Aether Renderer**: DirectComposition / Direct2D 2D GPU compositing pipeline rendering directly onto Windows DWM `WorkerW` surfaces.
- **Aether SDK**: Multi-language developer SDK supporting Rust, C# .NET 8, and TypeScript.
- **Aether CLI**: npm-style package manager CLI with Ed25519 signature verification.
- **Aether Studio**: WinUI 3 management dashboard.

---

## Subsystem Architecture Topology

```mermaid
graph TB
    subgraph GUI_Layer ["Aether Studio (C# / WinUI 3 Dashboard)"]
        Dashboard["Dashboard & Settings UI"]
        MarketplaceUI["Marketplace GUI"]
    end

    subgraph Service_Layer ["Aether Runtime Daemon Service (Rust / Tokio)"]
        CoreRuntime["Core Runtime Event Loop"]
        LayoutEngine["Taffy Flexbox Solver"]
        RenderEngine["Aether Renderer (DirectComposition)"]
        AnimationEngine["Spring Physics Engine"]
        ThemeEngine["Theme Token Resolver"]
        SystemProviders["Telemetry Collector ('Collect Once')"]
        MarketplaceManager["Aether CLI Manager & Ed25519 Verifier"]
        CloudSyncEngine["CRDT Encrypted Cloud Sync"]
        AiEngine["AI Intelligence & Workflow Engine"]
        ProductionEngine["Performance Profiler"]
    end

    subgraph IPC_Layer ["Inter-Process Communication Boundary"]
        NamedPipes["Win32 Named Pipes (Control Commands)"]
        SharedMemory["Shared Memory Ring Buffer (Telemetry Data)"]
    end

    subgraph Sandbox_Layer ["Zero-Trust AppContainer Sandboxes"]
        PluginProcess1["Lua 5.4 Sandboxed Process"]
        PluginProcess2["Native Executable Plugin"]
    end

    subgraph OS_Layer ["Windows DWM & Graphics Hardware"]
        DWMCompositor["DWM Compositor Surface (WorkerW)"]
        GPU["Direct3D Hardware Acceleration"]
    end

    Dashboard <-->|Named Pipes| CoreRuntime
    CoreRuntime <-->|Shared Memory / Pipes| PluginProcess1
    CoreRuntime <-->|Shared Memory / Pipes| PluginProcess2

    CoreRuntime --> LayoutEngine
    CoreRuntime --> ThemeEngine
    CoreRuntime --> SystemProviders
    CoreRuntime --> AnimationEngine
    CoreRuntime --> MarketplaceManager
    CoreRuntime --> CloudSyncEngine
    CoreRuntime --> AiEngine
    CoreRuntime --> ProductionEngine

    AnimationEngine --> RenderEngine
    RenderEngine --> DWMCompositor
    DWMCompositor --> GPU
```

---

## Further Reading
- [Rendering Pipeline](file:///Users/tanmay/Documents/Github/Cutom-widget-Aether/docs/Rendering.md)
- [Security & Sandboxing](file:///Users/tanmay/Documents/Github/Cutom-widget-Aether/docs/Security.md)
- [Multi-Language Plugin SDK](file:///Users/tanmay/Documents/Github/Cutom-widget-Aether/docs/PluginSDK.md)
- [IPC Specification](file:///Users/tanmay/Documents/Github/Cutom-widget-Aether/docs/IPC.md)
- [Benchmarking Methodology](file:///Users/tanmay/Documents/Github/Cutom-widget-Aether/docs/Benchmarking.md)
