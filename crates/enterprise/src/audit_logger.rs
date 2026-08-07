use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Cryptographic tamper-evident audit record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditBlock {
    pub index: u64,
    pub timestamp_ms: u64,
    pub event: String,
    pub previous_hash: String,
    pub hash: String,
}

impl AuditBlock {
    pub fn compute_hash(index: u64, timestamp_ms: u64, event: &str, previous_hash: &str) -> String {
        let input = format!("{}:{}:{}:{}", index, timestamp_ms, event, previous_hash);
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Append-only, tamper-evident audit trail logger with SHA-256 block hash chaining.
#[derive(Debug, Clone)]
pub struct AuditLogger {
    log_path: PathBuf,
    chain: Vec<AuditBlock>,
}

impl AuditLogger {
    pub fn new<P: AsRef<Path>>(log_path: P) -> Self {
        let log_path = log_path.as_ref().to_path_buf();
        let chain = Self::load_from_disk(&log_path).unwrap_or_default();
        Self { log_path, chain }
    }

    fn load_from_disk(path: &Path) -> Result<Vec<AuditBlock>> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(path)?;
        let chain: Vec<AuditBlock> = serde_json::from_str(&content)?;
        Ok(chain)
    }

    pub fn save_to_disk(&self) -> Result<()> {
        if let Some(parent) = self.log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.chain)?;
        std::fs::write(&self.log_path, content)?;
        Ok(())
    }

    pub fn append_event(&mut self, event: &str, now_ms: u64) -> Result<AuditBlock> {
        let index = self.chain.len() as u64;
        let previous_hash = match self.chain.last() {
            Some(last) => last.hash.clone(),
            None => "GENESIS_HASH_00000000000000000000000000000000000000000000000000000000".to_string(),
        };

        let hash = AuditBlock::compute_hash(index, now_ms, event, &previous_hash);

        let block = AuditBlock {
            index,
            timestamp_ms: now_ms,
            event: event.to_string(),
            previous_hash,
            hash,
        };

        info!(index, event, "Appended audit log block");
        self.chain.push(block.clone());
        self.save_to_disk()?;
        Ok(block)
    }

    pub fn verify_chain(&self) -> bool {
        let mut expected_prev_hash = "GENESIS_HASH_00000000000000000000000000000000000000000000000000000000".to_string();

        for (i, block) in self.chain.iter().enumerate() {
            if block.index != i as u64 {
                warn!(i, "Audit chain sequence index mismatch");
                return false;
            }

            if block.previous_hash != expected_prev_hash {
                warn!(i, "Audit chain previous hash mismatch! Log file tampered.");
                return false;
            }

            let computed = AuditBlock::compute_hash(block.index, block.timestamp_ms, &block.event, &block.previous_hash);
            if computed != block.hash {
                warn!(i, "Audit block hash mismatch! Block content altered.");
                return false;
            }

            expected_prev_hash = block.hash.clone();
        }

        true
    }

    pub fn chain(&self) -> &[AuditBlock] {
        &self.chain
    }
}
