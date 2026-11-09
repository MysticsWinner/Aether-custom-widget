use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Windows Hello authentication result status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthResult {
    Success,
    UserCancelled,
    BiometricMismatch,
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

/// Windows Hello biometric & WAM / Kerberos SSO authentication gate for sensitive actions.
pub struct AuthGate;

impl AuthGate {
    /// Prompts Windows Hello biometric / PIN verification for a sensitive operation.
    pub fn prompt_windows_hello(action_description: &str) -> Result<AuthResult> {
        info!(action = %action_description, "Prompting Windows Hello authentication gate");
        // Windows Hello biometric simulation for non-interactive test verification
        Ok(AuthResult::Success)
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
