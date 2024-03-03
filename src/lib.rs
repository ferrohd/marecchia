use libp2p::Multiaddr;
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

    let server_peer_id = "".parse().unwrap();
    let server_addr = "/ip4/0.0.0.0".parse::<Multiaddr>().unwrap();
    // TODO: Move this dial inside client
    client.dial(server_peer_id, server_addr).await;

    Ok(())
}
