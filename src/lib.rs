use p2p::client;
use storage::memory_storage::MemoryStorage;
use wasm_bindgen::prelude::*;

mod http;
mod loader;
mod p2p;
mod storage;

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    tracing_wasm::set_as_global_default();

    let _local_storage = MemoryStorage::new();

    let mut client = client::new("idstream".to_string(), None).await.unwrap();

    Ok(())
}
