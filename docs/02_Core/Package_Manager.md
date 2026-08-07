# Package Manager Subsystem (`package_manager`)

**Purpose**: npm-style widget package installer, manifest parser, and Ed25519 cryptographic verifier.  
**Audience**: CLI Users, Package Authors, Security Engineers.  
**Prerequisites**: [Security_Model.md](../07_Security/Security_Model.md).  
**Related Documents**: [Marketplace.md](Marketplace.md).  
**Last Updated**: 2026-08-07  
**Status**: Active / Core Subsystem  
**Owner**: Package & Distribution Team  

---

## 1. Package Installation Workflow

1. Download widget `.aether` package archive.
2. Verify Ed25519 publisher signature against trusted public key keyring.
3. Validate TOML manifest schema (`widget.toml`).
4. Extract files to sandboxed widget directory.

---

## Future Work
- Add differential package update downloads.

## Known Issues
- None.

## References
- [crates/package_manager/src/lib.rs](file:///d:/Code/Aether-custom-widget/crates/package_manager/src/lib.rs)

## Related Documents
- [Marketplace.md](Marketplace.md)
