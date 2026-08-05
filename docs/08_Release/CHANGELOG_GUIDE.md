# Aether — Changelog Formatting Guidelines

**Standard Maintainer Conventions for Release Notes**

---

## 1. Standard Keep-a-Changelog Format

All release notes follow the [Keep a Changelog](https://keepachangelog.com/) standard categorized by change type:

- `Added`: New features, endpoints, or crates.
- `Changed`: Modifications to existing features or refactored implementations.
- `Deprecated`: Features scheduled for removal in future major releases.
- `Removed`: Deprecated capabilities removed in major releases.
- `Fixed`: Bug fixes and patch releases.
- `Security`: Vulnerability fixes and sandbox security hardening.

---

## 2. Sample Changelog Entry

```markdown
## [0.15.0-rc1] - 2026-08-05

### Added
- Complete modular documentation suite under `docs/` (00_Project through 08_Release).
- WinUI 3 management dashboard with live metric poller and Mica backdrop.
- Ratatui terminal dashboard client for CLI monitoring over Named Pipe IPC.

### Fixed
- CPU percentage calculation clamping in `system_providers::CpuProvider`.
```
