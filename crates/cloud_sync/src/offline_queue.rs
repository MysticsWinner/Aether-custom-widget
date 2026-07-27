use crate::entities::SyncEntity;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tracing::info;

/// Operations queued during Offline Mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncOperation {
    Upsert { entity_id: String, entity: SyncEntity },
    Delete { entity_id: String },
}

/// Offline Transaction Log Queue managing offline state persistence.
#[derive(Debug, Default)]
pub struct OfflineSyncQueue {
    queue: VecDeque<SyncOperation>,
    is_online: bool,
}

impl OfflineSyncQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            is_online: true,
        }
    }

    pub fn set_online_status(&mut self, online: bool) {
        self.is_online = online;
        if online {
            info!("Network connection restored. Switching to ONLINE mode.");
        } else {
            info!("Network connection lost. Switching to OFFLINE MODE (Buffering changes locally).");
        }
    }

    pub fn is_online(&self) -> bool {
        self.is_online
    }

    /// Enqueues a sync operation when in Offline Mode or online buffer pass.
    pub fn enqueue(&mut self, op: SyncOperation) {
        self.queue.push_back(op);
    }

    /// Flushes all pending offline operations upon network reconnect.
    pub fn flush(&mut self) -> Vec<SyncOperation> {
        let ops: Vec<SyncOperation> = self.queue.drain(..).collect();
        info!("Flushed {} pending offline sync operations to cloud.", ops.len());
        ops
    }

    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_queue_buffering_and_flushing() {
        let mut queue = OfflineSyncQueue::new();
        queue.set_online_status(false);
        assert!(!queue.is_online());

        queue.enqueue(SyncOperation::Delete {
            entity_id: "layout_old".to_string(),
        });

        assert_eq!(queue.pending_count(), 1);

        queue.set_online_status(true);
        let flushed = queue.flush();
        assert_eq!(flushed.len(), 1);
        assert_eq!(queue.pending_count(), 0);
    }
}
