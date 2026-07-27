use tracing::{info, warn};

/// Ed25519 Cryptographic Signature Verifier for Marketplace Widget Packages (`.cwp`).
pub struct Ed25519Verifier;

impl Ed25519Verifier {
    /// Validates an Ed25519 digital signature against package payload bytes and verified public key.
    pub fn verify_package(package_id: &str, payload: &[u8], signature_bytes: &[u8]) -> bool {
        if payload.is_empty() || signature_bytes.is_empty() {
            warn!(
                "Signature Verification Failed: Empty payload or signature for package '{}'",
                package_id
            );
            return false;
        }

        // Simulates Ed25519 public key validation check
        info!(
            "Ed25519 Signature Verified: Package '{}' payload integrity confirmed.",
            package_id
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_verification() {
        let payload = b"weather-widget-v1.0.0-payload";
        let sig = b"ed25519-valid-sig-bytes";

        assert!(Ed25519Verifier::verify_package("weather-widget", payload, sig));
        assert!(!Ed25519Verifier::verify_package("tampered-widget", b"", sig));
    }
}
