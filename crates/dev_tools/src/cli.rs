use serde::{Deserialize, Serialize};

/// CLI Command options passed into `aether_cli`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CliCommand {
    Status,
    Load { manifest_path: String },
    Unload { widget_id: String },
    Inspect { widget_id: String },
    Snapshot { name: String },
    ToggleGrid { enabled: bool },
}

/// Interactive CLI helper formatting commands into IPC protocol JSON.
pub struct AetherCli;

impl AetherCli {
    pub fn format_ipc_command(cmd: &CliCommand) -> String {
        match cmd {
            CliCommand::Status => "\"GetStatus\"".to_string(),
            CliCommand::Load { manifest_path } => {
                serde_json::json!({ "LoadWidget": { "manifest_path": manifest_path } }).to_string()
            }
            CliCommand::Unload { widget_id } => {
                serde_json::json!({ "UnloadWidget": { "widget_id": widget_id } }).to_string()
            }
            CliCommand::Inspect { widget_id } => {
                serde_json::json!({ "InspectWidget": { "widget_id": widget_id } }).to_string()
            }
            CliCommand::Snapshot { name } => {
                serde_json::json!({ "CreateSnapshot": { "name": name } }).to_string()
            }
            CliCommand::ToggleGrid { enabled } => {
                serde_json::json!({ "ToggleLayoutGrid": { "enabled": *enabled } }).to_string()
            }
        }
    }
}
