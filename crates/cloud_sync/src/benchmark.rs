use crate::entities::{LayoutEntity, SyncEntity};
use crate::manager::CloudSyncManager;
use std::time::Instant;
use tracing::info;

/// Performance benchmark harness evaluating Cloud Sync Engine CRDT resolution and payload serialization throughput.
pub struct CloudSyncBenchmark;

impl CloudSyncBenchmark {
    pub fn run_benchmark() {
        let mut manager = CloudSyncManager::new("workstation_bench");
        let entity_count = 1_000usize;

        let start = Instant::now();
        for i in 0..entity_count {
            let layout = SyncEntity::Layout(LayoutEntity {
                layout_id: format!("layout_bench_{}", i),
                display_id: "DISPLAY_1".into(),
                bounds_x: i as f32,
                bounds_y: i as f32,
                width: 300.0,
                height: 200.0,
            });
            manager.sync_entity(format!("layout_bench_{}", i), layout);
        }
        let elapsed = start.elapsed();

        let throughput = (entity_count as f64) / elapsed.as_secs_f64();
        info!(
            "Cloud Sync Benchmark: {} CRDT Sync Operations = {:?} ({:.0} ops / sec)",
            entity_count, elapsed, throughput
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_sync_benchmark_execution() {
        CloudSyncBenchmark::run_benchmark();
    }
}
