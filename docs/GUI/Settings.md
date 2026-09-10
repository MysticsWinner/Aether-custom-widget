# Settings Management & Persistence

**Purpose**: Specifications for user settings persistence, JSON schema validation, and config migrations.  
**Audience**: Application Developers.  
**Prerequisites**: [Dashboard.md](Dashboard.md).  
**Related Documents**: [Engine.md](../Core/Engine.md).  
**Last Updated**: 2026-09-04  
**Status**: Active / Production  
**Owner**: Config & Settings Team  

---

## 1. Atomic Settings Persistence (`config_manager`)

Settings changes are written atomically via temp file → sync → rename workflow with 5-generation backup rotation.

---

## 2. Dashboard Widget Settings Service (`IWidgetSettingsService`)

The WinUI 3 Dashboard provides client-side per-widget display options management through `IWidgetSettingsService` and `WidgetSettingsService`.

### 2.1 Storage & Schema
- **Path**: `%LOCALAPPDATA%\Aether\widget_settings\<widget_id>.json`
- **Schema** (`WidgetDisplayOptions`):
  - `widget_id`: Unique identifier of the desktop widget.
  - `opacity`: Double in range `[0.0, 1.0]`.
  - `scale`: Double in range `[0.1, 3.0]`.
  - `locked`: Boolean indicating if widget desktop dragging/movement is locked.
  - `enabled`: Boolean controlling active rendering state.
  - `quick_swap`: Boolean enabling quick swapping behavior.

### 2.2 Core Service Operations
- `LoadAllSettingsAsync()`: Scans and deserializes all JSON manifests in the widget settings directory.
- `GetSettingsAsync(widgetId)`: Returns active or default display options for a given widget.
- `SaveSettingsAsync(widgetId, options)`: Persists settings to disk and dispatches `UpdateWidgetDisplayOptions` via IPC.
- `SetPositionAsync(widgetId, x, y)`: Synchronises desktop widget window coordinates `(x, y)` to Core Engine.
- `SetLockedAsync(widgetId, locked)`: Updates lock flag on disk and synchronises via `UpdateWidgetDisplayOptions` IPC.
- `SetOpacityAsync(widgetId, opacity)`: Clamps opacity to `[0.0, 1.0]` and updates disk/IPC.
- `SetScaleAsync(widgetId, scale)`: Clamps scale to `[0.1, ...]` and synchronises.
- `SetEnabledAsync(widgetId, enabled)`: Toggles active widget status on disk and via IPC.

---

## Future Work
- Add cloud sync toggle options per settings key.

## Known Issues
- None.

## References
- [crates/config_manager/src/lib.rs](file:///d:/Code/Aether-custom-widget/crates/config_manager/src/lib.rs)

## Related Documents
- [Dashboard.md](Dashboard.md)
