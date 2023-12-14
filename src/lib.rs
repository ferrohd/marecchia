use futures::{future::select_ok, StreamExt};
use libp2p::Multiaddr;
use p2p::client;
use storage::memory_storage::MemoryStorage;
use wasm_bindgen::prelude::*;

use crate::{p2p::event_loop::LoopEvent, storage::SegmentStorage};

mod http;
mod loader;
mod p2p;
mod storage;

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    tracing_wasm::set_as_global_default();

    let mut local_storage = MemoryStorage::new();

    let (mut client, mut event_stream) = client::new(None).await.unwrap();

    let server_peer_id = "".parse().unwrap();
    let server_addr = "/ip4/0.0.0.0".parse::<Multiaddr>().unwrap();

    client.start_listening().await.unwrap();
    client.dial(server_peer_id, server_addr).await;

    // Player request segment
    let segment_id = "segment_1";
    let segment = match local_storage.get(segment_id) {
        Some(segment) => Some(segment.to_owned()),
        None => {
            let providers = client.get_providers(segment_id.to_string()).await;

            if providers.is_empty() {
                match http::downloader::download_segment(segment_id).await {
                    Ok(segment) => {
                        local_storage.set(segment_id, segment.to_vec());
                        client.start_providing(segment_id.to_string()).await;
                        return Ok(());
                    }
                    Err(_) => return Ok(()),
                }
            }

            let mut segments = Vec::new();
            for provider in providers {
                let r = client
                    .request_segment(provider.to_owned(), segment_id.to_string())
                    .await
                    .ok()
                    .flatten();
                segments.push(r);
            }

            let segment = segments.into_iter().find(|s| s.is_some()).flatten();
            match segment {
                Some(seg) => {
                    local_storage.set(segment_id, seg.to_vec());
                    client.start_providing(segment_id.to_string()).await;
                    Some(seg)
                }
                None => {
                    match http::downloader::download_segment(segment_id)
                    .await {
                        Ok(segment) => {
                            local_storage.set(segment_id, segment.to_vec());
                            client.start_providing(segment_id.to_string()).await;
                            Some(segment)
                        }
                        Err(_) => None,
                    }
                },
            }
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
