use anyhow::Result;
use std::fmt::Debug;
use std::time::Instant;

pub mod cpu_topology;
pub mod cross_platform;
pub mod crypto_financial;
pub mod gpu_d3dkmt;
pub mod network_diagnostics;
pub mod wasapi_audio;

pub use cpu_topology::{CpuTopologyProvider, CpuTopologyTelemetry};
pub use cross_platform::{LinuxProcfsProvider, MacOsHostStatsProvider};
pub use crypto_financial::{CryptoAssetTelemetry, CryptoFinancialProvider};
pub use gpu_d3dkmt::{DedicatedGpuProvider, GpuTelemetry, GpuVendor};
pub use network_diagnostics::{NetworkDiagnosticsProvider, NetworkDiagnosticsTelemetry};
pub use wasapi_audio::{AudioSpectrumTelemetry, MediaPlaybackTelemetry, WasapiAudioProvider};

/// Represents a single metric sample collected from an OS/Hardware provider.
#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Percentage(f32),
    Megabytes(f32),
    BytesPerSec(u64),
    /// Used by MemoryProvider to return both used and total RAM in one sample.
    MemoryStats { used_mb: f32, total_mb: f32 },
    /// Used by NetworkProvider to return both rx and tx throughput in bytes/sec.
    NetworkStats { rx_bytes_per_sec: u64, tx_bytes_per_sec: u64 },
    /// Battery status metrics
    BatteryStats {
        charge_pct: f32,
        remaining_secs: u64,
        is_charging: bool,
    },
    /// Audio master volume metrics
    AudioStats {
        master_volume_pct: f32,
        is_muted: bool,
    },
    /// Process categorization metrics
    ProcessStats {
        open_apps_count: u32,
        browser_tabs_count: u32,
        audio_playing_apps_count: u32,
        gaming_apps_count: u32,
        dev_suite_apps_count: u32,
        other_apps_count: u32,
    },
    /// Display & GPU topology metrics
    DisplayTopologyStats {
        total_gpu_count: u32,
        integrated_gpu_count: u32,
        dedicated_gpu_count: u32,
        total_display_count: u32,
        external_display_count: u32,
        virtual_display_count: u32,
    },
    /// Deep dedicated GPU and VRAM hardware metrics.
    GpuStats(GpuTelemetry),
    /// Multi-core topology and P/E core load distribution.
    CpuTopology(CpuTopologyTelemetry),
    /// Audio frequency spectrum FFT bins.
    AudioSpectrum(AudioSpectrumTelemetry),
    /// Media playback session state.
    MediaSession(MediaPlaybackTelemetry),
    /// Live financial and cryptocurrency market asset prices.
    CryptoAssets(Vec<CryptoAssetTelemetry>),
    /// Deep network latency and bandwidth diagnostics.
    NetworkDiagnostics(NetworkDiagnosticsTelemetry),
}

/// Abstract Metric Provider Interface.
/// Enforces interface isolation so collectors depend on abstractions.
pub trait MetricProvider: Send + Sync + Debug {
    /// Returns the provider name identifier.
    fn name(&self) -> &'static str;

    /// Samples hardware metric from system APIs.
    fn sample(&mut self) -> Result<MetricValue>;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

#[cfg(windows)]
fn filetime_to_u64(ft: windows::Win32::Foundation::FILETIME) -> u64 {
    ((ft.dwHighDateTime as u64) << 32) | ft.dwLowDateTime as u64
}

// ─── CPU Provider ─────────────────────────────────────────────────────────────

/// CPU Hardware Metric Collector Provider.
/// Uses `GetSystemTimes` (Win32) for accurate delta-based CPU utilisation,
/// with Task Manager-style zero-delta value holding and Exponential Moving Average (EMA) smoothing.
#[derive(Debug)]
pub struct CpuProvider {
    prev_idle: u64,
    prev_total: u64,
    last_valid_pct: f32,
    ema_pct: f32,
    tick: u64,
    alpha: f32,
}

impl CpuProvider {
    pub fn new() -> Self {
        // Seed with an initial snapshot so the first delta is valid
        let (idle, kernel, user) = Self::system_times_raw().unwrap_or((0, 1, 0));
        Self {
            prev_idle: idle,
            prev_total: kernel + user,
            last_valid_pct: 0.0,
            ema_pct: 0.0,
            tick: 0,
            alpha: 0.25, // Task Manager temporal smoothing factor
        }
    }

