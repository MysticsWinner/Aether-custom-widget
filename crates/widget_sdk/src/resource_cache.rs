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

    pub fn clear(&mut self) {
        self.entries.clear();
        self.usage_order.clear();
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
}
