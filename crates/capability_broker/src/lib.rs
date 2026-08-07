pub mod broker;
pub mod firewall;
pub mod grant_store;
pub mod token;

pub use broker::CapabilityBroker;
pub use firewall::WidgetFirewall;
pub use grant_store::GrantStore;
pub use token::{CapabilityError, CapabilityToken, CapabilityType, GrantDecision};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_capability_broker_grants_valid_token() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("grants.json");
        let mut broker = CapabilityBroker::new(&store_path);

        let token = broker
            .request_token("clock_widget", &CapabilityType::TelemetryRead, 1000, Some(5000))
            .unwrap();

        assert_eq!(token.widget_id, "clock_widget");
        assert!(token.is_valid(1000));
        assert!(broker
            .verify_token(&token.token_id, &CapabilityType::TelemetryRead, 2000)
            .is_ok());
    }

    #[test]
    fn test_capability_broker_rejects_forbidden_capability() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("grants.json");
        let mut broker = CapabilityBroker::new(&store_path);

        let res = broker.request_token("evil_widget", &CapabilityType::ShellExecute, 1000, None);
        assert!(matches!(res, Err(CapabilityError::Forbidden(_))));

        let res_reg = broker.request_token("evil_widget", &CapabilityType::RegistryWrite, 1000, None);
        assert!(matches!(res_reg, Err(CapabilityError::Forbidden(_))));
    }

    #[test]
    fn test_capability_token_expiration() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("grants.json");
        let mut broker = CapabilityBroker::new(&store_path);

        let token = broker
            .request_token("weather_widget", &CapabilityType::NetworkHttp, 1000, Some(500))
            .unwrap();

        assert!(broker
            .verify_token(&token.token_id, &CapabilityType::NetworkHttp, 1200)
            .is_ok());
        assert!(matches!(
            broker.verify_token(&token.token_id, &CapabilityType::NetworkHttp, 1600),
            Err(CapabilityError::TokenExpired(_))
        ));
    }

    #[test]
    fn test_capability_token_single_use() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("grants.json");
        let mut broker = CapabilityBroker::new(&store_path);

        // Record AllowOnce decision
        broker
            .grant_store_mut()
            .record_decision("single_widget", &CapabilityType::ClipboardRead, GrantDecision::AllowOnce)
            .unwrap();

        let token = broker
            .request_token("single_widget", &CapabilityType::ClipboardRead, 1000, None)
            .unwrap();

        // First verification succeeds
        assert!(broker
            .verify_token(&token.token_id, &CapabilityType::ClipboardRead, 1050)
            .is_ok());

        // Second verification fails due to single_use constraint
        assert!(matches!(
            broker.verify_token(&token.token_id, &CapabilityType::ClipboardRead, 1060),
            Err(CapabilityError::TokenExpired(_))
        ));
    }

    #[test]
    fn test_capability_token_revocation() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("grants.json");
        let mut broker = CapabilityBroker::new(&store_path);

        let token = broker
            .request_token("my_widget", &CapabilityType::TelemetryRead, 1000, None)
            .unwrap();

        assert!(broker.revoke_token(&token.token_id));
        assert!(matches!(
            broker.verify_token(&token.token_id, &CapabilityType::TelemetryRead, 1100),
            Err(CapabilityError::TokenRevoked(_))
        ));
    }

    #[test]
    fn test_grant_store_persistence() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("grants.json");

        {
            let mut store = GrantStore::new(&store_path);
            store
                .record_decision("persist_widget", &CapabilityType::NetworkHttp, GrantDecision::Always)
                .unwrap();
        }

        let store2 = GrantStore::new(&store_path);
        assert_eq!(
            store2.get_decision("persist_widget", &CapabilityType::NetworkHttp),
            Some(GrantDecision::Always)
        );
    }

    #[test]
    fn test_widget_firewall_validation() {
        let dir = tempdir().unwrap();
        let store_path = dir.path().join("grants.json");
        let mut broker = CapabilityBroker::new(&store_path);

        let token = broker
            .request_token("fw_widget", &CapabilityType::NetworkHttp, 1000, Some(2000))
            .unwrap();

        assert!(WidgetFirewall::validate_access(&token, &CapabilityType::NetworkHttp, 1500).is_ok());
        assert!(matches!(
            WidgetFirewall::validate_access(&token, &CapabilityType::ShellExecute, 1500),
            Err(CapabilityError::Forbidden(_))
        ));
    }
}