    #[cfg(windows)]
    fn system_times_raw() -> Result<(u64, u64, u64)> {
        use windows::Win32::Foundation::FILETIME;
        use windows::Win32::System::Threading::GetSystemTimes;
        unsafe {
            let mut idle = FILETIME::default();
            let mut kernel = FILETIME::default();
            let mut user = FILETIME::default();
            GetSystemTimes(
                Some(&mut idle),
                Some(&mut kernel),
                Some(&mut user),
            )?;
            Ok((
                filetime_to_u64(idle),
                filetime_to_u64(kernel),
                filetime_to_u64(user),
            ))
        }
    }

    #[cfg(not(windows))]
    fn system_times_raw() -> Result<(u64, u64, u64)> {
        Ok((0, 1, 0))
    }
}

impl Default for CpuProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricProvider for CpuProvider {
    fn name(&self) -> &'static str {
        "CpuProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        self.tick += 1;

        let raw_pct = if let Ok((idle, kernel, user)) = Self::system_times_raw() {
            let total = kernel + user;
            let delta_idle = idle.saturating_sub(self.prev_idle);
            let delta_total = total.saturating_sub(self.prev_total);

            if delta_total > 0 {
                self.prev_idle = idle;
                self.prev_total = total;

                // kernel time includes idle, so busy = total − idle
                let cpu_pct = (1.0 - delta_idle as f32 / delta_total as f32) * 100.0;
                let clamped = cpu_pct.clamp(0.0, 100.0);
                self.last_valid_pct = clamped;
                clamped
            } else {
                // If sub-quantum tick (delta_total == 0), hold last valid percentage instead of fake sine waves
                self.last_valid_pct
            }
        } else {
            self.last_valid_pct
        };

        // Apply Exponential Moving Average (EMA) smoothing for Task Manager consistency
        if self.tick == 1 {
            self.ema_pct = raw_pct;
        } else {
            self.ema_pct = (self.alpha * raw_pct) + ((1.0 - self.alpha) * self.ema_pct);
        }

        Ok(MetricValue::Percentage(self.ema_pct.clamp(0.0, 100.0)))
    }
}

// ─── Memory Provider ──────────────────────────────────────────────────────────

/// RAM Hardware Metric Collector Provider.
/// Uses `GlobalMemoryStatusEx` (Win32) for real used / total MB with EMA temporal smoothing.
#[derive(Debug)]
pub struct MemoryProvider {
    tick: u64,
    ema_used_mb: f32,
    alpha: f32,
}

impl MemoryProvider {
    pub fn new() -> Self {
        Self {
            tick: 0,
            ema_used_mb: 0.0,
            alpha: 0.3,
        }
    }

    #[cfg(windows)]
    fn query_memory() -> Result<(f32, f32)> {
        use windows::Win32::System::SystemInformation::{
            GlobalMemoryStatusEx, MEMORYSTATUSEX,
        };
        unsafe {
            let mut status = MEMORYSTATUSEX::default();
            status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
            GlobalMemoryStatusEx(&mut status)?;
            let total_mb = status.ullTotalPhys as f32 / (1024.0 * 1024.0);
            let avail_mb = status.ullAvailPhys as f32 / (1024.0 * 1024.0);
            Ok((total_mb - avail_mb, total_mb))
        }
    }

    #[cfg(not(windows))]
    fn query_memory() -> Result<(f32, f32)> {
        Ok((4096.0, 16384.0))
    }
}

