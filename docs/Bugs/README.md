# Aether — Bug & Issue Tracker

This directory records discovered bugs, suspected issues, architectural concerns, technical debt, incomplete features, and future ideas for the Aether project.

## Categories

| Category | Description | File |
|:---|:---|:---|
| **Confirmed Bugs** | Verified defects in the current implementation | [BUGS.md](BUGS.md) |
| **Technical Debt** | Code quality issues, shortcuts, and deferred cleanup | [TECH_DEBT.md](TECH_DEBT.md) |
| **Architectural Concerns** | Design-level issues that may affect long-term maintainability | [ARCHITECTURE_CONCERNS.md](ARCHITECTURE_CONCERNS.md) |
| **Future Ideas** | Enhancement proposals and feature requests not yet committed | [FUTURE_IDEAS.md](FUTURE_IDEAS.md) |

## How to Record an Issue

1. Choose the appropriate file for the category.
2. Add a new entry with a unique ID (e.g. `BUG-001`, `TD-006`, `AC-001`, `FI-001`).
3. Include: **Summary**, **Severity** (Critical / High / Medium / Low), **Component** (crate name), **Evidence** (what you observed and where), and **Status** (Open / In Progress / Resolved / Deferred).
4. Optionally include reproduction steps, root cause analysis, or proposed fix.

## Severity Definitions

| Severity | Definition |
|:---|:---|
| **Critical** | Data loss, security vulnerability, or crash affecting the host daemon |
| **High** | Significant functional defect or architectural weakness |
| **Medium** | Non-blocking defect, inaccurate documentation, or performance concern |
| **Low** | Cosmetic issue, minor code quality concern, or dead code |
