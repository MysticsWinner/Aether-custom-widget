//! Real-Time Financial & Cryptocurrency Ticker Matrix Widget
//!
//! Renders live cryptocurrency prices (BTC, ETH, SOL), equity indices, RSI technical indicators,
//! and historical spline area charts with interactive tab selection.

use anyhow::Result;
use system_providers::{CryptoAssetTelemetry, SharedTelemetryCache};
use theme_engine::{MaterialSpec, MaterialType};
use tracing::info;
use widget_sdk::events::{HitTarget, HitTestTree, PointerEvent};
use widget_sdk::lifecycle::{TickContext, WidgetLifecycle, WidgetState};
use widget_sdk::perf_budget::PerformanceBudget;
use widget_sdk::rendering::{BatchRenderCanvas, Color, RectF, RenderCanvas, RenderEffect};

/// Financial & Cryptocurrency Ticker Widget.
pub struct CryptoStocksWidget {
    state: WidgetState,
    cache: SharedTelemetryCache,
    selected_symbol: String,
    hit_tree: HitTestTree,
    material: MaterialSpec,
    budget: PerformanceBudget,
    tick_count: u64,
}

impl CryptoStocksWidget {
    /// Creates a new `CryptoStocksWidget`.
    pub fn new(cache: SharedTelemetryCache) -> Self {
        let mut hit_tree = HitTestTree::new();
        let symbols = ["BTC", "ETH", "SOL", "SPX"];
        for (i, sym) in symbols.iter().enumerate() {
            let x = 25.0 + (i as f32 * 100.0);
            hit_tree.register_target(HitTarget::new(*sym, RectF::new(x, 15.0, 90.0, 35.0)));
        }

        Self {
            state: WidgetState::Unloaded,
            cache,
            selected_symbol: "BTC".to_string(),
            hit_tree,
            material: MaterialSpec {
                material_type: MaterialType::Mica,
                tint_color: "#0F172A".to_string(),
                tint_opacity: 0.90,
                blur_radius: 35.0,
                ..Default::default()
            },
            budget: PerformanceBudget {
                target_cpu_pct: 0.05,
                target_memory_mb: 18.0,
                target_fps: 60,
                material_cost: "medium".to_string(),
                animation_cost: "low".to_string(),
            },
            tick_count: 0,
        }
    }