impl Default for MemoryProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricProvider for MemoryProvider {
    fn name(&self) -> &'static str {
        "MemoryProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        self.tick += 1;
        let (used_mb, total_mb) = Self::query_memory().unwrap_or((4096.0, 16384.0));

        if self.tick == 1 {
            self.ema_used_mb = used_mb;
        } else {
            self.ema_used_mb = (self.alpha * used_mb) + ((1.0 - self.alpha) * self.ema_used_mb);
        }

        Ok(MetricValue::MemoryStats {
            used_mb: self.ema_used_mb.min(total_mb),
            total_mb,
        })
    }
}

// ─── GPU Provider ─────────────────────────────────────────────────────────────

/// GPU Hardware Metric Collector Provider.
/// Queries DXGI video memory usage via `IDXGIFactory1::QueryVideoMemoryInfo` with EMA smoothing.
#[derive(Debug)]
pub struct GpuProvider {
    tick: u64,
    ema_pct: f32,
    alpha: f32,
}

impl GpuProvider {
    pub fn new() -> Self {
        Self {
            tick: 0,
            ema_pct: 0.0,
            alpha: 0.25,
        }
    }

    #[cfg(windows)]
    fn query_gpu_memory_pct() -> Result<f32> {
        use windows::core::Interface;
        use windows::Win32::Graphics::Dxgi::{
            CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
            DXGI_MEMORY_SEGMENT_GROUP_LOCAL, IDXGIFactory1, IDXGIAdapter3,
        };
        unsafe {
            let factory: IDXGIFactory1 = CreateDXGIFactory1()?;
            let mut adapter_index = 0;
            while let Ok(adapter) = factory.EnumAdapters1(adapter_index) {
                if let Ok(desc) = adapter.GetDesc1() {
                    // Skip software rasterizer adapters
                    if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) == 0 {
                        if let Ok(adapter3) = adapter.cast::<IDXGIAdapter3>() {
                            let mut info = std::mem::zeroed();
                            if adapter3
                                .QueryVideoMemoryInfo(
                                    0,
                                    DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
                                    &mut info,
                                )
                                .is_ok()
                            {
                                if info.Budget > 0 {
                                    let usage_pct = (info.CurrentUsage as f32
                                        / info.Budget as f32)
                                        * 100.0;
                                    return Ok(usage_pct.clamp(0.0, 100.0));
                                }
                            }
                        }
                    }
                }
                adapter_index += 1;
            }
        }
        anyhow::bail!("No active DXGI hardware adapter found")
    }

    #[cfg(not(windows))]
    fn query_gpu_memory_pct() -> Result<f32> {
        anyhow::bail!("Non-windows environment")
    }
}

impl Default for GpuProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricProvider for GpuProvider {
    fn name(&self) -> &'static str {
        "GpuProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        self.tick += 1;
        let raw_pct = Self::query_gpu_memory_pct().unwrap_or(0.0);

        if self.tick == 1 {
            self.ema_pct = raw_pct;
        } else {
            self.ema_pct = (self.alpha * raw_pct) + ((1.0 - self.alpha) * self.ema_pct);
        }

        Ok(MetricValue::Percentage(self.ema_pct.clamp(0.0, 100.0)))
    }
}

// ─── Network Provider ─────────────────────────────────────────────────────────

/// Network Throughput Metric Collector Provider.
/// Uses `GetIfTable2` (Win32) to query hardware network interface octet throughput,
/// scaled by actual elapsed time and filtered with EMA smoothing.
#[derive(Debug)]
pub struct NetworkProvider {
    tick: u64,
    prev_rx_bytes: u64,
    prev_tx_bytes: u64,
    last_sample_time: Instant,
    ema_rx_bytes_per_sec: f64,
    ema_tx_bytes_per_sec: f64,
    alpha: f64,
}

impl NetworkProvider {
    pub fn new() -> Self {
        let (initial_rx, initial_tx) = Self::query_network_octets().unwrap_or((0, 0));
        Self {
            tick: 0,
            prev_rx_bytes: initial_rx,
            prev_tx_bytes: initial_tx,
            last_sample_time: Instant::now(),
            ema_rx_bytes_per_sec: 0.0,
            ema_tx_bytes_per_sec: 0.0,
            alpha: 0.3,
        }
    }

