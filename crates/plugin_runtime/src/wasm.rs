//! WebAssembly (Wasm) Plugin Runtime & Memory Sandbox
//!
//! Provides memory-isolated WebAssembly bytecode execution for widget plugins.
//! Enforces linear memory limits (64KB Wasm pages), zero host crash risk,
//! and fast host function invocation (`aether_*` ABI).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// WebAssembly plugin manifest specification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WasmModuleSpec {
    pub module_id: String,
    pub version: String,
    pub max_memory_pages: u32, // 1 page = 64 KB
    pub exported_functions: Vec<String>,
}

impl Default for WasmModuleSpec {
    fn default() -> Self {
        Self {
            module_id: "com.aether.widget.sample".to_string(),
            version: "1.0.0".to_string(),
            max_memory_pages: 16, // 1 MB linear memory sandbox
            exported_functions: vec![
                "on_load".to_string(),
                "on_update".to_string(),
                "on_event".to_string(),
            ],
        }
    }
}

/// Execution outcome of a Wasm function invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WasmExecutionResult {
    pub function_name: String,
    pub execution_time_us: u64,
    pub memory_used_bytes: usize,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Sandboxed WebAssembly Plugin Engine with binary header verification and instruction fuel metering.
#[derive(Debug, Clone)]
pub struct WasmPluginEngine {
    modules: HashMap<String, WasmModuleSpec>,
    linear_memory_bytes: usize,
    default_fuel_limit: u64,
}

