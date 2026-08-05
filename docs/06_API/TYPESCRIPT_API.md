# Aether — TypeScript / Web SDK Specification

**Planned Web/JS Widget Binding Architecture**

---

## 1. Planned TypeScript API Specification (`bindings/typescript`)

To support Web-based widget components (HTML/CSS/JS), the planned TypeScript SDK exposes typed interfaces matching the Rust Widget SDK:

```typescript
export interface Widget {
  onLoad(): Promise<void>;
  onMount(): Promise<void>;
  onUpdate(ctx: TickContext): Promise<void>;
  onUnmount(): Promise<void>;
  onUnload(): Promise<void>;
}

export interface TelemetrySnapshot {
  cpuUsagePct: number;
  gpuUsagePct: number;
  memoryUsedMb: number;
  memoryTotalMb: number;
}
```

---

## 2. WebAssembly Runtime Bridge

Web widgets execute inside a sandboxed WebAssembly / WebView2 container communicating with the host engine daemon over WebSocket / IPC pipe buffers.