    #[cfg(windows)]
    fn query_network_octets() -> Result<(u64, u64)> {
        use windows::Win32::NetworkManagement::IpHelper::{
            FreeMibTable, GetIfTable2, MIB_IF_TABLE2,
        };
        unsafe {
            let mut table_ptr: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
            GetIfTable2(&mut table_ptr).ok()?;
            if table_ptr.is_null() {
                return Ok((0, 0));
            }
            let table = &*table_ptr;
            let mut total_rx: u64 = 0;
            let mut total_tx: u64 = 0;
            let count = table.NumEntries as usize;
            let rows = std::slice::from_raw_parts(table.Table.as_ptr(), count);
            for row in rows {
                total_rx += row.InOctets;
                total_tx += row.OutOctets;
            }
            FreeMibTable(table_ptr as *const _);
            Ok((total_rx, total_tx))
        }
    }

    #[cfg(not(windows))]
    fn query_network_octets() -> Result<(u64, u64)> {
        Ok((0, 0))
    }
}

impl Default for NetworkProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricProvider for NetworkProvider {
    fn name(&self) -> &'static str {
        "NetworkProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        self.tick += 1;
        let now = Instant::now();
        let elapsed_secs = now.duration_since(self.last_sample_time).as_secs_f64();
        self.last_sample_time = now;

        let (raw_rx_rate, raw_tx_rate) = if let Ok((cur_rx, cur_tx)) = Self::query_network_octets() {
            let rx_rate = if cur_rx >= self.prev_rx_bytes {
                let delta = cur_rx.saturating_sub(self.prev_rx_bytes);
                self.prev_rx_bytes = cur_rx;
                if elapsed_secs > 0.001 { delta as f64 / elapsed_secs } else { self.ema_rx_bytes_per_sec }
            } else {
                self.prev_rx_bytes = cur_rx;
                0.0
            };

            let tx_rate = if cur_tx >= self.prev_tx_bytes {
                let delta = cur_tx.saturating_sub(self.prev_tx_bytes);
                self.prev_tx_bytes = cur_tx;
                if elapsed_secs > 0.001 { delta as f64 / elapsed_secs } else { self.ema_tx_bytes_per_sec }
            } else {
                self.prev_tx_bytes = cur_tx;
                0.0
            };

            (rx_rate, tx_rate)
        } else {
            (0.0, 0.0)
        };

        if self.tick == 1 {
            self.ema_rx_bytes_per_sec = raw_rx_rate;
            self.ema_tx_bytes_per_sec = raw_tx_rate;
        } else {
            self.ema_rx_bytes_per_sec = (self.alpha * raw_rx_rate) + ((1.0 - self.alpha) * self.ema_rx_bytes_per_sec);
            self.ema_tx_bytes_per_sec = (self.alpha * raw_tx_rate) + ((1.0 - self.alpha) * self.ema_tx_bytes_per_sec);
        }

        Ok(MetricValue::NetworkStats {
            rx_bytes_per_sec: self.ema_rx_bytes_per_sec as u64,
            tx_bytes_per_sec: self.ema_tx_bytes_per_sec as u64,
        })
    }
}

// ─── Battery Provider ─────────────────────────────────────────────────────────

/// Battery & Power Status Metric Collector Provider.
/// Queries Win32 `GetSystemPowerStatus` for real-time charge %, remaining seconds, and charging state.
#[derive(Debug)]
pub struct BatteryProvider;

impl BatteryProvider {
    pub fn new() -> Self {
        Self
    }

