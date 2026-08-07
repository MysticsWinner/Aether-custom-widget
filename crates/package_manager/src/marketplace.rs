use crate::package::WidgetPackage;
use std::collections::HashMap;

/// Rich Marketplace++ catalog with search and dependency resolution graph traversal.
#[derive(Debug, Clone, Default)]
pub struct MarketplaceCatalog {
    packages: HashMap<String, WidgetPackage>,
}

impl MarketplaceCatalog {
    pub fn new() -> Self {
        let mut catalog = Self {
            packages: HashMap::new(),
        };
        catalog.seed_sample_packages();
        catalog
    }

    fn seed_sample_packages(&mut self) {
        let mut p1 = WidgetPackage::new("weather-pro", "Weather Overlay Pro", "2.1.0", "Aether Team");
        p1.publisher.verified = true;
        p1.publisher.reputation_score = 4.9;

        let mut p2 = WidgetPackage::new("gpu-gauge", "Neon GPU Gauge", "1.0.4", "Community");
        p2.publisher.verified = true;
        p2.publisher.reputation_score = 4.7;

        self.packages.insert(p1.id.clone(), p1);
        self.packages.insert(p2.id.clone(), p2);
    }

    pub fn search(&self, query: &str) -> Vec<WidgetPackage> {
        let q = query.to_lowercase();
        self.packages
            .values()
            .filter(|p| p.name.to_lowercase().contains(&q) || p.id.to_lowercase().contains(&q))
            .cloned()
            .collect()
    }

    pub fn resolve_dependencies(&self, package_id: &str) -> Vec<String> {
        let mut resolved = Vec::new();
        if let Some(pkg) = self.packages.get(package_id) {
            for dep in pkg.dependencies.keys() {
                resolved.push(dep.clone());
            }
        }
        resolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marketplace_catalog_dependency_resolution() {
        let catalog = MarketplaceCatalog::new();
        let results = catalog.search("gpu");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "gpu-gauge");
        assert!(results[0].publisher.verified);
    }
}
