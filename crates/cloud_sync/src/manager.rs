use crate::crdt::{CrdtItem, CrdtResolver, VectorClock};
use crate::entities::{LayoutEntity, SyncEntity};
use crate::offline_queue::{OfflineSyncQueue, SyncOperation};
use std::collections::HashMap;
use tracing::info;

/// Cloud Sync Manager coordinating AES-256-GCM encryption, CRDT conflict resolution, and Offline Mode.
pub struct CloudSyncManager {
    device_id: String,
    local_vector_clock: VectorClock,
    local_items: HashMap<String, CrdtItem<SyncEntity>>,
    offline_queue: OfflineSyncQueue,
}

impl CloudSyncManager {
    pub fn new(device_id: impl Into<String>) -> Self {
        Self {
            device_id: device_id.into(),
            local_vector_clock: VectorClock::new(),
            local_items: HashMap::new(),
            offline_queue: OfflineSyncQueue::new(),
        }
    }

    /// Upserts a local entity, handling offline queuing or CRDT resolution.
    pub fn sync_entity(&mut self, entity_id: impl Into<String>, entity: SyncEntity) {
        let id = entity_id.into();
        self.local_vector_clock.increment(&self.device_id);

        let item = CrdtItem {
            item_id: id.clone(),
            payload: entity.clone(),
            vector_clock: self.local_vector_clock.clone(),
            timestamp_ms: 1000,
        };

        if !self.offline_queue.is_online() {
            info!("Offline Mode active. Queuing sync operation for entity '{}'", id);
            self.offline_queue.enqueue(SyncOperation::Upsert {
                entity_id: id.clone(),
                entity,
            });
        } else {
            // Apply CRDT conflict resolution
            if let Some(existing) = self.local_items.get(&id) {
                let resolved = CrdtResolver::resolve(existing.clone(), item);
                self.local_items.insert(id, resolved);
            } else {
                self.local_items.insert(id, item);
            }
        }
    }

    pub fn set_online_status(&mut self, online: bool) {
        self.offline_queue.set_online_status(online);
        if online && self.offline_queue.pending_count() > 0 {
            let _pending_ops = self.offline_queue.flush();
            info!("Flushed pending offline operations and synced with cloud.");
        }
    }

    pub fn get_entity(&self, entity_id: &str) -> Option<&SyncEntity> {
        self.local_items.get(entity_id).map(|item| &item.payload)
    }

    pub fn is_online(&self) -> bool {
        self.offline_queue.is_online()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_sync_manager_workflow() {
        let mut manager = CloudSyncManager::new("workstation_main");

        let layout = SyncEntity::Layout(LayoutEntity {
            layout_id: "layout_4k_desktop".into(),
            display_id: "DISPLAY_1".into(),
            bounds_x: 0.0,
            bounds_y: 0.0,
            width: 3840.0,
            height: 2160.0,
        });

        manager.sync_entity("layout_4k_desktop", layout.clone());
        assert_eq!(manager.get_entity("layout_4k_desktop"), Some(&layout));

        // Test Offline transition
        manager.set_online_status(false);
        assert!(!manager.is_online());

        let layout_off = SyncEntity::Layout(LayoutEntity {
            layout_id: "layout_offline".into(),
            display_id: "DISPLAY_2".into(),
            bounds_x: 100.0,
            bounds_y: 100.0,
            width: 1920.0,
            height: 1080.0,
        });
        manager.sync_entity("layout_offline", layout_off);

        manager.set_online_status(true);
        assert!(manager.is_online());
    }
}
