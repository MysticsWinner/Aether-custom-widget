# Contributing to Aether Platform

## Guidelines

1. **Architecture First**: Review [Architecture Specification](file:///Users/tanmay/Documents/Github/Cutom-widget-Aether/docs/Architecture.md) before implementing core changes.
2. **Zero Allocation Hot Loops**: Hot path code must maintain zero allocations where possible.
3. **Testing & Verification**: Every pull request must pass existing unit tests and stress profiling scripts.

---

## Workspace Build Setup

```bash
# Check compilation
cargo check --workspace

# Run integration test suite
cargo test --workspace

# Build release binaries
cargo build --release --workspace
```
