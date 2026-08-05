use crate::schema::{AnimationConfig, FontConfig, LayoutConfig, ThemeSchema};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tracing::info;

/// Abstract Theme Resolver Interface.
/// Enforces interface isolation so engine components consume abstractions.
pub trait ThemeResolver: Send + Sync {
    fn resolve_color(&self, token: &str) -> String;
    fn resolve_font(&self, token: &str) -> FontConfig;
    fn resolve_icon(&self, token: &str) -> String;
    fn resolve_widget_style(&self, widget_id: &str, property: &str) -> Option<String>;
    fn resolve_layout(&self, token: &str) -> LayoutConfig;
    fn resolve_animation(&self, token: &str) -> AnimationConfig;
    fn hot_swap_schema(&self, new_schema: ThemeSchema);
}

/// In-Memory Atomic Theme Store supporting microsecond token resolution and zero-downtime hot reloading.
#[derive(Clone)]
pub struct DynamicThemeStore {
    schema: Arc<RwLock<ThemeSchema>>,
}

impl DynamicThemeStore {
    pub fn new(initial_schema: ThemeSchema) -> Self {
        Self {
            schema: Arc::new(RwLock::new(initial_schema)),
        }
    }
}

impl Default for DynamicThemeStore {
    fn default() -> Self {
        Self::new(ThemeSchema::default())
    }
}

impl ThemeResolver for DynamicThemeStore {
    fn resolve_color(&self, token: &str) -> String {
        let lock = self.schema.read().unwrap();
        lock.colors
            .get(token)
            .cloned()
            .unwrap_or_else(|| "#FFFFFF".to_string())
    }

    fn resolve_font(&self, token: &str) -> FontConfig {
        let lock = self.schema.read().unwrap();
        lock.fonts
            .get(token)
            .cloned()
            .unwrap_or_default()
    }

    fn resolve_icon(&self, token: &str) -> String {
        let lock = self.schema.read().unwrap();
        lock.icons
            .get(token)
            .cloned()
            .unwrap_or_else(|| "assets/icons/default.svg".to_string())
    }

    fn resolve_widget_style(&self, widget_id: &str, property: &str) -> Option<String> {
        let lock = self.schema.read().unwrap();
        lock.widgets
            .get(widget_id)
            .and_then(|props| props.get(property).cloned())
    }

    fn resolve_layout(&self, token: &str) -> LayoutConfig {
        let lock = self.schema.read().unwrap();
        lock.layouts
            .get(token)
            .cloned()
            .unwrap_or_default()
    }

    fn resolve_animation(&self, token: &str) -> AnimationConfig {
        let lock = self.schema.read().unwrap();
        lock.animations
            .get(token)
            .cloned()
            .unwrap_or_default()
    }

    fn hot_swap_schema(&self, new_schema: ThemeSchema) {
        let name = new_schema.metadata.name.clone();
        if let Ok(mut lock) = self.schema.write() {
            *lock = new_schema;
            info!("Theme successfully hot-swapped to: '{}' (No restart required!)", name);
        }
    }
}

impl DynamicThemeStore {
    /// Queries the active Windows 11 system accent color via DwmGetColorizationColor
    /// and updates the `theme.accent` token in real time.
    pub fn sync_windows_system_accent(&self) -> bool {
        if let Some(accent_hex) = query_windows_accent_color() {
            if let Ok(mut lock) = self.schema.write() {
                lock.colors.insert("theme.accent".to_string(), accent_hex.clone());
                info!("Synced Windows 11 system accent color -> {}", accent_hex);
                return true;
            }
        }
        false
    }
}


/// Performance benchmark harness evaluating Theme Engine token resolution throughput.
pub struct ThemeBenchmark;

impl ThemeBenchmark {
    pub fn run_benchmark() {
        let store = DynamicThemeStore::default();
        let lookup_count = 100_000usize;

        let start = Instant::now();
        for _ in 0..lookup_count {
            let _c = store.resolve_color("theme.accent");
            let _f = store.resolve_font("default");
        }
        let elapsed = start.elapsed();

        let throughput_per_sec = (lookup_count as f64 * 2.0) / elapsed.as_secs_f64();
        info!(
            "Theme Engine Benchmark: Token Resolution Throughput = {:.0} lookups / sec ({:?})",
            throughput_per_sec, elapsed
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_resolver_and_hot_swap() {
        let store = DynamicThemeStore::default();
        assert_eq!(store.resolve_color("theme.accent"), "#0078D7");

        let mut new_schema = ThemeSchema::default();
        new_schema.colors.insert("theme.accent".into(), "#FF0000".into());

        store.hot_swap_schema(new_schema);
        assert_eq!(store.resolve_color("theme.accent"), "#FF0000");
    }

    #[test]
    fn test_theme_benchmark_execution() {
        ThemeBenchmark::run_benchmark();
    }

    #[test]
    fn test_windows_accent_color_query() {
        let _accent = query_windows_accent_color();
    }
}

/// Queries Windows 11 desktop accent color using DwmGetColorizationColor Win32 API.
#[cfg(windows)]
pub fn query_windows_accent_color() -> Option<String> {
    use windows::Win32::Graphics::Dwm::DwmGetColorizationColor;
    unsafe {
        let mut colorization: u32 = 0;
        let mut opaque_blend = windows::Win32::Foundation::BOOL(0);
        if DwmGetColorizationColor(&mut colorization, &mut opaque_blend).is_ok() {
            let r = (colorization >> 16) & 0xFF;
            let g = (colorization >> 8) & 0xFF;
            let b = colorization & 0xFF;
            return Some(format!("#{:02X}{:02X}{:02X}", r, g, b));
        }
    }
    None
}

#[cfg(not(windows))]
pub fn query_windows_accent_color() -> Option<String> {
    None
}
