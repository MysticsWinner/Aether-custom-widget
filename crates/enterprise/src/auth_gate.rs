use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn};

/// Windows Hello authentication result status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthResult {
    Success,
    UserCancelled,
    BiometricMismatch,
    DeviceNotPresent,
    NotConfiguredForUser,
    DisabledByPolicy,
    DeviceBusy,
    RetriesExhausted,
    SsoTokenExpired,
    DomainMismatch,
}

/// Windows Web Account Manager (WAM) & Active Directory Kerberos SSO ticket descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WamSsoToken {
    pub account_upn: String,
    pub tenant_id: String,
    pub kerberos_ticket_sha256: String,
    pub expires_at_ms: u64,
}

/// Global test mode switch for non-interactive automated test environments.
/// When enabled via `AuthGate::set_test_mode(true)`, calls skip live interactive UI popups
/// and return `AuthResult::Success` or a configured test state. In production (`test_mode == false`),
/// the real Windows Hello WinRT `UserConsentVerifier` API is invoked.
static TEST_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Windows Hello biometric & WAM / Kerberos SSO authentication gate for sensitive actions.
pub struct AuthGate;

impl AuthGate {
    /// Configures whether test-mode bypass is enabled (only to be called by automated test runners).
    pub fn set_test_mode(enabled: bool) {
        TEST_MODE_ACTIVE.store(enabled, Ordering::SeqCst);
    }

    /// Returns true if test-mode bypass is active.
    pub fn is_test_mode() -> bool {
        TEST_MODE_ACTIVE.load(Ordering::SeqCst)
    }

    /// Prompts Windows Hello biometric / PIN verification for a sensitive operation using the
    /// native Windows Runtime `Windows.Security.Credentials.UI.UserConsentVerifier` API.
    pub fn prompt_windows_hello(action_description: &str) -> Result<AuthResult> {
        info!(action = %action_description, "Prompting Windows Hello authentication gate");

        if Self::is_test_mode() {
            info!("AuthGate running in test mode — returning verified test result");
            return Ok(AuthResult::Success);
        }

        #[cfg(windows)]
        {
            use windows::core::HSTRING;
            use windows::Security::Credentials::UI::{UserConsentVerificationResult, UserConsentVerifier};

            let h_action = HSTRING::from(action_description);
            let async_op = match UserConsentVerifier::RequestVerificationAsync(&h_action) {
                Ok(op) => op,
                Err(e) => {
                    warn!("Failed to initiate UserConsentVerifier: {:?}", e);
                    return Ok(AuthResult::DeviceNotPresent);
                }
            };

            // Synchronously wait for user consent completion on Windows
            match async_op.get() {
                Ok(UserConsentVerificationResult::Verified) => Ok(AuthResult::Success),
                Ok(UserConsentVerificationResult::DeviceNotPresent) => Ok(AuthResult::DeviceNotPresent),
                Ok(UserConsentVerificationResult::NotConfiguredForUser) => Ok(AuthResult::NotConfiguredForUser),
                Ok(UserConsentVerificationResult::DisabledByPolicy) => Ok(AuthResult::DisabledByPolicy),
                Ok(UserConsentVerificationResult::DeviceBusy) => Ok(AuthResult::DeviceBusy),
                Ok(UserConsentVerificationResult::RetriesExhausted) => Ok(AuthResult::RetriesExhausted),
                Ok(UserConsentVerificationResult::Canceled) => Ok(AuthResult::UserCancelled),
                Ok(_) => Ok(AuthResult::BiometricMismatch),
                Err(err) => {
                    warn!("Windows Hello consent verification error: {:?}", err);
                    Ok(AuthResult::BiometricMismatch)
                }
            }
        }

        #[cfg(not(windows))]
        {
            Ok(AuthResult::DeviceNotPresent)
        }
    }

    /// Validates an Enterprise Windows Web Account Manager (WAM) / Kerberos SSO token against
    /// expiration bounds and required corporate Active Directory UPN suffix.
    pub fn validate_wam_sso_token(
        token: &WamSsoToken,
        current_time_ms: u64,
        required_domain_suffix: &str,
    ) -> Result<AuthResult> {
        if current_time_ms >= token.expires_at_ms {
            return Ok(AuthResult::SsoTokenExpired);
        }
        let expected_suffix = format!("@{}", required_domain_suffix.trim_start_matches('@'));
        if !token
            .account_upn
            .to_lowercase()
            .ends_with(&expected_suffix.to_lowercase())
            || token.kerberos_ticket_sha256.len() < 16
        {
            return Ok(AuthResult::DomainMismatch);
        }
        Ok(AuthResult::Success)
    }
}
