pub mod memory_storage;

pub trait SegmentStorage {
    fn get(&mut self, key: &str) -> Option<&Vec<u8>>;
    fn set(&mut self, key: &str, value: Vec<u8>) -> Option<(String, Vec<u8>)>;
}
