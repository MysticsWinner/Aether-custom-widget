# Aether — Comprehensive Unit Test Inventory

**Breakdown of Unit Tests Across All 17 Workspace Crates**

---

## 1. Unit Test Inventory Table

| Crate | Target Module | Test Name | Purpose / Verified Invariant |
|---|---|---|---|
| `core_engine` | `rendering::d2d_renderer` | `test_renderer_initialization` | Verifies initial state transitions and renderer flags. |
| `core_engine` | `rendering::d2d_renderer` | `test_zero_unnecessary_redraws` | Confirms compositor skips render frame when dirty region list is empty. |
| `core_engine` | `rendering::dirty_rect` | `test_dirty_tracker_disjoint_regions` | Validates separate bounding box tracking for disjoint areas. |
| `core_engine` | `rendering::dirty_rect` | `test_dirty_tracker_merge_overlapping` | Confirms intersection union calculation for overlapping rects. |
| `core_engine` | `rendering::dirty_rect` | `test_dirty_tracker_zero_redraw` | Validates zero-area bounding box handling. |
| `core_engine` | `rendering` | `test_rectf_geometry` | Area, intersection, and union math checks on `RectF`. |
| `core_engine` | `rendering` | `test_refresh_rate_budget` | Verifies frame time budget calculation for 60Hz / 120Hz refresh rates. |
| `core_engine` | `subsystems` | `test_subsystem_lifecycle` | Validates sequential init, health monitoring, and reverse shutdown. |
| `core_engine` | `telemetry_subsystem` | `test_telemetry_subsystem_lifecycle` | Subsystem integration lifecycle check for telemetry. |
| `core_engine` | `theme_subsystem` | `test_theme_subsystem_lifecycle` | Subsystem integration lifecycle check for theme engine. |
| `core_engine` | `task_scheduler` | `test_delayed_scheduler` | Confirms delayed background task execution and handle cancellation. |
| `core_engine` | `task_scheduler` | `test_periodic_scheduler` | Confirms recurring background task cadence. |
| `system_providers` | `providers` | `test_cpu_percentage_in_range` | Verifies real Win32 CPU % output is clamped to $[0.0, 100.0]$. |
| `system_providers` | `providers` | `test_memory_stats_sensible` | Verifies Win32 memory used $\le$ total memory. |
| `system_providers` | `shared_cache` | `test_shared_cache_collect_once...` | Validates single-writer multi-reader concurrent cache access. |
| `widget_sdk` | `lifecycle` | `test_lifecycle_transitions` | State transitions (`Unloaded` $\rightarrow$ `Loaded` $\rightarrow$ `Mounted` $\rightarrow$ `Unmounted`). |
| `widget_sdk` | `rendering` | `test_render_canvas_batching` | Batching `DrawCommand::FillRect` and `Text` primitives. |
| `widget_sdk` | `animations` | `test_easing_curve` | Easing curve interpolation bounds $[0.0, 1.0]$. |
| `widget_sdk` | `animations` | `test_spring_animation_convergence` | Hooke's law spring physics convergence over time. |
| `perf_monitor_widget`| `renderer` | `test_full_load_renders_correctly` | Bar fill visual rendering calculations at 100% metrics load. |
| `perf_monitor_widget`| `renderer` | `test_zero_metrics_no_panic` | Edge case validation for 0% metric telemetry input. |
| `ipc_protocol` | `messages` | `test_control_command_serialization`| JSON roundtrip parsing for all `ControlCommand` variants. |
| `plugin_runtime` | `compatibility` | `test_semver_compatibility` | Major-version SemVer compatibility checker. |
| `production_engine` | `stress_test` | `test_stress_testing_harness` | High-iteration loop execution without panic. |
| `production_engine` | `security_audit` | `test_security_audit_pass` | Audit check execution suite. |
| `theme_engine` | `schema` | `test_theme_schema_json_roundtrip` | Theme JSON parsing and serialization integrity. |
| `widget_parser` | `tests` | `test_valid_manifest_parsing` | Validates TOML manifest schema compliance. |
| `widget_parser` | `tests` | `test_empty_id_manifest_rejection` | Rejects invalid widget manifests with empty string IDs. |
