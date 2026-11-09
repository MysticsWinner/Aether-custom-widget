use std::collections::HashMap;

/// LRU resource cache for rendering objects (brushes, fonts, textures).
#[derive(Debug, Clone)]
pub struct LruResourceCache {
    capacity: usize,
    entries: HashMap<String, String>, // key -> cached payload representation
    usage_order: Vec<String>,
}

impl LruResourceCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: HashMap::new(),
            usage_order: Vec::new(),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&String> {
        if self.entries.contains_key(key) {
            self.usage_order.retain(|k| k != key);
            self.usage_order.push(key.to_string());
            self.entries.get(key)
        } else {
            None
        }
    }

    pub fn put(&mut self, key: &str, value: &str) {
        if self.entries.contains_key(key) {
            self.usage_order.retain(|k| k != key);
        } else if self.entries.len() >= self.capacity {
            if !self.usage_order.is_empty() {
                let lru_key = self.usage_order.remove(0);
                self.entries.remove(&lru_key);
            }
        }
        self.entries.insert(key.to_string(), value.to_string());
        self.usage_order.push(key.to_string());
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn set_capacity(&mut self, new_capacity: usize) {
        self.capacity = new_capacity;
        while self.entries.len() > self.capacity && !self.usage_order.is_empty() {
            let lru_key = self.usage_order.remove(0);
            self.entries.remove(&lru_key);
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.usage_order.clear();
    }

    /// Performs resource garbage collection.
    ///
    /// In normal mode, trims entries down to 50% of capacity if over half full.
    /// In aggressive mode (e.g. dev mode or low-memory triggers), flushes at least 80%
    /// or all unpinned entries, returning total count of evicted entries.
    pub fn collect_garbage(&mut self, aggressive: bool) -> usize {
        let initial_count = self.entries.len();
        let target_count = if aggressive {
            0 // aggressive dev mode: purge all cached render resources
        } else {
            self.capacity / 2 // gentle mode: trim to half capacity
        };

        while self.entries.len() > target_count && !self.usage_order.is_empty() {
            let lru_key = self.usage_order.remove(0);
            self.entries.remove(&lru_key);
        }

        initial_count.saturating_sub(self.entries.len())
    }

    /// Shrinks cache to a target capacity and evicts excess entries.
    pub fn shrink_to_fit(&mut self, target_capacity: usize) -> usize {
        self.set_capacity(target_capacity);
        self.entries.shrink_to_fit();
        self.usage_order.shrink_to_fit();
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_resource_cache_eviction_on_capacity() {
        let mut cache = LruResourceCache::new(2);

        cache.put("brush_red", "Color(255,0,0)");
        cache.put("brush_blue", "Color(0,0,255)");
        assert_eq!(cache.len(), 2);

        // Access brush_red to update LRU order
        assert!(cache.get("brush_red").is_some());

        // Insert third item -> brush_blue should be evicted as LRU
        cache.put("brush_green", "Color(0,255,0)");
        assert_eq!(cache.len(), 2);

        assert!(cache.get("brush_red").is_some());
        assert!(cache.get("brush_green").is_some());
        assert!(cache.get("brush_blue").is_none());
    }

    #[test]
    fn test_lru_resource_cache_garbage_collection() {
        let mut cache = LruResourceCache::new(10);
        for i in 0..8 {
            cache.put(&format!("key_{i}"), &format!("val_{i}"));
        }
        assert_eq!(cache.len(), 8);

        // Normal GC trims down to half capacity (5)
        let evicted = cache.collect_garbage(false);
        assert_eq!(evicted, 3);
        assert_eq!(cache.len(), 5);

        // Aggressive GC trims everything down to 0
        let aggressive_evicted = cache.collect_garbage(true);
        assert_eq!(aggressive_evicted, 5);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_lru_resource_cache_shrink_to_fit() {
        let mut cache = LruResourceCache::new(10);
        for i in 0..6 {
            cache.put(&format!("k_{i}"), &format!("v_{i}"));
        }
        assert_eq!(cache.capacity(), 10);
        assert_eq!(cache.len(), 6);

        cache.shrink_to_fit(3);
        assert_eq!(cache.capacity(), 3);
        assert_eq!(cache.len(), 3);
    }
}

