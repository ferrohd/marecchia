use wasm_bindgen::prelude::*;

mod p2p;
mod http;
mod loader;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);
}
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

trait StreamStrategy {
    fn new() -> Self;
    //fn get_video(&self) -> Vec<u8>;
}

struct HttpStreamStrategy;
impl StreamStrategy for HttpStreamStrategy {
    fn new() -> Self {
        HttpStreamStrategy
    }
}

struct P2PStreamStrategy;
impl StreamStrategy for P2PStreamStrategy {
    fn new() -> Self {
        P2PStreamStrategy
    }
}

struct VideoEngine<T: StreamStrategy> {
    stream_strategy: T,
}
impl<T: StreamStrategy> VideoEngine<T> {
    fn new(stream_strategy: T) -> Self {
        VideoEngine { stream_strategy }
    }
}

#[wasm_bindgen(start)]
async fn main() -> Result<(), JsValue> {
    use crate::p2p::client;
    let client = client::new(None);
    Ok(())
}
