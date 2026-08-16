use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod declarative;

pub use declarative::{DeclarativeBinding, DeclarativeWidgetSpec};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Failed to parse widget TOML manifest: {0}")]
    TomlError(#[from] toml::de::Error),
    #[error("Invalid manifest schema: {0}")]
    InvalidSchema(String),
}

/// Declarative Widget Schema Manifest definition (TOML)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetManifest {
    pub metadata: WidgetMetadata,
    pub layout: LayoutSpec,
    pub elements: Vec<WidgetElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetMetadata {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub update_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSpec {
    pub width: f32,
    pub height: f32,
    pub flex_direction: Option<String>,
    pub padding: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetElement {
    pub id: String,
    pub element_type: String, // "text", "progress_bar", "graph", "image"
    pub binding: Option<String>, // e.g. "sys.cpu_usage"
    pub font_size: Option<f32>,
    pub color_token: Option<String>, // e.g. "theme.accent"
}

impl WidgetManifest {
    pub fn parse_toml(content: &str) -> Result<Self, ParserError> {
        let manifest: WidgetManifest = toml::from_str(content)?;
        if manifest.metadata.id.is_empty() {
            return Err(ParserError::InvalidSchema(
                "Widget ID cannot be empty".into(),
            ));
        }
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_manifest_parsing() {
        let toml_str = r#"
            [metadata]
            id = "widget.sys.cpu"
            name = "CPU Usage Widget"
            author = "SystemTeam"
            version = "1.0.0"
            update_interval_ms = 1000

            [layout]
            width = 300.0
            height = 150.0
            padding = 10.0

            [[elements]]
            id = "cpu_label"
            element_type = "text"
            binding = "sys.cpu_usage"
            font_size = 14.0
            color_token = "theme.text_primary"
        "#;

        let manifest = WidgetManifest::parse_toml(toml_str).unwrap();
        assert_eq!(manifest.metadata.id, "widget.sys.cpu");
        assert_eq!(manifest.elements.len(), 1);
        assert_eq!(
            manifest.elements[0].binding.as_deref(),
            Some("sys.cpu_usage")
        );
    }

    #[test]
    fn test_empty_id_manifest_rejection() {
        let toml_str = r#"
            [metadata]
            id = ""
            name = "Invalid Widget"
            author = "Unknown"
            version = "1.0.0"
            update_interval_ms = 1000

            [layout]
            width = 100.0
            height = 100.0
            flex_direction = "column"
            padding = 10.0

            [[elements]]
            id = "elem1"
            element_type = "text"
        "#;

        let err = WidgetManifest::parse_toml(toml_str).unwrap_err();
        assert!(matches!(err, ParserError::InvalidSchema(_)));
    }
}
