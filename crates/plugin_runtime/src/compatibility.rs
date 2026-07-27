use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{info, warn};

/// Semantic Versioning structure (MAJOR.MINOR.PATCH).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ApiVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
}

impl fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Enforces API Compatibility and Semantic Versioning contracts between Host and Plugins.
pub struct CompatibilityChecker;

impl CompatibilityChecker {
    /// Host SDK API version supported by current runtime engine.
    pub const HOST_API_VERSION: ApiVersion = ApiVersion {
        major: 1,
        minor: 0,
        patch: 0,
    };

    /// Checks if a plugin's required API version is compatible with current Host API version.
    /// Rules: Major versions must match exactly. Host minor version must be >= required minor version.
    pub fn is_compatible(required_version: ApiVersion) -> bool {
        let host = Self::HOST_API_VERSION;

        if required_version.major != host.major {
            warn!(
                "API Version Incompatible: Host is v{}, but plugin requires v{}",
                host, required_version
            );
            return false;
        }

        if required_version.minor > host.minor {
            warn!(
                "API Version Incompatible: Plugin requires newer minor features (v{}) than Host supports (v{})",
                required_version, host
            );
            return false;
        }

        info!(
            "API Compatibility Verified: Plugin version v{} is compatible with Host v{}",
            required_version, host
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_compatibility() {
        assert!(CompatibilityChecker::is_compatible(ApiVersion::new(1, 0, 0)));
        assert!(CompatibilityChecker::is_compatible(ApiVersion::new(1, 0, 5)));

        // Incompatible major version
        assert!(!CompatibilityChecker::is_compatible(ApiVersion::new(2, 0, 0)));
        // Incompatible future minor version
        assert!(!CompatibilityChecker::is_compatible(ApiVersion::new(1, 2, 0)));
    }
}
