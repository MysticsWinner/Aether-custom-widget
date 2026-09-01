# TypeScript / JavaScript Widget SDK Specification

**TypeScript Definitions & WebView2 Bridge API**

---

## 1. Type Definitions (`bindings/typescript/custom-widget-sdk/index.d.ts`)

```typescript
export interface WidgetMetadata {
  id: string;
  name: string;
  version: string;
  author: string;
  description?: string;
}

export interface TelemetrySnapshot {
  timestampMs: number;
  cpuUsagePct: number;
  gpuUsagePct: number;
  memoryUsedMb: number;
  memoryTotalMb: number;
  netRecvBytesPerSec: number;
  netSentBytesPerSec: number;
}

export interface WidgetInstance {
  metadata: WidgetMetadata;
  onLoad(): Promise<void>;
  onMount(container: HTMLElement): Promise<void>;
  onUpdate(snapshot: TelemetrySnapshot): void;
  onUnmount(): Promise<void>;
  onUnload(): Promise<void>;
}
```
