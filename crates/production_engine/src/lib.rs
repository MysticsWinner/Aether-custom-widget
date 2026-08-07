//! Next-Gen Windows Desktop Customization Platform - Production Readiness Engine Crate
//!
//! Provides production security auditing, 100-widget stress testing, MSIX auto-updater,
//! zero-PII crash analytics, documentation portal generation, and release candidate verification.

pub mod auto_updater;
pub mod benchmark;
pub mod chaos_harness;
pub mod crash_analytics;
pub mod docs_portal;
pub mod security_audit;
pub mod stress_test;

pub use auto_updater::AutoUpdater;
pub use benchmark::MasterReleaseSuite;
pub use chaos_harness::{ChaosHarness, ChaosScenario};
pub use crash_analytics::CrashAnalytics;
pub use docs_portal::DocumentationPortal;
pub use security_audit::SecurityAuditor;
pub use stress_test::StressTestingHarness;
