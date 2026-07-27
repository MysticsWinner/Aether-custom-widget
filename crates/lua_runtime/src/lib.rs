use mlua::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LuaRuntimeError {
    #[error("Lua execution error: {0}")]
    LuaError(#[from] LuaError),
}

pub struct EmbeddedLuaPluginHost {
    lua: Lua,
}

impl EmbeddedLuaPluginHost {
    pub fn new() -> Result<Self, LuaRuntimeError> {
        let lua = Lua::new();

        // Register restricted safe APIs for widget scripts
        let globals = lua.globals();
        
        let print_fn = lua.create_function(|_, msg: String| {
            tracing::info!(target: "lua_plugin", "{}", msg);
            Ok(())
        })?;
        globals.set("log_info", print_fn)?;

        Ok(Self { lua })
    }

    /// Execute a Lua widget script string safely
    pub fn execute_script(&self, script: &str) -> Result<(), LuaRuntimeError> {
        self.lua.load(script).exec()?;
        Ok(())
    }
}