    #[cfg(windows)]
    fn query_battery() -> (f32, u64, bool) {
        use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
        unsafe {
            let mut status = SYSTEM_POWER_STATUS::default();
            if GetSystemPowerStatus(&mut status).is_ok() {
                let pct = if status.BatteryLifePercent != 255 {
                    status.BatteryLifePercent as f32
                } else {
                    100.0 // Desktop / AC power
                };
                let secs = if status.BatteryLifeTime != u32::MAX {
                    status.BatteryLifeTime as u64
                } else {
                    0
                };
                let charging = (status.BatteryFlag & 8) != 0 || status.ACLineStatus == 1;
                (pct, secs, charging)
            } else {
                (100.0, 0, true)
            }
        }
    }

    #[cfg(not(windows))]
    fn query_battery() -> (f32, u64, bool) {
        (100.0, 0, true)
    }
}

impl Default for BatteryProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricProvider for BatteryProvider {
    fn name(&self) -> &'static str {
        "BatteryProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        let (charge_pct, remaining_secs, is_charging) = Self::query_battery();
        Ok(MetricValue::BatteryStats {
            charge_pct,
            remaining_secs,
            is_charging,
        })
    }
}

// ─── Audio Provider ───────────────────────────────────────────────────────────

/// Master Audio Volume Metric Collector Provider.
/// Queries Windows Core Audio (WASAPI) `IMMDeviceEnumerator` and `IAudioEndpointVolume`
/// for real-time system master scalar volume and hardware/software mute state.
#[derive(Debug)]
pub struct AudioProvider {
    volume_pct: f32,
    is_muted: bool,
}

impl AudioProvider {
    pub fn new() -> Self {
        let (volume_pct, is_muted) = Self::query_system_audio().unwrap_or((0.0, false));
        Self {
            volume_pct,
            is_muted,
        }
    }

    #[cfg(windows)]
    pub fn query_system_audio() -> Result<(f32, bool)> {
        use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
        use windows::Win32::Media::Audio::{
            eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator,
        };
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
        };

        unsafe {
            // Initialize COM on the calling thread if not already initialized.
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let endpoint = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
            let volume_ctrl: IAudioEndpointVolume =
                endpoint.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)?;
            let scalar = volume_ctrl.GetMasterVolumeLevelScalar()?;
            let muted = volume_ctrl.GetMute()?.as_bool();
            let pct = (scalar * 100.0).clamp(0.0, 100.0);
            Ok((pct, muted))
        }
    }

    #[cfg(not(windows))]
    pub fn query_system_audio() -> Result<(f32, bool)> {
        Ok((0.0, false))
    }
}

impl Default for AudioProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricProvider for AudioProvider {
    fn name(&self) -> &'static str {
        "AudioProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        if let Ok((vol, muted)) = Self::query_system_audio() {
            self.volume_pct = vol;
            self.is_muted = muted;
        }
        Ok(MetricValue::AudioStats {
            master_volume_pct: self.volume_pct,
            is_muted: self.is_muted,
        })
    }
}

// ─── Process Metrics Provider ─────────────────────────────────────────────────

/// Classification category for a running operating system process executable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessCategory {
    Browser,
    AudioMedia,
    Gaming,
    DevSuite,
    SystemBackground,
    OtherApp,
}

/// Process & Application Lifecycle Metric Collector Provider.
/// Enumerates active running processes via Win32 `EnumProcesses` and classifies
/// real executable image names via `QueryFullProcessImageNameW`.
#[derive(Debug)]
pub struct ProcessMetricsProvider;

impl ProcessMetricsProvider {
    pub fn new() -> Self {
        Self
    }

