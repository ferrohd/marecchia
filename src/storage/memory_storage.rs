use super::SegmentStorage;
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct MemoryStorage(LruCache<String, Vec<u8>>);

impl MemoryStorage {
    pub fn new() -> Self {
        Self(LruCache::new(NonZeroUsize::new(20).unwrap()))
    }
}

impl SegmentStorage for MemoryStorage {
    fn get(&mut self, key: &str) -> Option<&Vec<u8>> {
        self.0.get(key)
    }

    fn set(&mut self, key: &str, value: Vec<u8>) -> Option<(String, Vec<u8>)> {
        self.0
            .put(key.to_string(), value)
            .map(|old| (key.to_string(), old))
    }
}
