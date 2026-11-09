//! Native Rust `cdylib` C-ABI Plugin Interface (Phase 26)
//!
//! Defines the stable C-ABI (`#[repr(C)]`) vtable contract for compiled dynamic library
//! widget plugins (`.dll` on Windows) exporting `extern "C" fn aether_widget_init()`.

use crate::lifecycle::{TickContext, WidgetState};
use std::ffi::{c_void, CStr};
use std::os::raw::c_char;

/// Current C-ABI protocol version enforced during dynamic library handshake.
pub const AETHER_CDYLIB_ABI_VERSION: u32 = 1;

/// C-ABI compatible tick context passed across `.dll` boundaries.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FfiTickContext {
    pub timestamp_ms: u64,
    pub delta_time_ms: f32,
    pub frame_index: u64,
}

impl From<TickContext> for FfiTickContext {
    fn from(ctx: TickContext) -> Self {
        Self {
            timestamp_ms: ctx.timestamp_ms,
            delta_time_ms: ctx.delta_time_ms,
            frame_index: ctx.frame_index,
        }
    }
}

/// Static metadata exported by a compiled `cdylib` widget plugin.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AetherPluginMetadata {
    pub abi_version: u32,
    pub id_utf8: *const c_char,
    pub name_utf8: *const c_char,
    pub version_utf8: *const c_char,
}

unsafe impl Send for AetherPluginMetadata {}
unsafe impl Sync for AetherPluginMetadata {}

/// C-ABI function pointer table exported by `aether_widget_init()`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AetherWidgetVTable {
    pub metadata: AetherPluginMetadata,
    pub create_instance: unsafe extern "C" fn() -> *mut c_void,
    pub destroy_instance: unsafe extern "C" fn(instance: *mut c_void),
    pub on_load: unsafe extern "C" fn(instance: *mut c_void) -> i32,
    pub on_mount: unsafe extern "C" fn(instance: *mut c_void) -> i32,
    pub on_update: unsafe extern "C" fn(instance: *mut c_void, ctx: FfiTickContext) -> i32,
    pub on_unmount: unsafe extern "C" fn(instance: *mut c_void) -> i32,
    pub on_unload: unsafe extern "C" fn(instance: *mut c_void) -> i32,
}

unsafe impl Send for AetherWidgetVTable {}
unsafe impl Sync for AetherWidgetVTable {}

/// Signature of the required `aether_widget_init` symbol exported by native Rust widget DLLs.
pub type AetherWidgetInitFn = unsafe extern "C" fn() -> *const AetherWidgetVTable;

/// Safe host wrapper around a loaded `AetherWidgetVTable` instance.
pub struct NativePluginHost {
    vtable: AetherWidgetVTable,
    instance: *mut c_void,
    state: WidgetState,
    id: String,
    name: String,
    version: String,
}

unsafe impl Send for NativePluginHost {}
unsafe impl Sync for NativePluginHost {}

impl NativePluginHost {
    /// Initializes a native plugin instance from a `aether_widget_init` entry point after verifying ABI compatibility.
    ///
    /// # Safety
    /// Caller must ensure `init_fn` points to a valid `AetherWidgetInitFn` returning a non-null `AetherWidgetVTable`.
    pub unsafe fn from_init_fn(init_fn: AetherWidgetInitFn) -> anyhow::Result<Self> {
        let vtable_ptr = init_fn();
        if vtable_ptr.is_null() {
            anyhow::bail!("aether_widget_init returned null vtable pointer");
        }
        let vtable = *vtable_ptr;
        if vtable.metadata.abi_version != AETHER_CDYLIB_ABI_VERSION {
            anyhow::bail!(
                "Incompatible cdylib ABI version: expected {}, got {}",
                AETHER_CDYLIB_ABI_VERSION,
                vtable.metadata.abi_version
            );
        }

        let id = Self::read_cstr(vtable.metadata.id_utf8, "unknown.plugin");
        let name = Self::read_cstr(vtable.metadata.name_utf8, "Unknown Native Plugin");
        let version = Self::read_cstr(vtable.metadata.version_utf8, "0.1.0");

        let instance = (vtable.create_instance)();
        if instance.is_null() {
            anyhow::bail!("Native plugin '{}' create_instance returned null", id);
        }

        Ok(Self {
            vtable,
            instance,
            state: WidgetState::Unloaded,
            id,
            name,
            version,
        })
    }

