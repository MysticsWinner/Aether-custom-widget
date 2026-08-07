# Release Process & Versioning Workflow

**Purpose**: Workflow guide for version tagging, release candidate builds, and package publishing.  
**Audience**: Release Engineers, Maintainers.  
**Prerequisites**: [Workspace.md](Workspace.md).  
**Related Documents**: [Release_Notes.md](../00_Project/Release_Notes.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Release Procedure  
**Owner**: Release Engineering Lead  

---

## 1. Release Process Steps

1. Verify all workspace tests pass (`cargo test --workspace`).
2. Run production security audit (`production_engine`).
3. Update version strings in `Cargo.toml` workspace manifest.
4. Generate `Changelog.md` and `Release_Notes.md` entry.
5. Create Git release tag (`git tag -a v0.7.0 -m "Release v0.7.0"`).
6. Build production binaries & NSIS installer packages.

---

## Future Work
- Automate release builds via GitHub Actions release workflow.

## Known Issues
- None.

## References
- [crates/production_engine/src/lib.rs](file:///d:/Code/Aether-custom-widget/crates/production_engine/src/lib.rs)

## Related Documents
- [Release_Notes.md](../00_Project/Release_Notes.md)
