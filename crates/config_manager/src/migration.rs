use anyhow::{anyhow, Result};
use serde_json::Value;
use tracing::info;

/// Trait implemented by schema migrations.
pub trait Migration: Send + Sync {
    /// Target version after this migration is applied.
    fn target_version(&self) -> u32;
    /// Applies migration mutations to the JSON payload.
    fn apply(&self, value: &mut Value) -> Result<()>;
}

/// Upgrades JSON configurations across sequential schema versions.
#[derive(Default)]
pub struct MigrationEngine {
    migrations: Vec<Box<dyn Migration>>,
}

impl MigrationEngine {
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    pub fn register<M: Migration + 'static>(&mut self, migration: M) {
        self.migrations.push(Box::new(migration));
        self.migrations.sort_by_key(|m| m.target_version());
    }

    /// Upgrades `value` from its current `schema_version` to target `max_version`.
    pub fn migrate(&self, value: &mut Value, target_version: u32) -> Result<u32> {
        let current_version = value
            .get("schema_version")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(1);

        if current_version >= target_version {
            return Ok(current_version);
        }

        let mut current = current_version;
        for m in &self.migrations {
            if m.target_version() == current + 1 && m.target_version() <= target_version {
                info!(from = current, to = m.target_version(), "Applying schema migration");
                m.apply(value)?;
                if let Some(obj) = value.as_object_mut() {
                    obj.insert("schema_version".to_string(), Value::from(m.target_version()));
                }
                current = m.target_version();
            }
        }

        if current != target_version {
            return Err(anyhow!(
                "Incomplete migration chain: reached version {} but expected {}",
                current,
                target_version
            ));
        }

        Ok(current)
    }
}