    unsafe fn read_cstr(ptr: *const c_char, fallback: &str) -> String {
        if ptr.is_null() {
            fallback.to_string()
        } else {
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn state(&self) -> WidgetState {
        self.state
    }

    pub fn load(&mut self) -> anyhow::Result<()> {
        let rc = unsafe { (self.vtable.on_load)(self.instance) };
        if rc != 0 {
            self.state = WidgetState::Error;
            anyhow::bail!("Native plugin '{}' on_load failed with code {}", self.id, rc);
        }
        self.state = WidgetState::Loaded;
        Ok(())
    }

    pub fn mount(&mut self) -> anyhow::Result<()> {
        let rc = unsafe { (self.vtable.on_mount)(self.instance) };
        if rc != 0 {
            self.state = WidgetState::Error;
            anyhow::bail!("Native plugin '{}' on_mount failed with code {}", self.id, rc);
        }
        self.state = WidgetState::Active;
        Ok(())
    }

    pub fn update(&mut self, ctx: TickContext) -> anyhow::Result<()> {
        let rc = unsafe { (self.vtable.on_update)(self.instance, ctx.into()) };
        if rc != 0 {
            self.state = WidgetState::Error;
            anyhow::bail!("Native plugin '{}' on_update failed with code {}", self.id, rc);
        }
        Ok(())
    }

    pub fn unmount(&mut self) -> anyhow::Result<()> {
        let rc = unsafe { (self.vtable.on_unmount)(self.instance) };
        if rc != 0 {
            self.state = WidgetState::Error;
            anyhow::bail!("Native plugin '{}' on_unmount failed with code {}", self.id, rc);
        }
        self.state = WidgetState::Unmounted;
        Ok(())
    }

    pub fn unload(&mut self) -> anyhow::Result<()> {
        let rc = unsafe { (self.vtable.on_unload)(self.instance) };
        if rc != 0 {
            self.state = WidgetState::Error;
            anyhow::bail!("Native plugin '{}' on_unload failed with code {}", self.id, rc);
        }
        self.state = WidgetState::Unloaded;
        Ok(())
    }
}

impl Drop for NativePluginHost {
    fn drop(&mut self) {
        if !self.instance.is_null() {
            unsafe {
                (self.vtable.destroy_instance)(self.instance);
            }
            self.instance = std::ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(C)]
    struct SampleCWidget {
        ticks: u64,
    }

    unsafe extern "C" fn sample_create() -> *mut c_void {
        Box::into_raw(Box::new(SampleCWidget { ticks: 0 })) as *mut c_void
    }

    unsafe extern "C" fn sample_destroy(ptr: *mut c_void) {
        if !ptr.is_null() {
            let _ = Box::from_raw(ptr as *mut SampleCWidget);
        }
    }

    unsafe extern "C" fn sample_ok(_ptr: *mut c_void) -> i32 {
        0
    }

    unsafe extern "C" fn sample_update(ptr: *mut c_void, _ctx: FfiTickContext) -> i32 {
        let w = &mut *(ptr as *mut SampleCWidget);
        w.ticks += 1;
        0
    }

    static SAMPLE_VTABLE: AetherWidgetVTable = AetherWidgetVTable {
        metadata: AetherPluginMetadata {
            abi_version: AETHER_CDYLIB_ABI_VERSION,
            id_utf8: b"com.aether.native_clock\0".as_ptr() as *const c_char,
            name_utf8: b"Native Clock\0".as_ptr() as *const c_char,
            version_utf8: b"1.0.0\0".as_ptr() as *const c_char,
        },
        create_instance: sample_create,
        destroy_instance: sample_destroy,
        on_load: sample_ok,
        on_mount: sample_ok,
        on_update: sample_update,
        on_unmount: sample_ok,
        on_unload: sample_ok,
    };

    unsafe extern "C" fn sample_aether_widget_init() -> *const AetherWidgetVTable {
        &SAMPLE_VTABLE
    }

    #[test]
    fn test_native_cdylib_plugin_host_lifecycle() {
        let mut host = unsafe { NativePluginHost::from_init_fn(sample_aether_widget_init) }.unwrap();
        assert_eq!(host.id(), "com.aether.native_clock");
        assert_eq!(host.name(), "Native Clock");
        assert_eq!(host.version(), "1.0.0");
        assert_eq!(host.state(), WidgetState::Unloaded);

        host.load().unwrap();
        assert_eq!(host.state(), WidgetState::Loaded);

        host.mount().unwrap();
        assert_eq!(host.state(), WidgetState::Active);

        host.update(TickContext {
            timestamp_ms: 1000,
            delta_time_ms: 16.6,
            frame_index: 1,
        })
        .unwrap();

        host.unmount().unwrap();
        assert_eq!(host.state(), WidgetState::Unmounted);

        host.unload().unwrap();
        assert_eq!(host.state(), WidgetState::Unloaded);
    }
}

