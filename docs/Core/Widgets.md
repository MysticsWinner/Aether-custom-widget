# Aether Showcase & Built-in Widgets Reference

**Architectural Specification, Telemetry Bindings, and Rendering Profiles for All 9 Built-in Desktop Widgets**

---

## 1. Overview & Architectural Principles

All Aether widgets adhere to the platform's core architecture:
1. **"Collect Once, Publish Everywhere"**: Widgets never invoke low-level Win32 or hardware APIs directly. Instead, they read exclusively from the lock-free `SharedTelemetryCache`.
2. **6-Pillar Lifecycle**: Every widget implements `WidgetLifecycle` (`on_load`, `on_mount`, `on_update`, `on_unmount`, `on_unload`, `state`).
3. **Decoupled Batch Rendering**: Widgets do not touch Direct2D render targets directly; they emit `DrawCommand` batches into `BatchRenderCanvas`, which the core engine composites onto the DirectComposition desktop overlay window.
4. **WCAG 2.1 Contrast Guard**: Text rendering is automatically checked and adapted by `ContrastGuard` against dynamic backgrounds.
5. **Adaptive Frame Scheduling**: Each widget declares a target refresh interval, which is dynamically scaled by `AdaptiveRefreshScheduler` during low power, window occlusion, or full-screen gaming mode.

---

## 2. Widget Catalog & Specifications

### 2.1 `perf_monitor_widget` (Performance Monitor)
- **Path**: `crates/perf_monitor_widget/`
- **Identifier**: `aether.builtin.perf_monitor`
- **Telemetry Bindings**:
  - `cache.get_cpu_pct()` — Total CPU utilization [0–100%]
  - `snapshot.gpu_usage_pct` — DXGI video engine utilization [0–100%]
  - `snapshot.memory_used_mb` / `snapshot.memory_total_mb` — Physical RAM utilization
  - `snapshot.net_recv_bytes_per_sec` / `snapshot.net_sent_bytes_per_sec` — Network throughput
- **Visual Presentation**:
  - Dark glassmorphic acrylic card (`RectF`, `MaterialSpec`)
  - Real-time gauge bars with smooth easing transitions
  - Multi-point historical sparkline graph rendering recent metric history

### 2.2 `weather_widget` (Meteorological Forecast)
- **Path**: `crates/weather_widget/`
- **Identifier**: `aether.builtin.weather`
- **Telemetry Bindings**:
  - `snapshot.custom_metrics["weather.temperature_c"]`
  - `snapshot.custom_metrics["weather.humidity_pct"]`
  - `snapshot.custom_metrics["weather.condition_code"]`
- **Visual Presentation**:
  - Large ambient temperature readout with high-contrast glyphs
  - Vector SVG weather iconography (sun, cloud, rain, lightning)
  - 5-day horizontal forecast card strip

### 2.3 `network_monitor_widget` (Network Bandwidth)
- **Path**: `crates/network_monitor_widget/`
- **Identifier**: `aether.builtin.network_monitor`
- **Telemetry Bindings**:
  - `snapshot.net_recv_bytes_per_sec` — Inbound throughput
  - `snapshot.net_sent_bytes_per_sec` — Outbound throughput
  - `snapshot.custom_metrics["net.active_adapter_type"]` (Ethernet / Wi-Fi)
- **Visual Presentation**:
  - Dual speedometer gauge rings for download and upload rates
  - Filled area spline graph (`draw_spline_area`) illustrating bandwidth spikes
  - Formatted byte units (`KB/s`, `MB/s`, `GB/s`)

### 2.4 `ai_assistant_widget` (Desktop AI Assistant)
- **Path**: `crates/ai_assistant_widget/`
- **Identifier**: `aether.builtin.ai_assistant`
- **Telemetry Bindings**:
  - Inter-process query channel, AI token streaming buffers
- **Visual Presentation**:
  - Conversational chat bubble cards with smooth entry animations
  - Markdown-style synthesized response layout
  - Quick action suggestion chips and prompt entry boundary

