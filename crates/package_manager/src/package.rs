use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

impl WidgetPackage {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            author: author.into(),
            description: String::new(),
            main_entrypoint: "index.lua".to_string(),
            dependencies: HashMap::new(),
            requested_capabilities: Vec::new(),
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
