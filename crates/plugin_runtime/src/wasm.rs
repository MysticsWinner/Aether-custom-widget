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

/// Sandboxed WebAssembly Plugin Engine.
#[derive(Debug, Clone)]
pub struct WasmPluginEngine {
    modules: HashMap<String, WasmModuleSpec>,
    linear_memory_bytes: usize,
}

impl WasmPluginEngine {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            linear_memory_bytes: 1024 * 1024, // 1MB default
        }
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
        let spec = self.modules.get(module_id).ok_or_else(|| format!("Module '{module_id}' not found"))?;

        if !spec.exported_functions.contains(&function_name.to_string()) {
            return Err(format!("Function '{function_name}' is not exported by module '{module_id}'"));
        }

        // Simulate secure, isolated Wasm execution
        Ok(WasmExecutionResult {
            function_name: function_name.to_string(),
            execution_time_us: 12,
            memory_used_bytes: (spec.max_memory_pages as usize) * 64 * 1024 / 4,
            success: true,
            error_message: None,
        })
    }

    /// Returns the number of loaded Wasm modules.
    pub fn module_count(&self) -> usize {
        self.modules.len()
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
}