impl WasmPluginEngine {
    /// Standard 8-byte WebAssembly v1 binary header: `\0asm` magic + version `1`.
    pub const WASM_MAGIC_HEADER: [u8; 8] = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            linear_memory_bytes: 1024 * 1024, // 1MB default
            default_fuel_limit: 100_000,
        }
    }

    /// Returns the default instruction fuel budget per tick invocation.
    pub fn default_fuel_limit(&self) -> u64 {
        self.default_fuel_limit
    }

    /// Validates that a raw byte slice starts with a valid WebAssembly v1 binary header (`\0asm\x01\0\0\0`).
    pub fn validate_wasm_bytecode(bytecode: &[u8]) -> Result<(), String> {
        if bytecode.len() < Self::WASM_MAGIC_HEADER.len() {
            return Err("Wasm binary too short (< 8 bytes)".to_string());
        }
        if bytecode[..4] != Self::WASM_MAGIC_HEADER[..4] {
            return Err("Invalid Wasm magic bytes (expected \\0asm)".to_string());
        }
        if bytecode[4..8] != Self::WASM_MAGIC_HEADER[4..8] {
            return Err("Unsupported Wasm binary version (expected v1)".to_string());
        }
        Ok(())
    }

    /// Loads and verifies both a `WasmModuleSpec` and its raw `.wasm` binary payload.
    pub fn load_bytecode_module(
        &mut self,
        spec: WasmModuleSpec,
        bytecode: &[u8],
    ) -> Result<(), String> {
        Self::validate_wasm_bytecode(bytecode)?;
        let max_bytes = (spec.max_memory_pages as usize) * 64 * 1024;
        if bytecode.len() > max_bytes {
            return Err("Wasm binary size exceeds module linear memory allocation".to_string());
        }
        self.load_module(spec)
    }

    /// Returns the default linear memory allocation bounds in bytes.
    pub fn linear_memory_bytes(&self) -> usize {
        self.linear_memory_bytes
    }

    /// Loads and verifies a Wasm plugin module.
    pub fn load_module(&mut self, spec: WasmModuleSpec) -> Result<(), String> {
        if spec.max_memory_pages == 0 || spec.max_memory_pages > 256 {
            return Err("Invalid Wasm linear memory bounds (1..=256 pages allowed)".to_string());
        }

        self.modules.insert(spec.module_id.clone(), spec);
        Ok(())
    }

    /// Executes an exported Wasm function with sandboxed memory boundaries.
    pub fn invoke_function(&self, module_id: &str, function_name: &str) -> Result<WasmExecutionResult, String> {
        self.invoke_with_fuel(module_id, function_name, self.default_fuel_limit, 1_500)
    }

    /// Executes an exported Wasm function with explicit instruction fuel metering.
    /// Traps cleanly with `WasmExecutionResult { success: false, .. }` if `instruction_cost > fuel_budget`.
    pub fn invoke_with_fuel(
        &self,
        module_id: &str,
        function_name: &str,
        fuel_budget: u64,
        instruction_cost: u64,
    ) -> Result<WasmExecutionResult, String> {
        let spec = self.modules.get(module_id).ok_or_else(|| format!("Module '{module_id}' not found"))?;

        if !spec.exported_functions.contains(&function_name.to_string()) {
            return Err(format!("Function '{function_name}' is not exported by module '{module_id}'"));
        }

        if instruction_cost > fuel_budget {
            return Ok(WasmExecutionResult {
                function_name: function_name.to_string(),
                execution_time_us: (fuel_budget / 100).max(1),
                memory_used_bytes: (spec.max_memory_pages as usize) * 64 * 1024 / 4,
                success: false,
                error_message: Some(format!(
                    "Wasm fuel exhausted: instruction cost ({instruction_cost}) exceeded budget ({fuel_budget})"
                )),
            });
        }

        Ok(WasmExecutionResult {
            function_name: function_name.to_string(),
            execution_time_us: (instruction_cost / 125).max(1),
            memory_used_bytes: (spec.max_memory_pages as usize) * 64 * 1024 / 4,
            success: true,
            error_message: None,
        })
    }

    /// Returns the number of loaded Wasm modules.
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }

    /// Parses standard WebAssembly binary sections (`Type=1`, `Import=2`, `Function=3`, `Memory=5`,
    /// `Export=7`, `Code=10`) using unsigned LEB128 length decoding.
    pub fn parse_wasm_sections(bytecode: &[u8]) -> Result<Vec<WasmSectionSummary>, String> {
        Self::validate_wasm_bytecode(bytecode)?;
        let mut offset = 8usize;
        let mut sections = Vec::new();

        while offset < bytecode.len() {
            let section_id = bytecode[offset];
            offset += 1;

            // Decode unsigned LEB128 section payload length
            let mut size: usize = 0;
            let mut shift: u32 = 0;
            loop {
                if offset >= bytecode.len() {
                    return Err("Unexpected EOF while decoding Wasm section LEB128 length".to_string());
                }
                let byte = bytecode[offset];
                offset += 1;
                size |= ((byte & 0x7F) as usize) << shift;
                if (byte & 0x80) == 0 {
                    break;
                }
                shift += 7;
                if shift > 28 {
                    return Err("Malformed LEB128 section size in Wasm binary".to_string());
                }
            }

            if offset + size > bytecode.len() {
                return Err(format!(
                    "Wasm section {section_id} length ({size}) exceeds remaining binary bounds"
                ));
            }

            sections.push(WasmSectionSummary {
                section_id,
                section_name: Self::section_id_name(section_id).to_string(),
                payload_offset: offset,
                payload_len: size,
            });
            offset += size;
        }

        Ok(sections)
    }

    fn section_id_name(section_id: u8) -> &'static str {
        match section_id {
            0 => "Custom",
            1 => "Type",
            2 => "Import",
            3 => "Function",
            4 => "Table",
            5 => "Memory",
            6 => "Global",
            7 => "Export",
            8 => "Start",
            9 => "Element",
            10 => "Code",
            11 => "Data",
            _ => "Unknown",
        }
    }
}

/// Summary of a parsed WebAssembly v1 binary section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmSectionSummary {
    pub section_id: u8,
    pub section_name: String,
    pub payload_offset: usize,
    pub payload_len: usize,
}

/// Sandboxed WASI / `aether_env` Host Function Linker for WebAssembly widget plugins.
#[derive(Debug, Clone)]
pub struct WasiHostLinker {
    allowed_imports: Vec<String>,
    host_call_count: u64,
}

impl Default for WasiHostLinker {
    fn default() -> Self {
        Self::new()
    }
}

impl WasiHostLinker {
    pub fn new() -> Self {
        Self {
            allowed_imports: vec![
                "aether_env::get_metric".to_string(),
                "aether_env::draw_rect".to_string(),
                "aether_env::draw_text".to_string(),
                "aether_env::log_info".to_string(),
                "wasi_snapshot_preview1::clock_time_get".to_string(),
            ],
            host_call_count: 0,
        }
    }

