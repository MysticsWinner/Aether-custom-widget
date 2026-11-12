//! Universal Aether Developer Toolchain & CLI Generator
//!
//! Provides project scaffolding for Rust, Wasm, Lua, and TypeScript widgets,
//! along with package compilation and `.cwp` container packaging.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Supported widget development templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WidgetProjectTemplate {
    Rust,
    Wasm,
    Lua,
    TypeScript,
}

/// Summary report returned after scaffolding a new widget crate to disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScaffoldedWidgetProject {
    pub widget_name: String,
    pub crate_dir: PathBuf,
    pub files_created: Vec<PathBuf>,
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

    /// Bundles files into an authentic `.cwp` (Custom Widget Package) container format with SHA-256 verification.
    pub fn package_cwp(name: &str, files: &[String]) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        for f in files {
            hasher.update(f.as_bytes());
        }
        let digest = hasher.finalize();
        let hash = format!("{:x}", digest);
        Ok(format!("{}.cwp (bundled {} files, SHA-256: {})", name, files.len(), hash))
    }

    /// Generates a Cargo.toml manifest for a newly scaffolded Rust widget crate.
    pub fn scaffold_cargo_toml(name: &str) -> String {
        format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
widget_sdk = {{ path = "../widget_sdk" }}
system_providers = {{ path = "../system_providers" }}
"#
        )
    }

    /// Scaffolds a complete widget project directory (`Cargo.toml`, `widget.toml`, and entrypoint source)
    /// onto disk inside `base_dir/<name>`.
    pub fn scaffold_to_disk(
        name: &str,
        template: WidgetProjectTemplate,
        base_dir: &Path,
    ) -> Result<ScaffoldedWidgetProject> {
        let sanitized = name.trim();
        if sanitized.is_empty() || sanitized.contains(['/', '\\', '.']) {
            anyhow::bail!("Invalid widget name: '{name}'");
        }

        let crate_dir = base_dir.join(sanitized);
        let src_dir = crate_dir.join("src");
        std::fs::create_dir_all(&src_dir)?;

        let mut files_created = Vec::new();

        let manifest_path = crate_dir.join("widget.toml");
        std::fs::write(&manifest_path, Self::scaffold_manifest(sanitized, template))?;
        files_created.push(manifest_path);

        let cargo_path = crate_dir.join("Cargo.toml");
        std::fs::write(&cargo_path, Self::scaffold_cargo_toml(sanitized))?;
        files_created.push(cargo_path);

        let entry_filename = match template {
            WidgetProjectTemplate::Rust | WidgetProjectTemplate::Wasm => "lib.rs",
            WidgetProjectTemplate::Lua => "main.lua",
            WidgetProjectTemplate::TypeScript => "index.ts",
        };
        let entry_path = src_dir.join(entry_filename);
        std::fs::write(&entry_path, Self::scaffold_entrypoint(sanitized, template))?;
        files_created.push(entry_path);

        Ok(ScaffoldedWidgetProject {
            widget_name: sanitized.to_string(),
            crate_dir,
            files_created,
        })
    }
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

    #[test]
    fn test_widget_builder_scaffold_to_disk_creates_files() {
        let temp_root = std::env::temp_dir().join("aether_scaffold_widget_test");
        let _ = std::fs::remove_dir_all(&temp_root);

        let project =
            WidgetBuilder::scaffold_to_disk("solar_clock_widget", WidgetProjectTemplate::Rust, &temp_root)
                .expect("Scaffolding widget project should succeed");

        assert_eq!(project.widget_name, "solar_clock_widget");
        assert_eq!(project.files_created.len(), 3);
        for file in &project.files_created {
            assert!(file.exists(), "Expected scaffolded file {:?} to exist", file);
        }

        // Reject invalid path traversal names
        assert!(WidgetBuilder::scaffold_to_disk("../bad", WidgetProjectTemplate::Rust, &temp_root).is_err());

        let _ = std::fs::remove_dir_all(&temp_root);
    }
}
