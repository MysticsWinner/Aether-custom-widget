use crate::token::{CapabilityError, CapabilityToken, CapabilityType};
use anyhow::Result;
use tracing::{info, warn};

/// Gateway firewall validating active capability tokens for widget requests.
pub struct WidgetFirewall;

impl WidgetFirewall {
    /// Validates token against target capability and timestamp.
    pub fn validate_access(
        token: &CapabilityToken,
        requested_capability: &CapabilityType,
        now_ms: u64,
    ) -> Result<(), CapabilityError> {
        if requested_capability.is_forbidden() {
            return Err(CapabilityError::Forbidden(
                requested_capability.as_str().to_string(),
            ));
        }

        if &token.capability != requested_capability {
            warn!(
                token_cap = %token.capability.as_str(),
                requested = %requested_capability.as_str(),
                "Capability mismatch in token validation"
            );
            return Err(CapabilityError::Denied(
                requested_capability.as_str().to_string(),
                token.widget_id.clone(),
            ));
        }

        if !token.is_valid(now_ms) {
            warn!(token_id = %token.token_id, "Attempted to use invalid or expired token");
            return Err(CapabilityError::TokenExpired(token.token_id.clone()));
        }

        info!(
            widget_id = %token.widget_id,
            cap = %requested_capability.as_str(),
            "Firewall access granted"
        );
        Ok(())
    }
}
