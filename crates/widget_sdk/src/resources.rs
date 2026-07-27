use crate::rendering::Color;
use std::collections::HashMap;

/// 6. Resources API Pillar Interface
pub trait ResourceManager: Send + Sync {
    /// Loads asset binary data by relative path or identifier.
    fn load_asset(&self, path: &str) -> anyhow::Result<Vec<u8>>;

    /// Resolves system theme token (e.g. "theme.accent") to color value.
    fn resolve_color_token(&self, token_id: &str) -> Color;

    /// Checks if dynamic font family is cached.
    fn is_font_cached(&self, font_family: &str) -> bool;
}

/// Default In-Memory Resource Manager implementation.
#[derive(Debug, Default)]
pub struct InMemoryResourceManager {
    tokens: HashMap<String, Color>,
    assets: HashMap<String, Vec<u8>>,
}

impl InMemoryResourceManager {
    pub fn new() -> Self {
        let mut tokens = HashMap::new();
        tokens.insert("theme.accent".to_string(), Color::rgba(0.0, 0.47, 0.84, 1.0));
        tokens.insert("theme.text_primary".to_string(), Color::rgba(1.0, 1.0, 1.0, 1.0));
        tokens.insert("theme.background".to_string(), Color::rgba(0.1, 0.1, 0.1, 0.8));

        Self {
            tokens,
            assets: HashMap::new(),
        }
    }

    pub fn register_asset(&mut self, path: &str, data: Vec<u8>) {
        self.assets.insert(path.to_string(), data);
    }
}

impl ResourceManager for InMemoryResourceManager {
    fn load_asset(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        self.assets
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Asset path not found: {}", path))
    }

    fn resolve_color_token(&self, token_id: &str) -> Color {
        self.tokens
            .get(token_id)
            .cloned()
            .unwrap_or_else(|| Color::rgba(1.0, 1.0, 1.0, 1.0))
    }

    fn is_font_cached(&self, font_family: &str) -> bool {
        font_family == "Segoe UI" || font_family == "Roboto"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_manager() {
        let mut mgr = InMemoryResourceManager::new();
        mgr.register_asset("icons/cpu.png", vec![0x89, 0x50, 0x4E, 0x47]);

        let accent = mgr.resolve_color_token("theme.accent");
        assert_eq!(accent, Color::rgba(0.0, 0.47, 0.84, 1.0));

        let asset = mgr.load_asset("icons/cpu.png").unwrap();
        assert_eq!(asset, vec![0x89, 0x50, 0x4E, 0x47]);

        assert!(mgr.is_font_cached("Segoe UI"));
    }
}