### 2.5 `audio_visualizer_widget` (WASAPI FFT Spectrum & SMTC)
- **Path**: `crates/audio_visualizer_widget/`
- **Identifier**: `aether.builtin.audio_visualizer`
- **Telemetry Bindings**:
  - `snapshot.audio_spectrum.fft_bins_16` — 16-band audio frequency amplitudes
  - `snapshot.media_playback.title` / `artist` — Windows SMTC track info
- **Visual Presentation**:
  - 16 vertical gradient spectrum bars with gravity peak-hold physics
  - Track metadata typography overlay with album art placeholder rect
  - Dynamic pulse glow synchronized with bass frequencies

### 2.6 `hardware_pro_widget` (GPU VRAM & Core Topology)
- **Path**: `crates/hardware_pro_widget/`
- **Identifier**: `aether.builtin.hardware_pro`
- **Telemetry Bindings**:
  - `snapshot.gpu_telemetry.vram_dedicated_used_mb` / `vram_dedicated_total_mb`
  - Per-core CPU utilization array
- **Visual Presentation**:
  - Multi-core CPU topology matrix grid (per-core load heatmap)
  - Dedicated GPU VRAM utilization gauge and memory bus clock
  - Thermal throttling indicator warning badges

### 2.7 `dock_launcher_widget` (Dynamic App Dock)
- **Path**: `crates/dock_launcher_widget/`
- **Identifier**: `aether.builtin.dock_launcher`
- **Interactive Capabilities**:
  - Pointer hit-testing tree (`HitTestTree`) for mouse hover and click events
  - Spring physics magnification: neighboring icons scale dynamically on hover
- **Telemetry Bindings**:
  - Process list from `system_providers` to determine running application state
- **Visual Presentation**:
  - Floating pill-shaped dark glass dock capsule
  - Vector SVG application icons (`draw_path`)
  - Running indicator luminous dots beneath active application icons

### 2.8 `weather_particles_widget` (Atmospheric Particle Simulation)
- **Path**: `crates/weather_particles_widget/`
- **Identifier**: `aether.builtin.weather_particles`
- **Telemetry Bindings**:
  - Weather condition state: `Rain`, `Snow`, `Embers`, `Clear`
- **Simulation Mechanics**:
  - 100+ particle physics simulation with velocity, angle, gravity, and lifetime
  - Particle recycling upon exiting screen boundaries
- **Visual Presentation**:
  - Translucent rain streaks, floating snowflakes, or glowing embers
  - Alpha-blended DirectComposition overlay across desktop background

### 2.9 `crypto_stocks_widget` (Financial Market Ticker)
- **Path**: `crates/crypto_stocks_widget/`
- **Identifier**: `aether.builtin.crypto_stocks`
- **Telemetry Bindings**:
  - Crypto and market quotes cache (`BTC`, `ETH`, `SOL`, `S&P500`, `NASDAQ`)
  - 24-hour delta percentages and volume metrics
- **Visual Presentation**:
  - Multi-column ticker tape with asset symbol and live prices
  - Color-coded trend badges (green for positive, red for negative)
  - Micro sparkline curves displaying recent trading session trajectory

---

## 3. Configuration & IPC Control

All widgets can be controlled dynamically via the Win32 Named Pipe IPC channel:
- **Query Registered Widgets**: `"ListWidgets"`
- **Update Display Options**: `{"UpdateWidgetDisplayOptions": {"widget_id": "...", "opacity": 0.85, "scale": 1.0, "locked": false, "enabled": true}}`
- **Quick Position/Config Swap**: `{"QuickSwapWidget": {"from_id": "...", "to_id": "...", "mode": "position"}}`
- **Dynamic Opacity Adjustment**: `{"SetWidgetOpacity": {"widget_id": "...", "opacity": 0.9}}`
- **Reset Configuration**: `{"ResetWidgetConfig": {"widget_id": "..."}}`