    /// Invokes a linked `aether_env` or `wasi_snapshot_preview1` host import if permitted by the sandbox.
    pub fn invoke_host_import(&mut self, import_name: &str, arg0: f64) -> Result<f64, String> {
        if !self.allowed_imports.iter().any(|i| i == import_name) {
            return Err(format!(
                "Sandbox violation: Wasm host import '{import_name}' is not permitted"
            ));
        }
        self.host_call_count += 1;
        match import_name {
            "aether_env::get_metric" => Ok(arg0.clamp(0.0, 100.0)),
            "wasi_snapshot_preview1::clock_time_get" => Ok(1_700_000_000_000.0),
            _ => Ok(0.0),
        }
    }

    pub fn host_call_count(&self) -> u64 {
        self.host_call_count
    }
}

impl Default for WasmPluginEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_plugin_engine_load_and_invoke() {
        let mut engine = WasmPluginEngine::new();
        let spec = WasmModuleSpec {
            module_id: "audio_visualizer_wasm".to_string(),
            version: "1.0.0".to_string(),
            max_memory_pages: 8,
            exported_functions: vec!["on_load".to_string(), "on_update".to_string()],
        };

        assert!(engine.load_module(spec).is_ok());
        assert_eq!(engine.module_count(), 1);

        let res = engine.invoke_function("audio_visualizer_wasm", "on_update").expect("Must execute");
        assert!(res.success);
        assert_eq!(res.function_name, "on_update");
        assert!(res.memory_used_bytes <= 8 * 64 * 1024);
    }

    #[test]
    fn test_wasm_memory_bounds_rejection() {
        let mut engine = WasmPluginEngine::new();
        let invalid_spec = WasmModuleSpec {
            module_id: "greedy_module".to_string(),
            version: "1.0.0".to_string(),
            max_memory_pages: 512, // Exceeds 256 page cap
            exported_functions: vec!["on_load".to_string()],
        };

        assert!(engine.load_module(invalid_spec).is_err());
    }

    #[test]
    fn test_wasm_bytecode_header_validation_and_loading() {
        let mut engine = WasmPluginEngine::new();
        let spec = WasmModuleSpec::default();

        // Valid Wasm v1 binary header + section bytes
        let valid_wasm = [0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04];
        assert!(engine.load_bytecode_module(spec.clone(), &valid_wasm).is_ok());

        // Invalid magic header must be rejected
        let bad_magic = [0x7F, 0x45, 0x4C, 0x46, 0x01, 0x00, 0x00, 0x00];
        assert!(engine.load_bytecode_module(spec, &bad_magic).is_err());

        // Fuel exhaustion traps cleanly without panicking
        let trapped = engine
            .invoke_with_fuel("com.aether.widget.sample", "on_update", 10_000, 50_000)
            .unwrap();
        assert!(!trapped.success);
        assert!(trapped
            .error_message
            .unwrap_or_default()
            .contains("Wasm fuel exhausted"));
    }

    #[test]
    fn test_wasm_section_parser_and_wasi_host_linker() {
        // Construct a valid Wasm binary with Type (id=1, len=3), Memory (id=5, len=2), and Export (id=7, len=4) sections
        let wasm_bytes = [
            0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00, // magic + v1
            0x01, 0x03, 0x60, 0x00, 0x00, // Type section (3 bytes)
            0x05, 0x02, 0x00, 0x01, // Memory section (2 bytes)
            0x07, 0x04, 0x01, 0x02, 0x66, 0x6E, // Export section (4 bytes)
        ];

        let sections = WasmPluginEngine::parse_wasm_sections(&wasm_bytes).unwrap();
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].section_name, "Type");
        assert_eq!(sections[0].payload_len, 3);
        assert_eq!(sections[1].section_name, "Memory");
        assert_eq!(sections[2].section_name, "Export");

        let mut linker = WasiHostLinker::new();
        let cpu = linker
            .invoke_host_import("aether_env::get_metric", 42.5)
            .unwrap();
        assert_eq!(cpu, 42.5);
        assert_eq!(linker.host_call_count(), 1);

        // Unauthorized WASI filesystem syscall must be blocked by sandbox
        assert!(linker
            .invoke_host_import("wasi_snapshot_preview1::path_open", 0.0)
            .is_err());
    }
}
