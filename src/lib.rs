use async_std::stream::StreamExt;
use storage::local_storage::LocalStorage;
use wasm_bindgen::prelude::*;

use crate::p2p::event_loop::LoopEvent;

mod http;
mod loader;
mod p2p;
mod storage;

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
        Self
    }
}

struct P2PStreamStrategy;
impl StreamStrategy for P2PStreamStrategy {
    fn new() -> Self {
        Self
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
    use crate::storage::local_storage::LocalStorage;

    let mut local_storage = LocalStorage::new();

    let (mut client, mut event_stream, event_loop) = client::new(None).await.unwrap();

    wasm_bindgen_futures::spawn_local(async move {
        event_loop.run().await;
    });


    // Player request segment
    let segment_id = "segment_1";
    let segment = match local_storage.get(segment_id) {
        Some(segment) => segment.to_vec(),
        None => {

            let providers = client.get_providers(segment_id.to_string()).await;
            let first_peer = providers.iter().next().unwrap();

            let segment = match client.request_segment(first_peer.to_owned(), segment_id.to_string()).await {
                Ok(segment) => match segment {
                    Some(segment) => segment,
                    None => http::downloader::download_segment(segment_id).await.unwrap(),
                }
                Err(_) => http::downloader::download_segment(segment_id).await.unwrap(),
            };

            local_storage.set(segment_id, segment.clone());
            segment
        }
    };

    loop {
        let event = event_stream.next().await.unwrap();
        match event {
            LoopEvent::SegmentRequest { segment_id, channel } => {
                let segment = local_storage.get(&segment_id).map(|s| s.to_vec());

                client.respond_segment(segment, channel).await;
            }
        }
    }

    Ok(())
}
