use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Granular capability categories requested by widgets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CapabilityType {
    TelemetryRead,
    FsRead,
    FsWrite,
    NetworkHttp,
    NetworkWebsocket,
    ClipboardRead,
    ClipboardWrite,
    Notifications,
    AiQuery,
    ShellExecute,   // Forbidden
    RegistryRead,
    RegistryWrite,  // Forbidden
    Custom(String),
}

impl CapabilityType {
    pub fn parse(s: &str) -> Self {
        match s {
            "telemetry.read" => Self::TelemetryRead,
            "fs.read" => Self::FsRead,
            "fs.write" => Self::FsWrite,
            "network.http" => Self::NetworkHttp,
            "network.websocket" => Self::NetworkWebsocket,
            "clipboard.read" => Self::ClipboardRead,
            "clipboard.write" => Self::ClipboardWrite,
            "notifications" => Self::Notifications,
            "ai.query" => Self::AiQuery,
            "shell.execute" => Self::ShellExecute,
            "registry.read" => Self::RegistryRead,
            "registry.write" => Self::RegistryWrite,
            other => Self::Custom(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::TelemetryRead => "telemetry.read",
            Self::FsRead => "fs.read",
            Self::FsWrite => "fs.write",
            Self::NetworkHttp => "network.http",
            Self::NetworkWebsocket => "network.websocket",
            Self::ClipboardRead => "clipboard.read",
            Self::ClipboardWrite => "clipboard.write",
            Self::Notifications => "notifications",
            Self::AiQuery => "ai.query",
            Self::ShellExecute => "shell.execute",
            Self::RegistryRead => "registry.read",
            Self::RegistryWrite => "registry.write",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn is_forbidden(&self) -> bool {
        matches!(self, Self::ShellExecute | Self::RegistryWrite)
    }
}

/// User approval decision for capability prompt.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GrantDecision {
    AllowOnce,
    Always,
    Never,
}

/// Token issued by CapabilityBroker granting temporary or persistent access.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityToken {
    pub token_id: String,
    pub widget_id: String,
    pub capability: CapabilityType,
    pub granted_at_ms: u64,
    pub expires_at_ms: Option<u64>,
    pub single_use: bool,
    pub used: bool,
}

impl CapabilityToken {
    pub fn is_valid(&self, now_ms: u64) -> bool {
        if self.single_use && self.used {
            return false;
        }
        if let Some(exp) = self.expires_at_ms {
            if now_ms >= exp {
                return false;
            }
        }
        true
    }
}

/// Security broker errors.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CapabilityError {
    #[error("Capability '{0}' is strictly forbidden by engine security policy")]
    Forbidden(String),

    #[error("Access to capability '{0}' was denied for widget '{1}'")]
    Denied(String, String),

    #[error("Capability token '{0}' has expired")]
    TokenExpired(String),

    #[error("Capability token '{0}' has been revoked or already used")]
    TokenRevoked(String),
}