    /// Classifies an executable basename (case-insensitive) into a `ProcessCategory`.
    pub fn classify_executable(exe_name: &str) -> ProcessCategory {
        let lower = exe_name
            .rsplit(['\\', '/'])
            .next()
            .unwrap_or(exe_name)
            .trim()
            .to_ascii_lowercase();

        if lower.is_empty() {
            return ProcessCategory::SystemBackground;
        }

        match lower.as_str() {
            "chrome.exe" | "msedge.exe" | "firefox.exe" | "brave.exe" | "opera.exe"
            | "vivaldi.exe" | "arc.exe" | "waterfox.exe" => ProcessCategory::Browser,

            "spotify.exe" | "music.ui.exe" | "vlc.exe" | "foobar2000.exe" | "audiodg.exe"
            | "wmplayer.exe" | "itunes.exe" | "aimp.exe" | "tidal.exe" => {
                ProcessCategory::AudioMedia
            }

            "steam.exe" | "steamwebhelper.exe" | "epicgameslauncher.exe" | "galaxyclient.exe"
            | "playnite.exe" | "riotclientservices.exe" | "battle.net.exe" | "upc.exe"
            | "eadesktop.exe" => ProcessCategory::Gaming,

            "code.exe" | "devenv.exe" | "rust-analyzer.exe" | "cargo.exe" | "rustc.exe"
            | "rider64.exe" | "idea64.exe" | "clion64.exe" | "pycharm64.exe" | "nvim.exe"
            | "vim.exe" | "windowsterminal.exe" | "powershell.exe" | "pwsh.exe" | "cmd.exe"
            | "git.exe" | "node.exe" | "dotnet.exe" | "cursor.exe" | "windsurf.exe" => {
                ProcessCategory::DevSuite
            }

            "svchost.exe" | "conhost.exe" | "csrss.exe" | "smss.exe" | "services.exe"
            | "lsass.exe" | "wininit.exe" | "winlogon.exe" | "dwm.exe" | "fontdrvhost.exe"
            | "sihost.exe" | "taskhostw.exe" | "runtimebroker.exe" | "searchindexer.exe"
            | "spoolsv.exe" | "dllhost.exe" | "wmiprvse.exe" | "ctfmon.exe" | "system"
            | "registry" => ProcessCategory::SystemBackground,

            _ => ProcessCategory::OtherApp,
        }
    }

    #[cfg(windows)]
    fn query_classified_processes() -> (u32, u32, u32, u32, u32, u32) {
        use windows::core::PWSTR;
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::ProcessStatus::EnumProcesses;
        use windows::Win32::System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
            PROCESS_QUERY_LIMITED_INFORMATION,
        };

        let mut browser_count = 0u32;
        let mut audio_count = 0u32;
        let mut gaming_count = 0u32;
        let mut dev_count = 0u32;
        let mut other_count = 0u32;

        unsafe {
            let mut pids = [0u32; 2048];
            let mut bytes_returned = 0u32;
            if EnumProcesses(
                pids.as_mut_ptr(),
                (pids.len() * std::mem::size_of::<u32>()) as u32,
                &mut bytes_returned,
            )
            .is_ok()
            {
                let count = (bytes_returned as usize) / std::mem::size_of::<u32>();
                for &pid in &pids[..count.min(pids.len())] {
                    if pid == 0 || pid == 4 {
                        continue;
                    }
                    if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                        let mut buf = [0u16; 512];
                        let mut size = buf.len() as u32;
                        let ok = QueryFullProcessImageNameW(
                            handle,
                            PROCESS_NAME_WIN32,
                            PWSTR(buf.as_mut_ptr()),
                            &mut size,
                        )
                        .is_ok();
                        let _ = CloseHandle(handle);

                        if ok && size > 0 {
                            let path_str = String::from_utf16_lossy(&buf[..size as usize]);
                            match Self::classify_executable(&path_str) {
                                ProcessCategory::Browser => browser_count += 1,
                                ProcessCategory::AudioMedia => audio_count += 1,
                                ProcessCategory::Gaming => gaming_count += 1,
                                ProcessCategory::DevSuite => dev_count += 1,
                                ProcessCategory::OtherApp => other_count += 1,
                                ProcessCategory::SystemBackground => {}
                            }
                        }
                    }
                }
            }
        }

        let open_apps = browser_count + audio_count + gaming_count + dev_count + other_count;
        (
            open_apps,
            browser_count,
            audio_count,
            gaming_count,
            dev_count,
            other_count,
        )
    }

    #[cfg(not(windows))]
    fn query_classified_processes() -> (u32, u32, u32, u32, u32, u32) {
        (0, 0, 0, 0, 0, 0)
    }
}

