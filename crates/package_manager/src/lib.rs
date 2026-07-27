//! Next-Gen Windows Desktop Customization Platform - Marketplace Package Manager Crate
//!
//! Provides an npm-like CLI package installation experience (install weather-widget, install spotify-widget, install taskbar-plus)
//! powered by Ed25519 cryptographic signature verification and local package management.

pub mod benchmark;
pub mod installer;
pub mod package;
pub mod security;

pub use benchmark::PackageManagerBenchmark;
pub use installer::PackageManager;
pub use package::WidgetPackage;
pub use security::Ed25519Verifier;
