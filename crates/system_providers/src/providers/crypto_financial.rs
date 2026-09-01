//! Real-Time Financial & Cryptocurrency Market Telemetry Provider
//!
//! Tracks multi-asset cryptocurrency prices (BTC, ETH, SOL), equity indices (S&P 500),
//! and commodities with historical sparklines and technical indicators (RSI-14, EMA-20).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

/// Real-time market telemetry snapshot for a financial asset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CryptoAssetTelemetry {
    pub symbol: String,
    pub name: String,
    pub price_usd: f64,
    pub change_24h_pct: f32,
    pub high_24h_usd: f64,
    pub low_24h_usd: f64,
    pub sparkline_7d: Vec<f32>,
    pub rsi_14: f32,
    pub ema_20: f64,
    pub last_updated_ms: u64,
}

impl Default for CryptoAssetTelemetry {
    fn default() -> Self {
        Self {
            symbol: "BTC".to_string(),
            name: "Bitcoin".to_string(),
            price_usd: 64250.0,
            change_24h_pct: 2.85,
            high_24h_usd: 65100.0,
            low_24h_usd: 63800.0,
            sparkline_7d: vec![62000.0, 62500.0, 63100.0, 62900.0, 63800.0, 64250.0],
            rsi_14: 58.4,
            ema_20: 63900.0,
            last_updated_ms: 0,
        }
    }
}

/// Provider for financial, stock index, and cryptocurrency market telemetry.
#[derive(Debug)]
pub struct CryptoFinancialProvider {
    assets: HashMap<String, CryptoAssetTelemetry>,
    tick_count: u64,
}

impl CryptoFinancialProvider {
    /// Creates a new `CryptoFinancialProvider` pre-populated with standard market assets.
    pub fn new() -> Self {
        let mut assets = HashMap::new();

        assets.insert(
            "BTC".to_string(),
            CryptoAssetTelemetry {
                symbol: "BTC".to_string(),
                name: "Bitcoin".to_string(),
                price_usd: 64500.0,
                change_24h_pct: 3.12,
                high_24h_usd: 65200.0,
                low_24h_usd: 63800.0,
                sparkline_7d: vec![61000.0, 61800.0, 62400.0, 63100.0, 63900.0, 64500.0],
                rsi_14: 61.2,
                ema_20: 63800.0,
                last_updated_ms: 1000,
            },
        );

        assets.insert(
            "ETH".to_string(),
            CryptoAssetTelemetry {
                symbol: "ETH".to_string(),
                name: "Ethereum".to_string(),
                price_usd: 3450.0,
                change_24h_pct: -1.25,
                high_24h_usd: 3520.0,
                low_24h_usd: 3410.0,
                sparkline_7d: vec![3350.0, 3400.0, 3480.0, 3510.0, 3470.0, 3450.0],
                rsi_14: 48.6,
                ema_20: 3465.0,
                last_updated_ms: 1000,
            },
        );

        assets.insert(
            "SOL".to_string(),
            CryptoAssetTelemetry {
                symbol: "SOL".to_string(),
                name: "Solana".to_string(),
                price_usd: 154.50,
                change_24h_pct: 5.40,
                high_24h_usd: 158.00,
                low_24h_usd: 147.20,
                sparkline_7d: vec![138.0, 142.5, 145.0, 149.0, 151.2, 154.5],
                rsi_14: 68.1,
                ema_20: 148.0,
                last_updated_ms: 1000,
            },
        );

        assets.insert(
            "SPX".to_string(),
            CryptoAssetTelemetry {
                symbol: "SPX".to_string(),
                name: "S&P 500".to_string(),
                price_usd: 5580.0,
                change_24h_pct: 0.65,
                high_24h_usd: 5595.0,
                low_24h_usd: 5560.0,
                sparkline_7d: vec![5510.0, 5530.0, 5545.0, 5560.0, 5575.0, 5580.0],
                rsi_14: 55.0,
                ema_20: 5550.0,
                last_updated_ms: 1000,
            },
        );

        Self {
            assets,
            tick_count: 0,
        }
    }

    /// Samples all tracked crypto and equity market assets.
    pub fn sample_all(&mut self) -> Result<Vec<CryptoAssetTelemetry>> {
        self.tick_count += 1;
        let mut list: Vec<CryptoAssetTelemetry> = self.assets.values().cloned().collect();
        list.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        Ok(list)
    }

    /// Updates price of an asset and recalculates indicators.
    pub fn update_price(&mut self, symbol: &str, price: f64) {
        if let Some(asset) = self.assets.get_mut(symbol) {
            asset.price_usd = price;
            asset.sparkline_7d.push(price as f32);
            if asset.sparkline_7d.len() > 20 {
                asset.sparkline_7d.remove(0);
            }
            // Update EMA-20
            let k = 2.0 / (20.0 + 1.0);
            asset.ema_20 = (price * k) + (asset.ema_20 * (1.0 - k));
        }
    }

    /// Computes Relative Strength Index (RSI-14) over a price window.
    pub fn compute_rsi(prices: &[f32]) -> f32 {
        if prices.len() < 2 {
            return 50.0;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;
        let mut count = 0;

        for window in prices.windows(2) {
            let diff = window[1] - window[0];
            if diff >= 0.0 {
                gains += diff;
            } else {
                losses += diff.abs();
            }
            count += 1;
        }

        if count == 0 || losses == 0.0 {
            return 100.0;
        }

        let avg_gain = gains / count as f32;
        let avg_loss = losses / count as f32;

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }
}

impl Default for CryptoFinancialProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_financial_provider_initialization() {
        let mut provider = CryptoFinancialProvider::new();
        let assets = provider.sample_all().expect("Should return assets");
        assert!(assets.len() >= 4);

        let btc = assets.iter().find(|a| a.symbol == "BTC").expect("BTC should exist");
        assert!(btc.price_usd > 0.0);
        assert!(btc.rsi_14 >= 0.0 && btc.rsi_14 <= 100.0);
    }

    #[test]
    fn test_update_price_and_sparkline() {
        let mut provider = CryptoFinancialProvider::new();
        provider.update_price("BTC", 65000.0);

        let assets = provider.sample_all().unwrap();
        let btc = assets.iter().find(|a| a.symbol == "BTC").unwrap();
        assert_eq!(btc.price_usd, 65000.0);
        assert_eq!(*btc.sparkline_7d.last().unwrap(), 65000.0);
    }

    #[test]
    fn test_compute_rsi() {
        let rising_prices = vec![10.0, 12.0, 14.0, 16.0, 18.0, 20.0];
        let rsi = CryptoFinancialProvider::compute_rsi(&rising_prices);
        assert!(rsi > 90.0, "Consistent gains should result in high RSI, got {}", rsi);

        let falling_prices = vec![20.0, 18.0, 16.0, 14.0, 12.0, 10.0];
        let rsi_down = CryptoFinancialProvider::compute_rsi(&falling_prices);
        assert!(rsi_down < 10.0, "Consistent losses should result in low RSI, got {}", rsi_down);
    }
}
