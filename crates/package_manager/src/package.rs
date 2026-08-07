use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Signed publisher metadata for marketplace verification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublisherMetadata {
    pub author: String,
    pub certificate: Option<String>,
    pub signature: Option<String>,
    pub reputation_score: f32,
    pub downloads: u64,
    pub verified: bool,
}

impl Default for PublisherMetadata {
    fn default() -> Self {
        Self {
            author: "Anonymous".to_string(),
            certificate: None,
            signature: None,
            reputation_score: 5.0,
            downloads: 0,
            verified: false,
        }
    }
}

/// Metadata describing an installed or marketplace widget package.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WidgetPackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub main_entrypoint: String,
    pub dependencies: HashMap<String, String>,
    pub requested_capabilities: Vec<String>,
    #[serde(default)]
    pub publisher: PublisherMetadata,
}

impl WidgetPackage {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        let author_str = author.into();
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            author: author_str.clone(),
            description: String::new(),
            main_entrypoint: "index.lua".to_string(),
            dependencies: HashMap::new(),
            requested_capabilities: Vec::new(),
            publisher: PublisherMetadata {
                author: author_str,
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_package_creation() {
        let pkg = WidgetPackage::new("weather-widget", "Weather Overlay Widget", "1.0.0", "Community");
        assert_eq!(pkg.id, "weather-widget");
        assert_eq!(pkg.version, "1.0.0");
    }
}