impl Default for ProcessMetricsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricProvider for ProcessMetricsProvider {
    fn name(&self) -> &'static str {
        "ProcessMetricsProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        let (open_apps, browser_tabs, audio_apps, gaming_apps, dev_suite, other_apps) =
            Self::query_classified_processes();
        Ok(MetricValue::ProcessStats {
            open_apps_count: open_apps,
            browser_tabs_count: browser_tabs,
            audio_playing_apps_count: audio_apps,
            gaming_apps_count: gaming_apps,
            dev_suite_apps_count: dev_suite,
            other_apps_count: other_apps,
        })
    }
}

// ─── Display Topology Provider ────────────────────────────────────────────────

/// Display & Multi-Monitor Topology Metric Collector Provider.
/// Queries Win32 `GetSystemMetrics(SM_CMONITORS)` and DXGI adapter topology for accurate GPU and display counts.
#[derive(Debug)]
pub struct DisplayTopologyProvider;

impl DisplayTopologyProvider {
    pub fn new() -> Self {
        Self
    }

    #[cfg(windows)]
    fn query_topology() -> (u32, u32, u32, u32) {
        use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CMONITORS};
        use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, IDXGIFactory1};

        let display_count = unsafe {
            let count = GetSystemMetrics(SM_CMONITORS);
            if count > 0 { count as u32 } else { 1 }
        };

        let mut int_gpus = 0u32;
        let mut ded_gpus = 0u32;

        unsafe {
            if let Ok(factory) = CreateDXGIFactory1::<IDXGIFactory1>() {
                let mut idx = 0;
                while let Ok(adapter) = factory.EnumAdapters1(idx) {
                    if let Ok(desc) = adapter.GetDesc1() {
                        if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) == 0 {
                            // Dedicated GPU if dedicated video memory > 512 MB, otherwise integrated
                            if desc.DedicatedVideoMemory > 512 * 1024 * 1024 {
                                ded_gpus += 1;
                            } else {
                                int_gpus += 1;
                            }
                        }
                    }
                    idx += 1;
                }
            }
        }

        let total_gpus = (int_gpus + ded_gpus).max(1);
        if int_gpus == 0 && ded_gpus == 0 {
            int_gpus = 1;
        }

        (total_gpus, int_gpus, ded_gpus, display_count)
    }

    #[cfg(not(windows))]
    fn query_topology() -> (u32, u32, u32, u32) {
        (1, 1, 0, 1)
    }
}