    /// Handles pointer interaction for asset tab switching.
    pub fn handle_pointer_event(&mut self, event: PointerEvent) -> Option<String> {
        match event {
            PointerEvent::Down { x, y, .. } => {
                if let Some(target) = self.hit_tree.hit_test(x, y) {
                    let sym = target.element_id.clone();
                    info!("Switched active crypto asset to: {}", sym);
                    self.selected_symbol = sym.clone();
                    Some(sym)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Returns currently selected asset symbol.
    pub fn selected_symbol(&self) -> &str {
        &self.selected_symbol
    }
}

impl WidgetLifecycle for CryptoStocksWidget {
    fn on_load(&mut self) -> Result<()> {
        self.state = WidgetState::Loaded;
        info!("CryptoStocksWidget loaded.");
        Ok(())
    }

    fn on_mount(&mut self) -> Result<()> {
        self.state = WidgetState::Mounted;
        info!("CryptoStocksWidget mounted.");
        Ok(())
    }

    fn on_update(&mut self, _ctx: &TickContext) -> Result<()> {
        self.tick_count += 1;

        let assets = self.cache.get_crypto_assets();
        let active_asset = assets
            .iter()
            .find(|a| a.symbol == self.selected_symbol)
            .cloned()
            .unwrap_or_else(|| {
                if !assets.is_empty() {
                    assets[0].clone()
                } else {
                    CryptoAssetTelemetry::default()
                }
            });

        let mut canvas = BatchRenderCanvas::new();

        // 1. Background Glass
        canvas.draw_effect(
            RenderEffect::GaussianBlur { radius: 25.0 },
            RectF::new(0.0, 0.0, 460.0, 260.0),
        );
        canvas.draw_rect(
            RectF::new(0.0, 0.0, 460.0, 260.0),
            Color::rgba(0.06, 0.09, 0.16, 0.92),
            20.0,
        );

        // 2. Asset Tabs
        let symbols = ["BTC", "ETH", "SOL", "SPX"];
        for (i, sym) in symbols.iter().enumerate() {
            let x = 25.0 + (i as f32 * 105.0);
            let is_active = *sym == self.selected_symbol;
            let tab_bg = if is_active {
                Color::rgba(0.15, 0.23, 0.42, 0.90)
            } else {
                Color::rgba(0.10, 0.14, 0.24, 0.50)
            };
            canvas.draw_rect(RectF::new(x, 15.0, 95.0, 32.0), tab_bg, 8.0);
            canvas.draw_text(
                *sym,
                "Segoe UI Variable Text",
                12.0,
                RectF::new(x + 12.0, 22.0, 70.0, 20.0),
                if is_active {
                    Color::rgb(0.0, 0.90, 1.0)
                } else {
                    Color::rgba(0.6, 0.7, 0.85, 0.8)
                },
            );
        }

        // 3. Active Asset Price & Metrics
        let price_str = format!("${:.2}", active_asset.price_usd);
        canvas.draw_text(
            &price_str,
            "Segoe UI Variable Display",
            30.0,
            RectF::new(25.0, 60.0, 240.0, 38.0),
            Color::rgb(0.95, 0.98, 1.0),
        );

        let delta_color = if active_asset.change_24h_pct >= 0.0 {
            Color::rgb(0.0, 0.92, 0.65)
        } else {
            Color::rgb(1.0, 0.35, 0.45)
        };
        let delta_str = format!("{:+.2}%", active_asset.change_24h_pct);
        canvas.draw_text(
            &delta_str,
            "Segoe UI Variable Text",
            14.0,
            RectF::new(270.0, 68.0, 90.0, 22.0),
            delta_color,
        );

        // 4. RSI & 24h High/Low Indicators
        let indicator_str = format!(
            "RSI-14: {:.1}  •  24h High: ${:.0}  •  24h Low: ${:.0}",
            active_asset.rsi_14, active_asset.high_24h_usd, active_asset.low_24h_usd
        );
        canvas.draw_text(
            &indicator_str,
            "Segoe UI Variable Text",
            11.0,
            RectF::new(25.0, 105.0, 410.0, 18.0),
            Color::rgba(0.65, 0.75, 0.90, 0.85),
        );

        // 5. Historical Spline Area Graph
        if active_asset.sparkline_7d.len() >= 2 {
            let pts: Vec<(f32, f32)> = active_asset
                .sparkline_7d
                .iter()
                .enumerate()
                .map(|(idx, &p)| {
                    let px = 25.0 + (idx as f32 * 20.0);
                    let min_p = active_asset.low_24h_usd as f32;
                    let max_p = active_asset.high_24h_usd as f32;
                    let range = (max_p - min_p).max(1.0);
                    let py = 220.0 - ((p - min_p) / range * 80.0);
                    (px, py)
                })
                .collect();

            canvas.draw_spline_area(pts, Color::rgba(0.0, 0.85, 1.0, 0.35), Color::rgb(0.0, 0.9, 1.0), 2.0);
        }

        Ok(())
    }

    fn on_unmount(&mut self) -> Result<()> {
        self.state = WidgetState::Unmounted;
        info!("CryptoStocksWidget unmounted.");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<()> {
        self.state = WidgetState::Unloaded;
        info!("CryptoStocksWidget unloaded.");
        Ok(())
    }

    fn state(&self) -> WidgetState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use widget_sdk::events::MouseButton;

    #[test]
    fn test_crypto_stocks_widget_lifecycle() {
        let cache = SharedTelemetryCache::new();
        let mut widget = CryptoStocksWidget::new(cache);
        assert_eq!(widget.state(), WidgetState::Unloaded);

        assert!(widget.on_load().is_ok());
        assert_eq!(widget.state(), WidgetState::Loaded);

        assert!(widget.on_mount().is_ok());
        assert_eq!(widget.state(), WidgetState::Mounted);

        let ctx = TickContext {
            timestamp_ms: 1000,
            delta_time_ms: 16.0,
            frame_index: 1,
        };
        assert!(widget.on_update(&ctx).is_ok());

        assert!(widget.on_unmount().is_ok());
        assert_eq!(widget.state(), WidgetState::Unmounted);

        assert!(widget.on_unload().is_ok());
        assert_eq!(widget.state(), WidgetState::Unloaded);
    }

    #[test]
    fn test_crypto_asset_tab_switching() {
        let cache = SharedTelemetryCache::new();
        let mut widget = CryptoStocksWidget::new(cache);
        assert_eq!(widget.selected_symbol(), "BTC");

        let clicked = widget.handle_pointer_event(PointerEvent::Down {
            x: 135.0,
            y: 25.0,
            button: MouseButton::Left,
        });
        assert_eq!(clicked, Some("ETH".to_string()));
        assert_eq!(widget.selected_symbol(), "ETH");
    }
}
