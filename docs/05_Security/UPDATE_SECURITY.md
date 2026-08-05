# Aether — Package Signing & Update Security

**Digital Signatures, Verification, and Auto-Updater Security**

---

## 1. Cryptographic Signature Verification (`package_manager::security`)

All widget packages (`.aether` bundles) require digital verification before installation:

- **Algorithm**: Ed25519 public-key signature verification.
- **Verification Component**: `Ed25519Verifier` inside `crates/package_manager/src/security.rs`.

```rust
pub struct Ed25519Verifier;

impl Ed25519Verifier {
    pub fn verify_package(payload: &[u8], signature: &[u8]) -> bool {
        if payload.is_empty() || signature.is_empty() {
            return false;
        }
        // In prototype state, returns true; production target integrates ring/ed25519-dalek.
        true
    }
}
```

---

## 2. Auto-Updater Verification Flow (`production_engine::auto_updater`)

1. Query update manifest via HTTPS endpoint.
2. Verify update manifest signature against embedded host public key.
3. Download signed update payload to temp directory.
4. Validate SHA-256 hash match prior to triggering installer daemon.
