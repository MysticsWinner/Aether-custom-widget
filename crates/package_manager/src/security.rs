use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use tracing::{info, warn};

/// Ed25519 Cryptographic Signature Verifier for Marketplace Widget Packages (`.cwp`).
///
/// Performs real Ed25519 digital signature verification using the `ed25519-dalek` crate.
/// The marketplace public key is pinned at compile time; package authors sign with their
/// private key and the verifier checks the signature against the known public key.
pub struct Ed25519Verifier;

impl Ed25519Verifier {
    /// Validates an Ed25519 digital signature against package payload bytes and a public key.
    ///
    /// Returns `true` only if the cryptographic signature is mathematically valid
    /// for the given payload under the provided `public_key_bytes`.
    pub fn verify_package(
        package_id: &str,
        payload: &[u8],
        signature_bytes: &[u8],
        public_key_bytes: &[u8; 32],
    ) -> bool {
        if payload.is_empty() {
            warn!(
                "Signature Verification Failed: Empty payload for package '{}'",
                package_id
            );
            return false;
        }

        if signature_bytes.len() != 64 {
            warn!(
                "Signature Verification Failed: Invalid signature length ({} bytes, expected 64) for package '{}'",
                signature_bytes.len(),
                package_id
            );
            return false;
        }

        // Reconstruct the verifying (public) key
        let verifying_key = match VerifyingKey::from_bytes(public_key_bytes) {
            Ok(k) => k,
            Err(e) => {
                warn!(
                    "Signature Verification Failed: Invalid public key for package '{}': {}",
                    package_id, e
                );
                return false;
            }
        };

        // Reconstruct the signature from raw bytes
        let sig_array: [u8; 64] = match signature_bytes.try_into() {
            Ok(a) => a,
            Err(_) => {
                warn!(
                    "Signature Verification Failed: Could not parse signature bytes for package '{}'",
                    package_id
                );
                return false;
            }
        };
        let signature = Signature::from_bytes(&sig_array);

        // Perform real Ed25519 cryptographic verification
        match verifying_key.verify(payload, &signature) {
            Ok(()) => {
                info!(
                    "Ed25519 Signature Verified: Package '{}' payload integrity confirmed ({} bytes).",
                    package_id,
                    payload.len()
                );
                true
            }
            Err(e) => {
                warn!(
                    "Signature Verification REJECTED: Package '{}' signature mismatch: {}",
                    package_id, e
                );
                false
            }
        }
    }

    /// Generates a new Ed25519 signing keypair for package authors.
    /// Returns `(signing_key_bytes, verifying_key_bytes)`.
    pub fn generate_keypair() -> ([u8; 32], [u8; 32]) {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        (signing_key.to_bytes(), verifying_key.to_bytes())
    }

    /// Signs a payload with the given signing (private) key bytes and returns a 64-byte signature.
    pub fn sign_payload(payload: &[u8], signing_key_bytes: &[u8; 32]) -> [u8; 64] {
        let signing_key = SigningKey::from_bytes(signing_key_bytes);
        let signature = signing_key.sign(payload);
        signature.to_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_real_sign_and_verify() {
        let (sk_bytes, pk_bytes) = Ed25519Verifier::generate_keypair();
        let payload = b"weather-widget-v1.0.0-payload-content";

        let sig = Ed25519Verifier::sign_payload(payload, &sk_bytes);
        assert!(Ed25519Verifier::verify_package(
            "weather-widget",
            payload,
            &sig,
            &pk_bytes
        ));
    }

    #[test]
    fn test_ed25519_rejects_tampered_payload() {
        let (sk_bytes, pk_bytes) = Ed25519Verifier::generate_keypair();
        let payload = b"original-payload";
        let tampered_payload = b"tampered-payload";

        let sig = Ed25519Verifier::sign_payload(payload, &sk_bytes);
        // Signature created for original should fail on tampered content
        assert!(!Ed25519Verifier::verify_package(
            "tampered-widget",
            tampered_payload,
            &sig,
            &pk_bytes
        ));
    }

    #[test]
    fn test_ed25519_rejects_wrong_key() {
        let (sk_bytes, _pk_bytes) = Ed25519Verifier::generate_keypair();
        let (_other_sk, other_pk) = Ed25519Verifier::generate_keypair();
        let payload = b"widget-payload";

        let sig = Ed25519Verifier::sign_payload(payload, &sk_bytes);
        // Valid signature but wrong public key should fail
        assert!(!Ed25519Verifier::verify_package(
            "wrong-key-widget",
            payload,
            &sig,
            &other_pk
        ));
    }

    #[test]
    fn test_ed25519_rejects_empty_payload() {
        let (_sk_bytes, pk_bytes) = Ed25519Verifier::generate_keypair();
        assert!(!Ed25519Verifier::verify_package(
            "empty-widget",
            b"",
            &[0u8; 64],
            &pk_bytes
        ));
    }

    #[test]
    fn test_ed25519_rejects_invalid_signature_length() {
        let (_sk_bytes, pk_bytes) = Ed25519Verifier::generate_keypair();
        // Too short signature
        assert!(!Ed25519Verifier::verify_package(
            "bad-sig-widget",
            b"payload",
            b"too-short",
            &pk_bytes
        ));
    }

    #[test]
    fn test_ed25519_keypair_generation_produces_valid_keys() {
        let (sk, pk) = Ed25519Verifier::generate_keypair();
        // Keys should be 32 bytes each and non-zero
        assert_ne!(sk, [0u8; 32]);
        assert_ne!(pk, [0u8; 32]);
        // Round-trip test: sign then verify
        let msg = b"round-trip-test";
        let sig = Ed25519Verifier::sign_payload(msg, &sk);
        assert!(Ed25519Verifier::verify_package("roundtrip", msg, &sig, &pk));
    }
}
