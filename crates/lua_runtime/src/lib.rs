use layout_engine::WidgetPositionStore;
use mlua::prelude::*;
use system_providers::SharedTelemetryCache;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LuaRuntimeError {
    #[error("Lua execution error: {0}")]
    LuaError(#[from] LuaError),
}

pub struct EmbeddedLuaPluginHost {
    lua: Lua,
    cache: SharedTelemetryCache,
    pos_store: WidgetPositionStore,
}

impl EmbeddedLuaPluginHost {
    pub fn new() -> Result<Self, LuaRuntimeError> {
        Self::with_providers(SharedTelemetryCache::new(), WidgetPositionStore::in_memory())
    }

    pub fn with_providers(
        cache: SharedTelemetryCache,
        pos_store: WidgetPositionStore,
    ) -> Result<Self, LuaRuntimeError> {
        let lua = Lua::new();

        // Register restricted safe APIs for widget scripts
        {
            let globals = lua.globals();

            let print_fn = lua.create_function(|_, msg: String| {
                tracing::info!(target: "lua_plugin", "{}", msg);
                Ok(())
            })?;
            globals.set("log_info", print_fn)?;

            let warn_fn = lua.create_function(|_, msg: String| {
                tracing::warn!(target: "lua_plugin", "{}", msg);
                Ok(())
            })?;
            globals.set("log_warn", warn_fn)?;

            let ver_fn = lua.create_function(|_, (): ()| {
                Ok("1.0.0".to_string())
            })?;
            globals.set("get_api_version", ver_fn)?;

            // ── Telemetry Bindings ───────────────────────────────────────────
            let cache_c1 = cache.clone();
            let cpu_fn = lua.create_function(move |_, (): ()| {
                Ok(cache_c1.get_cpu_pct())
            })?;
            globals.set("get_cpu_pct", cpu_fn)?;

            let cache_c2 = cache.clone();
            let gpu_fn = lua.create_function(move |_, (): ()| {
                Ok(cache_c2.get_snapshot().gpu_usage_pct)
            })?;
            globals.set("get_gpu_pct", gpu_fn)?;

            let cache_c3 = cache.clone();
            let mem_fn = lua.create_function(move |_, (): ()| {
                let snap = cache_c3.get_snapshot();
                Ok((snap.memory_used_mb, snap.memory_total_mb))
            })?;
            globals.set("get_memory_mb", mem_fn)?;

            let cache_c4 = cache.clone();
            let net_fn = lua.create_function(move |_, (): ()| {
                Ok(cache_c4.get_snapshot().net_recv_bytes_per_sec)
            })?;
            globals.set("get_net_rate", net_fn)?;

            // ── Position & Lock Bindings ─────────────────────────────────────
            let pos_store_c1 = pos_store.clone();
            let pos_fn = lua.create_function(move |_, widget_id: String| {
                if let Some((x, y)) = pos_store_c1.get_position(&widget_id) {
                    Ok((Some(x), Some(y)))
                } else {
                    Ok((None, None))
                }
            })?;
            globals.set("get_widget_position", pos_fn)?;

            let pos_store_c2 = pos_store.clone();
            let lock_fn = lua.create_function(move |_, widget_id: String| {
                Ok(pos_store_c2.is_locked(&widget_id))
            })?;
            globals.set("is_widget_locked", lock_fn)?;
        }

        Ok(Self { lua, cache, pos_store })
    }

    /// Execute a Lua widget script string safely
    pub fn execute_script(&self, script: &str) -> Result<(), LuaRuntimeError> {
        self.lua.load(script).exec()?;
        Ok(())
    }

    /// Evaluates a Lua expression and returns result as String
    pub fn eval_script(&self, script: &str) -> Result<String, LuaRuntimeError> {
        let res: String = self.lua.load(script).eval()?;
        Ok(res)
    }

    pub fn cache(&self) -> &SharedTelemetryCache {
        &self.cache
    }

    pub fn position_store(&self) -> &WidgetPositionStore {
        &self.pos_store
    }
}

impl Default for EmbeddedLuaPluginHost {
    fn default() -> Self {
        Self::new().expect("Failed to initialize default EmbeddedLuaPluginHost")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use system_providers::TelemetrySnapshot;

    #[test]
    fn test_lua_runtime_execution() {
        let host = EmbeddedLuaPluginHost::new().unwrap();
        let script = r#"
            log_info("Hello from Lua widget!")
            log_warn("Warning from Lua widget!")
            local ver = get_api_version()
            assert(ver == "1.0.0")
        "#;
        assert!(host.execute_script(script).is_ok());
    }

    #[test]
    fn test_lua_telemetry_and_position_bindings() {
        let cache = SharedTelemetryCache::new();
        let mut snap = TelemetrySnapshot::default();
        snap.cpu_usage_pct = 54.5;
        snap.gpu_usage_pct = 22.1;
        snap.memory_used_mb = 8192.0;
        snap.memory_total_mb = 16384.0;
        snap.net_recv_bytes_per_sec = 1048576;
        cache.update_snapshot(snap);

        let pos_store = WidgetPositionStore::in_memory();
        pos_store.set_position("perf_monitor_widget", 320, 180).unwrap();
        pos_store.set_locked("perf_monitor_widget", true).unwrap();

        let host = EmbeddedLuaPluginHost::with_providers(cache, pos_store).unwrap();

        let script = r#"
            local cpu = get_cpu_pct()
            local gpu = get_gpu_pct()
            local used, total = get_memory_mb()
            local net = get_net_rate()
            local x, y = get_widget_position("perf_monitor_widget")
            local locked = is_widget_locked("perf_monitor_widget")

            assert(cpu > 50.0, "CPU should match snapshot")
            assert(gpu > 20.0, "GPU should match snapshot")
            assert(used == 8192.0, "RAM used match")
            assert(total == 16384.0, "RAM total match")
            assert(net == 1048576, "Net rate match")
            assert(x == 320 and y == 180, "Position match")
            assert(locked == true, "Lock state match")

            return string.format("CPU=%.1f%% RAM=%.0fMB Pos=(%d,%d)", cpu, used, x, y)
        "#;

        let res = host.eval_script(script).unwrap();
        assert!(res.contains("CPU=54.5%"));
        assert!(res.contains("Pos=(320,180)"));
    }
}

