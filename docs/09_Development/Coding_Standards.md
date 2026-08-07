# Rust & C# Coding Standards

**Purpose**: Style guidelines, linting rules, and architectural constraints for Aether.  
**Audience**: All Developers.  
**Prerequisites**: [Contributing.md](Contributing.md).  
**Related Documents**: [Workspace.md](Workspace.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Style Guide  
**Owner**: Quality Engineering Lead  

---

## 1. Rust Coding Standards

- **Formatting**: Run `cargo fmt` prior to committing.
- **Linting**: Run `cargo clippy --workspace -- -D warnings`.
- **Error Handling**: Use `anyhow::Result` for application code and `thiserror` for library crates. Never use `unwrap()` or `expect()` in production paths.
- **Async Safety**: Avoid blocking synchronous code inside Tokio async tasks; use `tokio::task::spawn_blocking` for GDI or heavy Win32 calls.

---

## Future Work
- Enforce `clippy` checks in CI pipeline.

## Known Issues
- None.

## References
- [AGENTS.md](../../.agents/AGENTS.md)

## Related Documents
- [Contributing.md](Contributing.md)
