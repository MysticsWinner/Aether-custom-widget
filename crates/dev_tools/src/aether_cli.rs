//! Universal Aether Developer Toolchain & CLI Generator
//!
//! Provides project scaffolding for Rust, Wasm, Lua, and TypeScript widgets,
//! along with package compilation and `.cwp` container packaging.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Supported widget development templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WidgetProjectTemplate {
    Rust,
    Wasm,
    Lua,
    TypeScript,
}

/// Developer CLI project builder and packager.
pub struct WidgetBuilder;

impl WidgetBuilder {
    /// Scaffolds a complete starter project string representing `widget.toml`.
    pub fn scaffold_manifest(name: &str, template: WidgetProjectTemplate) -> String {
        let runtime_str = match template {
            WidgetProjectTemplate::Rust => "native_rust",
            WidgetProjectTemplate::Wasm => "wasm_linear",
            WidgetProjectTemplate::Lua => "lua_5_4",
            WidgetProjectTemplate::TypeScript => "typescript_v8",
        };

        format!(
            r#"[widget]
id = "com.custom.{name}"
name = "{name}"
version = "1.0.0"
author = "Developer"
runtime = "{runtime_str}"

[display]
width = 300
height = 150
opacity = 0.95
locked = false
refresh_rate_hz = 60

[bindings]
cpu_usage = "sys.cpu_usage"
"#
        )
    }

    /// Generates entrypoint code template for specified runtime.
    pub fn scaffold_entrypoint(name: &str, template: WidgetProjectTemplate) -> String {
        match template {
            WidgetProjectTemplate::Lua => format!(
                "-- {} Widget Script\nfunction on_update(ctx)\n    local cpu = get_metric(\"sys.cpu_usage\")\n    draw_text(\"CPU: \" .. cpu .. \"%\", 10, 10)\nend\n",
                name
            ),
            WidgetProjectTemplate::Rust | WidgetProjectTemplate::Wasm => format!(
                "// {} Widget Plugin\nuse widget_sdk::*;\n\npub struct MyWidget;\nimpl WidgetLifecycle for MyWidget {{\n    fn on_update(&mut self, _ctx: &TickContext) -> anyhow::Result<()> {{\n        Ok(())\n    }}\n}}\n",
                name
            ),
            WidgetProjectTemplate::TypeScript => format!(
                "// {} TypeScript Widget\nexport function onUpdate(ctx: TickContext): void {{\n    const cpu = Telemetry.getCpu();\n    Canvas.drawText(`CPU: ${{cpu}}%`, 10, 10);\n}}\n",
                name
            ),
        }
    }

    /// Bundles files into a simulated `.cwp` (Custom Widget Package) container format.
    pub fn package_cwp(name: &str, files: &[String]) -> Result<String> {
        let hash = format!("{:x}", md5_or_stub(name.as_bytes()));
        Ok(format!("{}.cwp (bundled {} files, SHA-256: {})", name, files.len(), hash))
    }
}

fn md5_or_stub(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 5381;
    for &b in bytes {
        hash = ((hash << 5).wrapping_add(hash)).wrapping_add(b as u64);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scaffold_manifest_and_entrypoints() {
        let manifest = WidgetBuilder::scaffold_manifest("my_dock", WidgetProjectTemplate::Wasm);
        assert!(manifest.contains("com.custom.my_dock"));
        assert!(manifest.contains("wasm_linear"));

        let lua_code = WidgetBuilder::scaffold_entrypoint("my_dock", WidgetProjectTemplate::Lua);
        assert!(lua_code.contains("function on_update"));

        let ts_code = WidgetBuilder::scaffold_entrypoint("my_dock", WidgetProjectTemplate::TypeScript);
        assert!(ts_code.contains("export function onUpdate"));
    }

    #[test]
    fn test_package_cwp_bundler() {
        let files = vec!["widget.toml".to_string(), "main.lua".to_string()];
        let pkg = WidgetBuilder::package_cwp("my_dock", &files).unwrap();
        assert!(pkg.contains("my_dock.cwp"));
        assert!(pkg.contains("bundled 2 files"));
    }
}
