pub mod heartbeat;
pub mod supervisor;

pub use heartbeat::{HeartbeatPayload, WatchdogHeartbeat};
pub use supervisor::{WatchdogStatus, WatchdogSupervisor};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watchdog_heartbeat_keeps_engine_alive_on_ping() {
        let mut supervisor = WatchdogSupervisor::new("aether_engine.exe", 5000);
        supervisor.record_heartbeat(1234, 1000);

        // Health check within 5s timeout -> healthy
        assert!(supervisor.check_health(3000).unwrap());
        assert_eq!(supervisor.status().restart_count, 0);
    }

    #[test]
    fn test_watchdog_heartbeat_restarts_engine_on_timeout() {
        let mut supervisor = WatchdogSupervisor::new("aether_engine.exe", 5000);
        supervisor.record_heartbeat(1234, 1000);

        // Health check beyond 5s timeout (at 7000ms) -> triggers restart
        let healthy = supervisor.check_health(7000).unwrap();
        assert!(!healthy);
        assert_eq!(supervisor.status().restart_count, 1);
    }
}
