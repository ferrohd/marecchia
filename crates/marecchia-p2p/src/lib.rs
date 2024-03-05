use p2p::client;
use wasm_bindgen::prelude::*;

mod http;
mod loader;
mod p2p;

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    tracing_wasm::set_as_global_default();

    let mut client = client::new_p2p_client("idstream".to_string(), None).await.unwrap();

    Ok(())
}
