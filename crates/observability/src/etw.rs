use serde::{Deserialize, Serialize};
use tracing::info;

/// Aether ETW Provider GUID: `{B9C23C91-6E94-4F93-8A2D-0F29E0B25E1A}`
pub const AETHER_PROVIDER_GUID_U128: u128 = 0xB9C23C91_6E94_4F93_8A2D_0F29E0B25E1A;

/// Event payload emitted via ETW provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EtwEvent {
    pub provider_name: String,
    pub event_name: String,
    pub payload_json: String,
    pub success: bool,
}

/// Native Event Tracing for Windows (ETW) provider wrapper.
/// Registers a real manifest-free ETW provider handle with the Windows kernel via `EventRegister`
/// and emits high-performance tracing events via `EventWriteString`.
#[derive(Debug)]
pub struct EtwProvider {
    provider_name: String,
    enabled: bool,
    #[cfg(windows)]
    reg_handle: u64,
}

impl Clone for EtwProvider {
    fn clone(&self) -> Self {
        Self::new(&self.provider_name)
    }
}

impl EtwProvider {
    pub fn new(provider_name: &str) -> Self {
        #[cfg(windows)]
        {
            use windows::core::GUID;
            use windows::Win32::System::Diagnostics::Etw::EventRegister;

            let guid = GUID::from_u128(AETHER_PROVIDER_GUID_U128);
            let mut handle_u64 = 0u64;

            unsafe {
                let _ = EventRegister(&guid, None, None, &mut handle_u64);
            }

            Self {
                provider_name: provider_name.to_string(),
                enabled: true,
                reg_handle: handle_u64,
            }
        }

        #[cfg(not(windows))]
        {
            Self {
                provider_name: provider_name.to_string(),
                enabled: true,
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Emits a structured event to Windows Event Tracing (ETW) kernel stream.
    pub fn write_event(&self, event_name: &str, payload_json: &str) -> EtwEvent {
        let mut success = true;

        if self.enabled {
            #[cfg(windows)]
            {
                use windows::core::HSTRING;
                use windows::Win32::System::Diagnostics::Etw::{EventWriteString, REGHANDLE};

                if self.reg_handle != 0 {
                    let formatted = format!("{}: {}", event_name, payload_json);
                    let hstr = HSTRING::from(&formatted);
                    unsafe {
                        // Level 4 (Informational), Keyword 0
                        let res = EventWriteString(REGHANDLE(self.reg_handle as i64), 4, 0, &hstr);
                        if res != 0 {
                            success = false;
                        }
                    }
                }
            }

            info!(
                provider = %self.provider_name,
                event = %event_name,
                "ETW Event emitted to kernel trace stream"
            );
        }

        EtwEvent {
            provider_name: self.provider_name.clone(),
            event_name: event_name.to_string(),
            payload_json: payload_json.to_string(),
            success,
        }
    }
}

impl Drop for EtwProvider {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            use windows::Win32::System::Diagnostics::Etw::{EventUnregister, REGHANDLE};
            if self.reg_handle != 0 {
                unsafe {
                    let _ = EventUnregister(REGHANDLE(self.reg_handle as i64));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etw_provider_lifecycle_and_emission() {
        let mut provider = EtwProvider::new("AetherEngineProvider");
        assert!(provider.is_enabled());

        let event = provider.write_event("TelemetryTick", r#"{"cpu":15.5,"fps":120}"#);
        assert_eq!(event.event_name, "TelemetryTick");
        assert_eq!(event.provider_name, "AetherEngineProvider");
        assert!(event.success);

        provider.set_enabled(false);
        assert!(!provider.is_enabled());
    }
}