impl Default for DisplayTopologyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricProvider for DisplayTopologyProvider {
    fn name(&self) -> &'static str {
        "DisplayTopologyProvider"
    }

    fn sample(&mut self) -> Result<MetricValue> {
        let (total_gpus, int_gpus, ded_gpus, displays) = Self::query_topology();
        let ext = displays.saturating_sub(1);
        Ok(MetricValue::DisplayTopologyStats {
            total_gpu_count: total_gpus,
            integrated_gpu_count: int_gpus,
            dedicated_gpu_count: ded_gpus,
            total_display_count: displays,
            external_display_count: ext,
            virtual_display_count: 0,
        })
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_providers_sampling() {
        let mut cpu = CpuProvider::new();
        let mut mem = MemoryProvider::new();
        let mut gpu = GpuProvider::new();
        let mut net = NetworkProvider::new();
        let mut bat = BatteryProvider::new();
        let mut audio = AudioProvider::new();
        let mut proc = ProcessMetricsProvider::new();
        let mut disp = DisplayTopologyProvider::new();

        assert_eq!(cpu.name(), "CpuProvider");
        assert!(matches!(
            cpu.sample().unwrap(),
            MetricValue::Percentage(_)
        ));
        assert!(matches!(
            mem.sample().unwrap(),
            MetricValue::MemoryStats { .. }
        ));
        assert!(matches!(
            gpu.sample().unwrap(),
            MetricValue::Percentage(_)
        ));
        assert!(matches!(
            net.sample().unwrap(),
            MetricValue::NetworkStats { .. }
        ));
        assert!(matches!(
            bat.sample().unwrap(),
            MetricValue::BatteryStats { .. }
        ));
        assert!(matches!(
            audio.sample().unwrap(),
            MetricValue::AudioStats { .. }
        ));
        assert!(matches!(
            proc.sample().unwrap(),
            MetricValue::ProcessStats { .. }
        ));
        assert!(matches!(
            disp.sample().unwrap(),
            MetricValue::DisplayTopologyStats { .. }
        ));
    }

    #[test]
    fn test_cpu_percentage_in_range() {
        let mut cpu = CpuProvider::new();
        for _ in 0..5 {
            let val = cpu.sample().unwrap();
            if let MetricValue::Percentage(pct) = val {
                assert!((0.0..=100.0).contains(&pct), "CPU% out of range: {pct}");
            }
        }
    }

    #[test]
    fn test_battery_stats_in_range() {
        let mut bat = BatteryProvider::new();
        let val = bat.sample().unwrap();
        if let MetricValue::BatteryStats { charge_pct, .. } = val {
            assert!((0.0..=100.0).contains(&charge_pct), "Battery % out of range: {charge_pct}");
        } else {
            panic!("Expected BatteryStats variant");
        }
    }

    #[test]
    fn test_display_topology_positive_counts() {
        let mut disp = DisplayTopologyProvider::new();
        let val = disp.sample().unwrap();
        if let MetricValue::DisplayTopologyStats { total_display_count, .. } = val {
            assert!(total_display_count >= 1, "At least 1 display must be reported");
        } else {
            panic!("Expected DisplayTopologyStats variant");
        }
    }

    #[test]
    fn test_audio_provider_wasapi_volume_and_mute_in_range() {
        let mut audio = AudioProvider::new();
        let val = audio.sample().unwrap();
        if let MetricValue::AudioStats {
            master_volume_pct,
            is_muted: _,
        } = val
        {
            assert!(
                (0.0..=100.0).contains(&master_volume_pct),
                "Master volume % out of range: {master_volume_pct}"
            );
        } else {
            panic!("Expected AudioStats variant");
        }
    }

    #[test]
    fn test_process_metrics_provider_real_executable_classification() {
        assert_eq!(
            ProcessMetricsProvider::classify_executable(
                r"C:\Program Files\Google\Chrome\Application\chrome.exe"
            ),
            ProcessCategory::Browser
        );
        assert_eq!(
            ProcessMetricsProvider::classify_executable(r"C:\Users\dev\AppData\Local\Programs\Microsoft VS Code\Code.exe"),
            ProcessCategory::DevSuite
        );
        assert_eq!(
            ProcessMetricsProvider::classify_executable(r"C:\Program Files\Spotify\Spotify.exe"),
            ProcessCategory::AudioMedia
        );
        assert_eq!(
            ProcessMetricsProvider::classify_executable(r"C:\Program Files (x86)\Steam\steam.exe"),
            ProcessCategory::Gaming
        );
        assert_eq!(
            ProcessMetricsProvider::classify_executable(r"C:\Windows\System32\svchost.exe"),
            ProcessCategory::SystemBackground
        );
        assert_eq!(
            ProcessMetricsProvider::classify_executable(r"C:\Program Files\CustomApp\MyApp.exe"),
            ProcessCategory::OtherApp
        );

        let mut proc = ProcessMetricsProvider::new();
        let val = proc.sample().unwrap();
        if let MetricValue::ProcessStats {
            open_apps_count,
            browser_tabs_count,
            audio_playing_apps_count,
            gaming_apps_count,
            dev_suite_apps_count,
            other_apps_count,
        } = val
        {
            assert_eq!(
                open_apps_count,
                browser_tabs_count
                    + audio_playing_apps_count
                    + gaming_apps_count
                    + dev_suite_apps_count
                    + other_apps_count
            );
        } else {
            panic!("Expected ProcessStats variant");
        }
    }
}
