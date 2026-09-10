use serde::{Deserialize, Serialize};

/// Execution state of a widget within its host sandbox.
/// Reflects the 11 formal lifecycle states of the Aether widget runtime:
/// 1. `Unloaded`: Dormant on disk, zero host memory allocation.
/// 2. `Loading`: Allocating resources, parsing manifest, compiling Lua scripts.
/// 3. `Loaded`: In memory, pre-warmed, ready for visual attachment.
/// 4. `Mounting`: Attaching DirectComposition visual tree / HWND layer to desktop.
/// 5. `Active`: Fully mounted and ticking at target refresh rate.
/// 6. `Mounted`: Canonical mounted state alias for backward compatibility with Active.
/// 7. `Paused`: Desktop occluded, full-screen 3D app active, or virtual desktop inactive.
/// 8. `Degraded`: Frame budget exceeded (> 16.6ms); throttling frame rate or dropping blur.
/// 9. `Unmounting`: Detaching visual nodes cleanly from composition tree.
/// 10. `Unloading`: Releasing GPU resources and tearing down AppContainer sandbox.
/// 11. `Unmounted`: Canonical unmounted state alias for backward compatibility.
/// 12. `Error`: Trapped unhandled exception; eligible for auto-recovery restart.
/// 13. `Quarantined`: Repeated crash threshold exceeded; permanently isolated until user reset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WidgetState {
    Unloaded,
    Loading,
    Loaded,
    Mounting,
    Active,
    Mounted,
    Paused,
    Degraded,
    Unmounting,
    Unloading,
    Unmounted,
    Error,
    Quarantined,
}

impl WidgetState {
    /// Returns true if the widget is actively rendering or mounted on the canvas.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active | Self::Mounted)
    }

    /// Returns true if the widget is in an operational state (loaded, active, paused, or degraded).
    pub fn is_operational(&self) -> bool {
        matches!(
            self,
            Self::Loaded | Self::Mounted | Self::Active | Self::Paused | Self::Degraded
        )
    }

    /// Returns true if the widget has failed or been quarantined.
    pub fn is_faulted(&self) -> bool {
        matches!(self, Self::Error | Self::Quarantined)
    }
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

    #[test]
    fn test_widget_state_classification_helpers() {
        assert!(WidgetState::Active.is_active());
        assert!(WidgetState::Mounted.is_active());
        assert!(!WidgetState::Paused.is_active());
        assert!(!WidgetState::Unloaded.is_active());

        assert!(WidgetState::Loaded.is_operational());
        assert!(WidgetState::Active.is_operational());
        assert!(WidgetState::Mounted.is_operational());
        assert!(WidgetState::Paused.is_operational());
        assert!(WidgetState::Degraded.is_operational());
        assert!(!WidgetState::Unloaded.is_operational());
        assert!(!WidgetState::Error.is_operational());
        assert!(!WidgetState::Quarantined.is_operational());

        assert!(WidgetState::Error.is_faulted());
        assert!(WidgetState::Quarantined.is_faulted());
        assert!(!WidgetState::Active.is_faulted());
    }
}
