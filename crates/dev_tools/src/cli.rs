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
    Gc { aggressive: bool, trim_working_set: bool },
    SetDevMode { enabled: bool, aggressive_gc: bool, interval_secs: u64 },
    GcStats,
    WidgetCreate { name: String, template: Option<String> },
}

/// Interactive CLI helper formatting commands into IPC protocol JSON and parsing CLI arguments.
pub struct AetherCli;

impl AetherCli {
    /// Parses command-line arguments (e.g. `aether widget create <name>`) into a structured `CliCommand`.
    pub fn parse_args(args: &[String]) -> Option<CliCommand> {
        let tokens: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let slice = if tokens.first().map(|s| s.ends_with("aether") || s.ends_with("aether.exe")).unwrap_or(false) {
            &tokens[1..]
        } else {
            &tokens[..]
        };

        match slice {
            ["status"] => Some(CliCommand::Status),
            ["gc-stats"] => Some(CliCommand::GcStats),
            ["load", path] => Some(CliCommand::Load {
                manifest_path: (*path).to_string(),
            }),
            ["unload", id] => Some(CliCommand::Unload {
                widget_id: (*id).to_string(),
            }),
            ["inspect", id] => Some(CliCommand::Inspect {
                widget_id: (*id).to_string(),
            }),
            ["snapshot", name] => Some(CliCommand::Snapshot {
                name: (*name).to_string(),
            }),
            ["widget", "create", name] => Some(CliCommand::WidgetCreate {
                name: (*name).to_string(),
                template: None,
            }),
            ["widget", "create", name, "--template", tpl] => Some(CliCommand::WidgetCreate {
                name: (*name).to_string(),
                template: Some((*tpl).to_string()),
            }),
            _ => None,
        }
    }

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
            CliCommand::Gc { aggressive, trim_working_set } => {
                serde_json::json!({ "TriggerGarbageCollection": { "aggressive": *aggressive, "trim_working_set": *trim_working_set } }).to_string()
            }
            CliCommand::SetDevMode { enabled, aggressive_gc, interval_secs } => {
                serde_json::json!({ "SetGarbageCollectionPolicy": { "dev_mode": *enabled, "aggressive": *aggressive_gc, "interval_secs": *interval_secs } }).to_string()
            }
            CliCommand::GcStats => "\"GetGarbageCollectionStats\"".to_string(),
            CliCommand::WidgetCreate { name, template } => {
                serde_json::json!({
                    "ScaffoldWidget": {
                        "name": name,
                        "template": template.as_deref().unwrap_or("rust")
                    }
                })
                .to_string()
            }
        }
    }
}

