pub mod backup;
pub mod manager;
pub mod migration;
pub mod snapshot;
pub mod transaction;
pub mod types;
pub mod validator;

pub use backup::ConfigBackupRotator;
pub use manager::ConfigManager;
pub use migration::{Migration, MigrationEngine};
pub use snapshot::SnapshotManager;
pub use transaction::ConfigTransaction;
pub use types::{ConfigHeader, Snapshot, SnapshotMeta, WidgetSnapshot};
pub use validator::ConfigValidator;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    struct TestV1ToV2Migration;
    impl Migration for TestV1ToV2Migration {
        fn target_version(&self) -> u32 {
            2
        }
        fn apply(&self, value: &mut serde_json::Value) -> anyhow::Result<()> {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("v2_field".to_string(), json!("migrated_value"));
            }
            Ok(())
        }
    }

    struct TestV2ToV3Migration;
    impl Migration for TestV2ToV3Migration {
        fn target_version(&self) -> u32 {
            3
        }
        fn apply(&self, value: &mut serde_json::Value) -> anyhow::Result<()> {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("v3_field".to_string(), json!(100));
            }
            Ok(())
        }
    }

    #[test]
    fn test_transaction_atomic_write() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let transaction = ConfigTransaction::new(&path);

        let payload = json!({
            "schema_version": 1,
            "theme": "dark"
        });
        transaction.write_atomic(&payload).unwrap();

        assert!(path.exists());
        let read_back: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(read_back["theme"], "dark");
    }

    #[test]
    fn test_transaction_backup_rotation() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let transaction = ConfigTransaction::new(&path);

        // Write generation 1
        transaction.write_atomic(&json!({"schema_version": 1, "v": 1})).unwrap();
        // Write generation 2
        transaction.write_atomic(&json!({"schema_version": 1, "v": 2})).unwrap();

        let bak1 = dir.path().join("settings.json.bak.json");
        assert!(bak1.exists());
        let bak1_val: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&bak1).unwrap()).unwrap();
        assert_eq!(bak1_val["v"], 1);
    }

    #[test]
    fn test_migration_engine_v1_to_v3_chain() {
        let mut engine = MigrationEngine::new();
        engine.register(TestV1ToV2Migration);
        engine.register(TestV2ToV3Migration);

        let mut payload = json!({
            "schema_version": 1,
            "initial": true
        });

        let new_version = engine.migrate(&mut payload, 3).unwrap();
        assert_eq!(new_version, 3);
        assert_eq!(payload["schema_version"], 3);
        assert_eq!(payload["v2_field"], "migrated_value");
        assert_eq!(payload["v3_field"], 100);
    }

    #[test]
    fn test_snapshot_create_and_list() {
        let dir = tempdir().unwrap();
        let manager = SnapshotManager::new(dir.path(), 5);

        let snapshot = manager
            .create_snapshot(
                "My Work Layout",
                json!({"theme": "dark"}),
                json!({"widgets": ["cpu", "ram"]}),
                json!({"accent": "#0078D4"}),
                10000,
            )
            .unwrap();

        assert_eq!(snapshot.name, "My Work Layout");

        let list = manager.list_snapshots().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, snapshot.id);
        assert_eq!(list[0].name, "My Work Layout");
    }

    #[test]
    fn test_snapshot_restore_and_get() {
        let dir = tempdir().unwrap();
        let manager = SnapshotManager::new(dir.path(), 5);

        let snapshot = manager
            .create_snapshot(
                "Backup 1",
                json!({"val": 42}),
                json!({"pos": "top"}),
                json!({"color": "red"}),
                5000,
            )
            .unwrap();

        let retrieved = manager.get_snapshot(&snapshot.id).unwrap();
        assert_eq!(retrieved.settings["val"], 42);
        assert_eq!(retrieved.layout["pos"], "top");
    }

    #[test]
    fn test_snapshot_export_and_import() {
        let dir = tempdir().unwrap();
        let manager = SnapshotManager::new(dir.path().join("snapshots"), 5);

        let snapshot = manager
            .create_snapshot("Export test", json!({"a": 1}), json!({"b": 2}), json!({"c": 3}), 1000)
            .unwrap();

        let export_file = dir.path().join("exported.snapshot.json");
        manager.export_snapshot(&snapshot.id, &export_file).unwrap();
        assert!(export_file.exists());

        // Import into second manager instance
        let manager2 = SnapshotManager::new(dir.path().join("snapshots2"), 5);
        let imported = manager2.import_snapshot(&export_file).unwrap();
        assert_eq!(imported.id, snapshot.id);
        assert_eq!(imported.name, "Export test");
    }

    #[test]
    fn test_snapshot_rotation_deletes_oldest() {
        let dir = tempdir().unwrap();
        let manager = SnapshotManager::new(dir.path(), 2); // Max 2 snapshots

        let s1 = manager.create_snapshot("S1", json!({}), json!({}), json!({}), 1000).unwrap();
        let _s2 = manager.create_snapshot("S2", json!({}), json!({}), json!({}), 2000).unwrap();
        let _s3 = manager.create_snapshot("S3", json!({}), json!({}), json!({}), 3000).unwrap();

        let list = manager.list_snapshots().unwrap();
        assert_eq!(list.len(), 2);
        // S1 (oldest) should have been auto-deleted by rotation
        assert!(!list.iter().any(|s| s.id == s1.id));
    }
}
