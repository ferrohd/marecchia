use lrumap::{LruHashMap, Removed};

pub struct LocalStorage {
    storage: LruHashMap<i64, Vec<u8>>,   
}

impl LocalStorage {
    pub fn new() -> Self {
        LocalStorage {
            storage: LruHashMap::new(10),
        }
    }

    pub fn get(&mut self, key: i64) -> Option<&Vec<u8>> {
        self.storage.get(&key)
    }

    pub fn set(&mut self, key: i64, value: Vec<u8>) -> Option<Removed<i64, Vec<u8>>> {
        self.storage.push(key, value)
    }


    
}

