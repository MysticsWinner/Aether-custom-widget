pub mod audit_logger;
pub mod auth_gate;
pub mod policy;

pub use audit_logger::{AuditBlock, AuditLogger};
pub use auth_gate::{AuthGate, AuthResult, WamSsoToken};
pub use policy::{EnterprisePolicy, PolicyEngine};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_policy_engine_enforces_blocked_widgets() {
        let dir = tempdir().unwrap();
        let policy_path = dir.path().join("policy.json");
        let mut engine = PolicyEngine::new(&policy_path);

        assert!(engine.is_widget_allowed("clock_w"));

        let mut policy = EnterprisePolicy::default();
        policy.blocked_widgets.insert("clock_w".to_string());
        engine.update_policy(policy).unwrap();

        assert!(!engine.is_widget_allowed("clock_w"));
    }

    #[test]
    fn test_audit_logger_sha256_hash_chaining() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("audit.log");
        let mut logger = AuditLogger::new(&log_path);

        logger.append_event("WidgetLoaded: clock_w", 1000).unwrap();
        logger.append_event("CapabilityGranted: network.http", 2000).unwrap();

        assert_eq!(logger.chain().len(), 2);
        assert!(logger.verify_chain());
    }

    #[test]
    fn test_audit_logger_detects_tampered_log() {
        let dir = tempdir().unwrap();
        let log_path = dir.path().join("audit.log");

        {
            let mut logger = AuditLogger::new(&log_path);
            logger.append_event("Event 1", 1000).unwrap();
            logger.append_event("Event 2", 2000).unwrap();
        }

        // Tamper log file
        let mut chain: Vec<AuditBlock> =
            serde_json::from_str(&std::fs::read_to_string(&log_path).unwrap()).unwrap();
        chain[1].event = "TAMPERED_EVENT".to_string();
        std::fs::write(&log_path, serde_json::to_string_pretty(&chain).unwrap()).unwrap();

        let tampered_logger = AuditLogger::new(&log_path);
        assert!(!tampered_logger.verify_chain());
    }

    #[test]
    fn test_auth_gate_biometric_validation() {
        let res = AuthGate::prompt_windows_hello("Grant capability network.http").unwrap();
        assert_eq!(res, AuthResult::Success);
    }

    #[test]
    fn test_ad_gpo_registry_sync_and_wam_kerberos_sso() {
        let dir = tempdir().unwrap();
        let policy_path = dir.path().join("ad_policy.json");
        let mut engine = PolicyEngine::new(&policy_path);
        assert_eq!(PolicyEngine::REGISTRY_GPO_PATH, r"HKLM\SOFTWARE\Policies\Aether");

        let mut gpo = std::collections::HashMap::new();
        gpo.insert("AllowMarketplace".to_string(), "0".to_string());
        gpo.insert("ForceSafeMode".to_string(), "1".to_string());
        gpo.insert(
            "BlockedWidgets".to_string(),
            "crypto_stocks_widget;untrusted_widget".to_string(),
        );
        gpo.insert(
            "AllowedCapabilities".to_string(),
            "telemetry.read;fs.read".to_string(),
        );

        engine.sync_from_ad_gpo_registry_values(&gpo).unwrap();
        assert!(!engine.policy().allow_marketplace);
        assert!(engine.policy().force_safe_mode);
        assert!(!engine.is_widget_allowed("crypto_stocks_widget"));
        assert!(engine.is_capability_allowed("telemetry.read"));
        assert!(!engine.is_capability_allowed("network.http"));

        let valid_token = WamSsoToken {
            account_upn: "alice@corp.aether.internal".to_string(),
            tenant_id: "tenant-01".to_string(),
            kerberos_ticket_sha256: "e3b0c44298fc1c149afbf4c8996fb924".to_string(),
            expires_at_ms: 2_000_000,
        };
        assert_eq!(
            AuthGate::validate_wam_sso_token(&valid_token, 1_500_000, "corp.aether.internal")
                .unwrap(),
            AuthResult::Success
        );
        assert_eq!(
            AuthGate::validate_wam_sso_token(&valid_token, 2_500_000, "corp.aether.internal")
                .unwrap(),
            AuthResult::SsoTokenExpired
        );
        assert_eq!(
            AuthGate::validate_wam_sso_token(&valid_token, 1_500_000, "other.domain.com")
                .unwrap(),
            AuthResult::DomainMismatch
        );
    }
}
