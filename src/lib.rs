use futures::{StreamExt, future::select_ok};
use libp2p::Multiaddr;
use p2p::client;
use storage::memory_storage::MemoryStorage;
use wasm_bindgen::prelude::*;
use wasm_logger;

use crate::{p2p::event_loop::LoopEvent, storage::SegmentStorage};

mod http;
mod loader;
mod p2p;
mod storage;

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());

    let mut local_storage = MemoryStorage::new();

    let (mut client, mut event_stream) = client::new(None).await.unwrap();

    let server_peer_id = "".parse().unwrap();
    let server_addr = "/ip4/0.0.0.0".parse::<Multiaddr>().unwrap();

    client.start_listening().await.unwrap();
    client.dial(server_peer_id, server_addr);

    // Player request segment
    let segment_id = "segment_1";
    let segment = match local_storage.get(segment_id) {
        Some(segment) => Some(segment.to_vec()),
        None => {
            let providers = client.get_providers(segment_id.to_string()).await;
            if providers.is_empty() {
                http::downloader::download_segment(segment_id).await.ok();
            }

            let segment_request = providers
                .iter()
                .map(|p| Box::pin(client.request_segment(p.to_owned(), segment_id.to_string())));
            let segment_response = select_ok(segment_request).await;
            let segment = match segment_response {
                Ok((segment, _)) => match segment {
                    Some(segment) => Some(segment),
                    None => http::downloader::download_segment(segment_id).await.ok(),
                },
                Err(a) => http::downloader::download_segment(segment_id).await.ok(),
            };

            if let Some(seg) = segment.clone() {
                local_storage.set(segment_id, seg);
                client.start_providing(segment_id.to_string()).await;
            }

            segment
        }
    };
    wasm_bindgen_futures::spawn_local(async move {
        loop {
            let event: LoopEvent = event_stream.next().await.unwrap();
            match event {
                LoopEvent::SegmentRequest {
                    segment_id,
                    channel,
                } => {
                    let segment = local_storage.get(&segment_id).map(|s| s.to_vec());

                    client.respond_segment(segment, channel).await;
                }
            }
        }
    });

    Ok(())
}
