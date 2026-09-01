# Package Signatures & Update Security

**Ed25519 Cryptographic Verification, Package Integrity, and Auto-Updater Security**

---

## 1. Ed25519 Digital Signatures (`package_manager`)

All `.cwp` (Custom Widget Package) archives installed via CLI or GUI Marketplace must pass cryptographic signature verification before extraction:

- **Algorithm**: Ed25519 (Edwards-curve Digital Signature Algorithm).
- **Public Key Registry**: Built-in trusted root public keys stored in `package_manager::verifier`.
- **Validation Flow**:
  1. Read archive header and extract embedded signature block.
  2. Compute SHA-256 digest of payload data.
  3. Verify signature using verified author public key.
  4. Reject untrusted or modified packages before disk extraction.

---

## 2. Auto-Updater Security (`production_engine`)

The `AutoUpdater` in `crates/production_engine` validates incoming update manifests:
- TLS 1.3 encrypted manifest downloads.
- SHA-256 checksum validation of target update binaries.
- Safe rollback to previous working version on boot failure.
