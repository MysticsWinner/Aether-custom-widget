use serde::{Deserialize, Serialize};

/// Output artifact of AI widget synthesis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SynthesizedWidget {
    pub id: String,
    pub name: String,
    pub manifest_toml: String,
    pub lua_script: String,
}

/// Natural language to widget manifest & Lua script AI synthesizer.
pub struct WidgetSynthesizer;

impl WidgetSynthesizer {
    pub fn synthesize(prompt: &str) -> SynthesizedWidget {
        let clean_prompt = prompt.trim().to_lowercase();
        let name = if clean_prompt.contains("cpu") {
            "AI Generated CPU Widget"
        } else if clean_prompt.contains("gpu") {
            "AI Generated GPU Widget"
        } else {
            "AI Generated Custom Widget"
        };

        let id = format!(
            "ai.generated.{}",
            name.to_lowercase().replace(' ', "_")
        );

        let manifest_toml = format!(
            r#"[widget]
id = "{id}"
name = "{name}"
version = "1.0.0"
author = "AI Engine"
description = "Synthesized from prompt: {prompt}"

[layout]
width = 300
height = 200
"#
        );

        let lua_script = format!(
            r#"-- Synthesized Widget: {name}
function on_update(ctx)
    local cpu = get_cpu_pct()
    local ram = get_memory_mb()
    -- Render synthesized UI
end
"#
        );

        SynthesizedWidget {
            id,
            name: name.to_string(),
            manifest_toml,
            lua_script,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_synthesizer_generates_manifest_and_lua() {
        let synthesized = WidgetSynthesizer::synthesize("Show CPU and RAM performance card");
        assert!(synthesized.id.contains("ai.generated"));
        assert!(synthesized.manifest_toml.contains("AI Generated CPU Widget"));
        assert!(synthesized.lua_script.contains("get_cpu_pct()"));
    }
}
