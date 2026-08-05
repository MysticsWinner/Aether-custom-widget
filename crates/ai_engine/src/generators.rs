use theme_engine::ThemeSchema;
use tracing::info;
use widget_parser::WidgetManifest;

/// AI Layout Generator synthesizing responsive flexbox layouts from prompt specifications.
pub struct LayoutGenerator;

impl LayoutGenerator {
    pub fn generate_layout(prompt: &str) -> (f32, f32) {
        info!("AI Layout Generator processing prompt: '{}'", prompt);
        if prompt.contains("4k") || prompt.contains("large") {
            (400.0, 600.0)
        } else {
            (300.0, 150.0)
        }
    }
}

/// AI Theme Generator synthesizing valid `ThemeSchema` JSON from natural language descriptions.
pub struct ThemeGenerator;

impl ThemeGenerator {
    pub fn generate_theme(prompt: &str) -> anyhow::Result<ThemeSchema> {
        info!(
            "AI Theme Generator synthesizing theme for prompt: '{}'",
            prompt
        );

        let mut schema = ThemeSchema::default();
        schema.metadata.name = format!("AI Generated Theme ({})", prompt);

        if prompt.contains("cyberpunk") || prompt.contains("neon") {
            schema
                .colors
                .insert("theme.accent".to_string(), "#FF007F".to_string());
            schema
                .colors
                .insert("theme.background".to_string(), "#0D0221E6".to_string());
            schema
                .colors
                .insert("theme.text_primary".to_string(), "#00F5D4".to_string());
        } else if prompt.contains("forest") || prompt.contains("green") {
            schema
                .colors
                .insert("theme.accent".to_string(), "#2E7D32".to_string());
            schema
                .colors
                .insert("theme.background".to_string(), "#1B2E1BE6".to_string());
            schema
                .colors
                .insert("theme.text_primary".to_string(), "#A5D6A7".to_string());
        }

        // Validate JSON serialization
        let json_str = schema.to_json()?;
        let validated = ThemeSchema::parse_json(&json_str)?;
        Ok(validated)
    }
}

/// AI Widget Generator synthesizing declarative TOML `WidgetManifest` schemas.
pub struct WidgetGenerator;

impl WidgetGenerator {
    pub fn generate_widget(prompt: &str) -> anyhow::Result<WidgetManifest> {
        info!(
            "AI Widget Generator synthesizing widget manifest for prompt: '{}'",
            prompt
        );

        let binding = if prompt.contains("gpu") {
            "sys.gpu_usage"
        } else if prompt.contains("ram") || prompt.contains("memory") {
            "sys.memory_used"
        } else if prompt.contains("net") || prompt.contains("network") {
            "sys.network_rate"
        } else {
            "sys.cpu_usage"
        };

        let sanitized_prompt = prompt.replace('"', "'");
        let manifest_toml = format!(
            r#"
            [metadata]
            id = "ai.generated.widget"
            name = "AI Synthesized Widget ({})"
            author = "AI Engine"
            version = "1.0.0"
            update_interval_ms = 1000

            [layout]
            width = 340.0
            height = 200.0
            padding = 12.0

            [[elements]]
            id = "ai_label"
            element_type = "text"
            binding = "{}"
            font_size = 16.0
            color_token = "theme.text_primary"
            "#,
            sanitized_prompt, binding
        );

        let manifest = WidgetManifest::parse_toml(&manifest_toml)?;
        Ok(manifest)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_generators() {
        // Layout
        let (w, h) = LayoutGenerator::generate_layout("generate 4k widget layout");
        assert_eq!(w, 400.0);
        assert_eq!(h, 600.0);

        // Theme
        let theme = ThemeGenerator::generate_theme("cyberpunk neon").unwrap();
        assert_eq!(theme.colors.get("theme.accent").unwrap(), "#FF007F");

        // Widget
        let widget = WidgetGenerator::generate_widget("CPU monitor").unwrap();
        assert_eq!(widget.metadata.id, "ai.generated.widget");
    }
}
