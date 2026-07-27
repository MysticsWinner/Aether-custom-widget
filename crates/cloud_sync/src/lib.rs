//! Next-Gen Windows Desktop Customization Platform - Cloud Sync Engine Crate
//!
//! Provides end-to-end encrypted synchronization across 6 core entities (Layouts, Themes, Settings, Plugins, Devices, Accounts)
//! powered by CRDT / Vector Clock Conflict Resolution and an Offline-First Local Cache Architecture.

pub mod benchmark;
pub mod crdt;
pub mod entities;
pub mod manager;
pub mod offline_queue;

pub use benchmark::CloudSyncBenchmark;
pub use crdt::{CrdtItem, CrdtResolver, VectorClock};
pub use entities::{
    AccountEntity, DeviceEntity, LayoutEntity, PluginEntity, SettingsEntity, SyncEntity,
    ThemeEntity,
};
pub use manager::CloudSyncManager;
pub use offline_queue::{OfflineSyncQueue, SyncOperation};
