use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Strongly-typed settings value variant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SettingValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

/// 3. Settings API Pillar Interface
pub trait SettingsStore: Send + Sync {
    fn get(&self, key: &str) -> Option<SettingValue>;
    fn set(&mut self, key: &str, value: SettingValue) -> anyhow::Result<()>;
    fn contains(&self, key: &str) -> bool;
}

/// Memory-backed reactive settings store implementation.
#[derive(Debug, Default)]
pub struct InMemorySettingsStore {
    store: HashMap<String, SettingValue>,
}

impl InMemorySettingsStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SettingsStore for InMemorySettingsStore {
    fn get(&self, key: &str) -> Option<SettingValue> {
        self.store.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: SettingValue) -> anyhow::Result<()> {
        self.store.insert(key.to_string(), value);
        Ok(())
    }

    fn contains(&self, key: &str) -> bool {
        self.store.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_store() {
        let mut settings = InMemorySettingsStore::new();
        settings.set("theme.mode", SettingValue::String("dark".to_string())).unwrap();
        settings.set("widget.opacity", SettingValue::Float(0.95)).unwrap();

        assert_eq!(
            settings.get("theme.mode"),
            Some(SettingValue::String("dark".to_string()))
        );
        assert_eq!(
            settings.get("widget.opacity"),
            Some(SettingValue::Float(0.95))
        );
    }
}
