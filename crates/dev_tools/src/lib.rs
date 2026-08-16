pub mod cli;
pub mod discovery;
pub mod hot_reload;
pub mod inspector;
pub mod layout_grid;

pub use cli::{AetherCli, CliCommand};
pub use discovery::WidgetDiscoveryScanner;
pub use hot_reload::DevHotReloader;
pub use inspector::{WidgetInspectionReport, WidgetInspector};
pub use layout_grid::LayoutGridOverlay;

#[cfg(test)]
mod tests {
    use super::*;
    use widget_sdk::RectF;

    #[test]
    fn test_dev_hot_reloader_detects_file_change() {
        let mut reloader = DevHotReloader::new();
        reloader.watch_directory("C:/widgets");

        let res = reloader.notify_file_change("C:/widgets/clock_widget.lua");
        assert_eq!(res, Some("clock_widget".to_string()));

        let pending = reloader.drain_reloads();
        assert_eq!(pending, vec!["clock_widget".to_string()]);
    }

    #[test]
    fn test_widget_inspector_builds_dom_tree_report() {
        let bounds = RectF {
            x: 100.0,
            y: 200.0,
            width: 300.0,
            height: 150.0,
        };
        let report = WidgetInspector::inspect("weather_w", bounds, 15, 1024.0, 450, 60);

        assert_eq!(report.widget_id, "weather_w");
        assert_eq!(report.draw_command_count, 15);
        assert_eq!(report.tick_duration_us, 450);
    }

    #[test]
    fn test_aether_cli_command_formatting() {
        let status_cmd = CliCommand::Status;
        assert_eq!(AetherCli::format_ipc_command(&status_cmd), "\"GetStatus\"");

        let load_cmd = CliCommand::Load {
            manifest_path: "widget.toml".to_string(),
        };
        assert!(AetherCli::format_ipc_command(&load_cmd).contains("LoadWidget"));
    }

    #[test]
    fn test_layout_grid_overlay_draw_commands() {
        let mut grid = LayoutGridOverlay::default();
        let bounds = RectF {
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 100.0,
        };

        // Disabled -> empty
        assert!(grid.generate_widget_bounds_overlay(bounds, "w1").is_empty());

        // Enabled -> 2 commands (Rect + Text)
        grid.set_enabled(true);
        let cmds = grid.generate_widget_bounds_overlay(bounds, "w1");
        assert_eq!(cmds.len(), 2);
    }
}
