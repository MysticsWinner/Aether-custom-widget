use crate::grant_store::GrantStore;
use crate::token::{CapabilityError, CapabilityToken, CapabilityType, GrantDecision};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn};
use uuid::Uuid;

/// Security Capability Broker issuing, tracking, and revoking runtime access tokens.
pub struct CapabilityBroker {
    grant_store: GrantStore,
    active_tokens: HashMap<String, CapabilityToken>, // token_id -> CapabilityToken
}

impl CapabilityBroker {
    pub fn new<P: AsRef<Path>>(store_path: P) -> Self {
        Self {
            grant_store: GrantStore::new(store_path),
            active_tokens: HashMap::new(),
        }
    }

    pub fn grant_store(&self) -> &GrantStore {
        &self.grant_store
    }

    pub fn grant_store_mut(&mut self) -> &mut GrantStore {
        &mut self.grant_store
    }

    /// Evaluates access and issues a CapabilityToken if authorized.
    pub fn request_token(
        &mut self,
        widget_id: &str,
        capability: &CapabilityType,
        now_ms: u64,
        ttl_ms: Option<u64>,
    ) -> Result<CapabilityToken, CapabilityError> {
        // 1. Hard block forbidden capabilities
        if capability.is_forbidden() {
            warn!(widget_id, cap = %capability.as_str(), "Forbidden capability request blocked by broker");
            return Err(CapabilityError::Forbidden(capability.as_str().to_string()));
        }

        // 2. Low-risk capabilities are auto-granted by default
        let is_low_risk = matches!(
            capability,
            CapabilityType::TelemetryRead
                | CapabilityType::FsRead
                | CapabilityType::FsWrite
                | CapabilityType::Notifications
                | CapabilityType::AiQuery
        );

        let decision = if is_low_risk {
            GrantDecision::Always
        } else {
            self.grant_store
                .get_decision(widget_id, capability)
                .unwrap_or(GrantDecision::AllowOnce)
        };

        if decision == GrantDecision::Never {
            return Err(CapabilityError::Denied(
                capability.as_str().to_string(),
                widget_id.to_string(),
            ));
        }

        // Issue token
        let token_id = Uuid::new_v4().to_string();
        let single_use = decision == GrantDecision::AllowOnce;
        let expires_at_ms = ttl_ms.map(|ttl| now_ms + ttl);

        let token = CapabilityToken {
            token_id: token_id.clone(),
            widget_id: widget_id.to_string(),
            capability: capability.clone(),
            granted_at_ms: now_ms,
            expires_at_ms,
            single_use,
            used: false,
        };

        self.active_tokens.insert(token_id.clone(), token.clone());
        info!(token_id = %token_id, widget_id = %widget_id, cap = %capability.as_str(), "Issued capability token");

        Ok(token)
    }

    /// Validates an active token and consumes single-use tokens.
    pub fn verify_token(
        &mut self,
        token_id: &str,
        capability: &CapabilityType,
        now_ms: u64,
    ) -> Result<(), CapabilityError> {
        let token = self
            .active_tokens
            .get_mut(token_id)
            .ok_or_else(|| CapabilityError::TokenRevoked(token_id.to_string()))?;

        if token.capability != *capability {
            return Err(CapabilityError::Denied(
                capability.as_str().to_string(),
                token.widget_id.clone(),
            ));
        }

        if !token.is_valid(now_ms) {
            return Err(CapabilityError::TokenExpired(token_id.to_string()));
        }

        if token.single_use {
            token.used = true;
        }

        Ok(())
    }

    /// Explicitly revokes an active capability token.
    pub fn revoke_token(&mut self, token_id: &str) -> bool {
        self.active_tokens.remove(token_id).is_some()
    }
}
