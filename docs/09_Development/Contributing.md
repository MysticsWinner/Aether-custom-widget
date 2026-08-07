# Contribution Guidelines & Process Governance

**Purpose**: Guides pull request submission, governance rules, and testing standards.  
**Audience**: All Contributors, Maintainers.  
**Prerequisites**: [Build.md](Build.md).  
**Related Documents**: [Coding_Standards.md](Coding_Standards.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Governance Guide  
**Owner**: Governance Lead  

---

## 1. Governance Rules (`AGENTS.md`)

1. **Mandatory Testing**: Every PR must include unit/integration tests. `cargo test --workspace` must pass 100%.
2. **Zero Warnings**: Code must compile with 0 compilation errors.
3. **Documentation Updates**: Update documentation in `docs/` alongside any code changes.
4. **Report Test Counts**: Always specify passing test count in PR descriptions (e.g. "184/184 tests pass").

---

## Future Work
- Add GitHub Actions PR approval bot enforcing AGENTS.md rules automatically.

## Known Issues
- None.

## References
- [AGENTS.md](../../.agents/AGENTS.md)

## Related Documents
- [Build.md](Build.md)
