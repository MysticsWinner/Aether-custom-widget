use serde::{Deserialize, Serialize};

/// Execution state of a widget within its host sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WidgetState {
    Unloaded,
    Loaded,
    Mounted,
    Unmounted,
}

/// Context provided to widgets on every update tick.
#[derive(Debug, Clone, Copy)]
pub struct TickContext {
    pub timestamp_ms: u64,
    pub delta_time_ms: f32,
    pub frame_index: u64,
}

/// 1. Lifecycle API Pillar
/// Trait implemented by every 3rd-party widget to hook into execution state transitions.
pub trait WidgetLifecycle: Send + Sync {
    /// Called when the widget is first loaded into memory.
    fn on_load(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Called when the widget visual tree is mounted onto the desktop canvas.
    fn on_mount(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Called periodically on tick pass to update widget state and logic.
    fn on_update(&mut self, _ctx: &TickContext) -> anyhow::Result<()> {
        Ok(())
    }

    /// Called when the widget is unmounted from display surface.
    fn on_unmount(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Called when the widget is unloaded and its sandbox is cleaned up.
    fn on_unload(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Returns current widget execution state.
    fn state(&self) -> WidgetState;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWidget {
        state: WidgetState,
        tick_count: u64,
    }

    impl WidgetLifecycle for TestWidget {
        fn on_load(&mut self) -> anyhow::Result<()> {
            self.state = WidgetState::Loaded;
            Ok(())
        }

        fn on_mount(&mut self) -> anyhow::Result<()> {
            self.state = WidgetState::Mounted;
            Ok(())
        }

        fn on_update(&mut self, _ctx: &TickContext) -> anyhow::Result<()> {
            self.tick_count += 1;
            Ok(())
        }

        fn on_unmount(&mut self) -> anyhow::Result<()> {
            self.state = WidgetState::Unmounted;
            Ok(())
        }

        fn state(&self) -> WidgetState {
            self.state
        }
    }

    #[test]
    fn test_lifecycle_transitions() {
        let mut widget = TestWidget {
            state: WidgetState::Unloaded,
            tick_count: 0,
        };

        assert_eq!(widget.state(), WidgetState::Unloaded);

        widget.on_load().unwrap();
        assert_eq!(widget.state(), WidgetState::Loaded);

        widget.on_mount().unwrap();
        assert_eq!(widget.state(), WidgetState::Mounted);

        let ctx = TickContext {
            timestamp_ms: 1000,
            delta_time_ms: 16.6,
            frame_index: 1,
        };
        widget.on_update(&ctx).unwrap();
        assert_eq!(widget.tick_count, 1);

        widget.on_unmount().unwrap();
        assert_eq!(widget.state(), WidgetState::Unmounted);
    }
}
