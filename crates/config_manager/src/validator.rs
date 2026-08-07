use anyhow::{anyhow, Result};
use serde_json::Value;

/// Validates JSON configuration schema and structure before atomic commit.
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validates that input value is a valid JSON Object.
    pub fn validate_json_object(val: &Value) -> Result<()> {
        if !val.is_object() {
            return Err(anyhow!("Config payload must be a JSON Object"));
        }
        Ok(())
    }

    /// Validates schema version presence and bounds.
    pub fn validate_schema_version(val: &Value, min_version: u32) -> Result<u32> {
        Self::validate_json_object(val)?;
        let ver = val
            .get("schema_version")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .ok_or_else(|| anyhow!("Missing or invalid 'schema_version' field in JSON payload"))?;

        if ver < min_version {
            return Err(anyhow!(
                "Config schema_version {} is lower than minimum required {}",
                ver,
                min_version
            ));
        }

        Ok(ver)
    }
}
