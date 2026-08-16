//! Declarative Widget Engine
//!
//! Enables simple widgets to be constructed entirely through TOML/JSON layout specs,
//! design tokens, metric bindings, and visual assets without executable Lua/WASM code.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeclarativeBinding {
    pub metric_key: String,
    pub target_element_id: String,
    pub transform: Option<String>, // e.g. "percentage", "bytes_human"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeclarativeWidgetSpec {
    pub is_declarative_only: bool,
    pub bindings: Vec<DeclarativeBinding>,
    pub fallback_text: String,
}

impl Default for DeclarativeWidgetSpec {
    fn default() -> Self {
        Self {
            is_declarative_only: true,
            bindings: Vec::new(),
            fallback_text: "N/A".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declarative_widget_spec_default() {
        let spec = DeclarativeWidgetSpec::default();
        assert!(spec.is_declarative_only);
        assert_eq!(spec.fallback_text, "N/A");
    }
}
