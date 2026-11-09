/**
 * Next-Gen Windows Desktop Customization Platform - TypeScript & WebView2 Widget SDK (v0.7.0)
 */

export type WidgetState =
  | 'Unloaded'
  | 'Loading'
  | 'Loaded'
  | 'Mounting'
  | 'Active'
  | 'Mounted'
  | 'Paused'
  | 'Degraded'
  | 'Unmounting'
  | 'Unloading'
  | 'Unmounted'
  | 'Error'
  | 'Quarantined';

export interface TickContext {
  timestampMs: number;
  deltaTimeMs: number;
  frameIndex: number;
}

export interface RectF {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Color {
  r: number;
  g: number;
  b: number;
  a: number;
}

export interface DrawCommand {
  kind: 'Clear' | 'DrawRect' | 'DrawText' | 'DrawImage' | 'PushClip' | 'PopClip';
  rect?: RectF;
  color?: Color;
  cornerRadius?: number;
  text?: string;
  fontFamily?: string;
  fontSize?: number;
  resourceId?: string;
  opacity?: number;
}

export interface RenderCanvas {
  clear(color: Color): void;
  drawRect(rect: RectF, color: Color, cornerRadius?: number): void;
  drawText(text: string, fontFamily: string, fontSize: number, rect: RectF, color: Color): void;
  drawImage(resourceId: string, rect: RectF, opacity?: number): void;
  pushClip(rect: RectF): void;
  popClip(): void;
  invalidate(rect: RectF): void;
  getCommands(): readonly DrawCommand[];
}

export declare class BatchRenderCanvas implements RenderCanvas {
  clear(color: Color): void;
  drawRect(rect: RectF, color: Color, cornerRadius?: number): void;
  drawText(text: string, fontFamily: string, fontSize: number, rect: RectF, color: Color): void;
  drawImage(resourceId: string, rect: RectF, opacity?: number): void;
  pushClip(rect: RectF): void;
  popClip(): void;
  invalidate(rect: RectF): void;
  getCommands(): readonly DrawCommand[];
  toJsonBatch(): string;
}

export interface SettingsStore {
  get(key: string): unknown;
  set(key: string, value: unknown): void;
}

export declare class InMemorySettingsStore implements SettingsStore {
  get(key: string): unknown;
  set(key: string, value: unknown): void;
}

export interface SpringParams {
  stiffness: number;
  damping: number;
  mass: number;
}

export interface AnimationController {
  springAnimate(initial: number, target: number, params?: SpringParams): number;
  evaluateEasing(t: number, curve: 'Linear' | 'EaseIn' | 'EaseOut'): number;
}

export interface ResourceManager {
  loadAsset(path: string): Promise<Uint8Array>;
  resolveColorToken(tokenId: string): Color;
}

export interface Widget {
  readonly state: WidgetState;
  onLoad?(): void | Promise<void>;
  onMount?(): void | Promise<void>;
  onUpdate?(ctx: TickContext, canvas: RenderCanvas): void;
  onUnmount?(): void | Promise<void>;
  onUnload?(): void | Promise<void>;
  onEvent?(topic: string, payload: unknown): void;
}
