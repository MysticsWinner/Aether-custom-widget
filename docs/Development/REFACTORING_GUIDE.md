# Aether — Refactoring Guide: Transitioning Simulated Skeletons to Real Hardware Implementation

**Step-by-Step Technical Guide for Upgrading Prototype Subsystems**

---

## 1. Overview

While core engine loop, IPC server, WinUI 3 dashboard, and CPU/RAM telemetry operate on real Win32 APIs, several subsystems currently operate as **Functional Skeletons** (simulated math or in-memory stubs). This guide outlines the exact refactoring steps to upgrade each simulated module to hardware-native implementation.

---

## 2. Refactoring Guides by Subsystem

### 2.1 Refactoring `GpuProvider` (Simulated Math → Real DXGI Telemetry)

- **Target File**: [crates/system_providers/src/providers.rs](file:///d:/Code/Aether-custom-widget/crates/system_providers/src/providers.rs)
- **Current State**: Mock sine wave math formula (`sin(tick * 0.07) * 45`).
- **Upgrade Steps**:
  1. Add `windows::Win32::Graphics::Dxgi` dependency to `system_providers/Cargo.toml`.
  2. Implement `CreateDXGIFactory1` and query `IDXGIFactory1::EnumAdapters1`.
  3. Query performance adapter statistics using `QueryVideoMemoryInfo`.
  4. Write actual GPU utilization percentage to `SharedTelemetryCache`.

---

### 2.2 Refactoring `NetworkProvider` (Modulo Arithmetic → Win32 `GetIfTable2`)

- **Target File**: [crates/system_providers/src/providers.rs](file:///d:/Code/Aether-custom-widget/crates/system_providers/src/providers.rs)
- **Current State**: Mock counter modulo (`(tick * 1024) % (1024 * 1024)`).
- **Upgrade Steps**:
  1. Add `windows::Win32::NetworkManagement::IpHelper` dependency.
  2. Call `GetIfTable2` to retrieve interface statistics (`MIB_IF_ROW2`).
  3. Calculate delta bytes received/sent between tick intervals.

---

### 2.3 Refactoring `PluginSupervisor` (In-Memory PID → AppContainer Sandbox)

- **Target File**: [crates/plugin_runtime/src/supervisor.rs](file:///d:/Code/Aether-custom-widget/crates/plugin_runtime/src/supervisor.rs)
- **Current State**: In-memory PID assignment (`5000 + n`).
- **Upgrade Steps**:
  1. Use Win32 `CreateAppContainerProfile` to allocate sandboxed Security Identifier (SID).
  2. Spawn widget process with `CreateProcessAsUserW` using restricted token.
  3. Assign process handle to a Windows Job Object (`CreateJobObjectW`) with CPU/memory limits.

---

### 2.4 Refactoring `Direct2DRenderer` (In-Memory Stats → Desktop Window Compositing)

- **Target File**: [crates/core_engine/src/rendering/d2d_renderer.rs](file:///d:/Code/Aether-custom-widget/crates/core_engine/src/rendering/d2d_renderer.rs)
- **Current State**: Dirty rect math & frame budgets in memory.
- **Upgrade Steps**:
  1. Call FFI to native C++ `workerw_hook.cpp` to obtain desktop `WorkerW` `HWND`.
  2. Initialize `ID3D11Device` and `ID2D1DeviceContext`.
  3. Bind `IDCompositionDevice` visual surface to desktop window handle.
  4. Execute `DrawCommand` batch during `end_frame()`.
