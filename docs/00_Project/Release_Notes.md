# Release Notes — Aether v0.7.0 Release Candidate

**Purpose**: Detailed release notes highlighting major architecture features in Aether v0.7.0.  
**Audience**: All Users, Contributors, System Administrators.  
**Prerequisites**: [Changelog.md](Changelog.md).  
**Related Documents**: [Detailed_Project_Report.md](Detailed_Project_Report.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Official Release Notes  
**Owner**: Release Engineering Lead  

---

## Highlights of Aether v0.7.0 RC

1. **Enterprise Governance & Policy Management (`crates/enterprise`)**:
   - `PolicyEngine` enforces Group Policy & MDM JSON security rules.
   - `AuditLogger` maintains SHA-256 block hash-chained tamper-evident logs.
   - `AuthGate` integrates Windows Hello biometric prompts for privileged operations.

2. **Developer Tools & Live Inspection (`crates/dev_tools`)**:
   - `DevHotReloader` monitors widget source code directories for instant hot-reloading.
   - `WidgetInspector` provides Chrome-style DOM inspection and frame timing profiling.

3. **AI Expansion & Synthesis Engine (`crates/ai_engine`)**:
   - `WidgetSynthesizer` generates full widget manifests from natural language prompts.
   - `WallpaperThemeGenerator` creates harmonized color palettes from wallpaper images.

4. **Self-Healing Watchdog & Chaos Engineering (`crates/watchdog` & `crates/event_recorder`)**:
   - Two-process heartbeat supervisor daemon (`aether_watchdog.exe`) auto-restarts host daemon on crashes.
   - Time-travel event stream recorder for bug report playback.

5. **Adaptive Frame Scheduler & Telemetry (`crates/system_providers` & `crates/widget_sdk`)**:
   - `TickRateAdvisor` dynamically scales tick rates (10ms active → 100ms battery saver).
   - Extended telemetry collectors for App counts, Battery specs, Audio volume, and Multi-GPU topology.

---

## Future Work
- Finalize production installer bundle.

## Known Issues
- None.

## References
- [Changelog.md](Changelog.md)

## Related Documents
- [Detailed_Project_Report.md](Detailed_Project_Report.md)
