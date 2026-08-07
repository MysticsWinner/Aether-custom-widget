# Stress Testing & Chaos Engineering (`production_engine`)

**Purpose**: High-concurrency stress testing, memory leak detection, and failure injection harness.  
**Audience**: QA Engineers, Security Testers.  
**Prerequisites**: [Test_Structure.md](Test_Structure.md).  
**Related Documents**: [Sandboxing.md](../07_Security/Sandboxing.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Testing Guide  
**Owner**: QA & Resilience Team  

---

## 1. Chaos Harness (`chaos_harness.rs`)

Injects synthetic faults into running engine instances:
- Forced plugin segfaults and panics.
- Sudden Named Pipe IPC disconnections.
- High memory allocation spikes.

Verifies host daemon surviving 100% uptime with sub-5ms worker recovery.

---

## Future Work
- Add long-running 72-hour continuous stress test job to CI pipeline.

## Known Issues
- None.

## References
- [crates/production_engine/src/chaos_harness.rs](file:///d:/Code/Aether-custom-widget/crates/production_engine/src/chaos_harness.rs)

## Related Documents
- [Test_Structure.md](Test_Structure.md)
