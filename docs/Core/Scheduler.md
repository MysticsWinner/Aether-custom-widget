# Scheduler Subsystem (`widget_sdk` & `system_providers`)

**Purpose**: Specifications for `FrameScheduler` and `TickRateAdvisor` adaptive tick scheduling.  
**Audience**: Engine Engineers, Performance Leads.  
**Prerequisites**: [Engine.md](Engine.md).  
**Related Documents**: [Telemetry.md](Telemetry.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Core Subsystem  
**Owner**: Core Engine Team  

---

## 1. `TickRateAdvisor` Adaptive Scaling

- **Active User Mode**: 10 ms cycle (100 Hz active sampling rate).
- **Idle User Mode**: 50 ms cycle (20 Hz idle sampling rate).
- **Battery Saver Mode**: 100 ms cycle (10 Hz low-power sampling rate).

---

## 2. `FrameScheduler` Enforced Target FPS

`FrameScheduler` monitors frame delta time and skips rendering ticks when frame budget thresholds are breached to ensure stable 60Hz/144Hz desktop presentation.

---

## Future Work
- Add Variable Refresh Rate (G-Sync/FreeSync) sync integration.

## Known Issues
- None.

## References
- [crates/widget_sdk/src/frame_scheduler.rs](file:///d:/Code/Aether-custom-widget/crates/widget_sdk/src/frame_scheduler.rs)
- [crates/system_providers/src/tick_advisor.rs](file:///d:/Code/Aether-custom-widget/crates/system_providers/src/tick_advisor.rs)

## Related Documents
- [Engine.md](Engine.md)
