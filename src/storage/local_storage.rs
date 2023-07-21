use std::num::NonZeroUsize;

use lru::LruCache;

pub struct LocalStorage<'a> {
    storage: LruCache<&'a str, Vec<u8>>,   
}

impl<'a> LocalStorage<'a> {
    pub fn new() -> Self {
        LocalStorage {
            storage: LruCache::new(NonZeroUsize::new(10).unwrap()),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&Vec<u8>> {
        self.storage.get(key)
    }

    pub fn set(&mut self, key: &'a str, value: Vec<u8>) -> Option<(&str, Vec<u8>)> {
        self.storage.push(key, value)
    }


    
}

