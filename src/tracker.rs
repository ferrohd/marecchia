use std::io::Cursor;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{
    console, window, MessageEvent, RtcPeerConnection, RtcSessionDescriptionInit, WebSocket,
};

#[derive(Serialize, Deserialize)]
pub struct ScrapeMessage {
    info_hash: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct ScrapeResponse {
    files: Vec<ScrapeFile>,
}
#[derive(Serialize, Deserialize)]
pub struct ScrapeFile {
    info_hash: String,
    complete: u32,
    incomplete: u32,
    downloaded: u32,
}
// Announce
#[derive(Serialize, Deserialize)]
pub enum AnnounceEvent {
    Started,
    Stopped,
    Completed,
    Update,
}
#[derive(Serialize, Deserialize)]
pub struct AnnounceMessage {
    info_hash: String,
    peer_id: String,
    uploaded: u64,
    downloaded: u64,
    left: u64,
    event: AnnounceEvent, //ip: String Called by tracker to find out the IP address of the client, if it is behind a NAT
}
#[derive(Serialize, Deserialize)]
pub struct AnnounceResponse {
    interval: u32,
    peers: Vec<PeerInfo>,
}
#[derive(Serialize, Deserialize)]
pub struct PeerInfo {
    peer_id: String,
    ip: u16,
}

// Offer
type Sdp = String;
#[derive(Serialize, Deserialize)]
pub struct OfferMessage {
    peer_id: String,
    offer: Sdp,
}
// Answer
#[derive(Serialize, Deserialize)]
pub struct AnswerMessage {
    peer_id: String,
    answer: Sdp,
}

#[derive(Serialize, Deserialize)]
pub enum TrackerMessage {
    Scrape(ScrapeMessage),
    Announce(AnnounceMessage),
    Offer(OfferMessage),
    Answer(AnswerMessage),
}
#[derive(Serialize, Deserialize)]
pub enum TrackerMessageResponse {
    Scrape(ScrapeResponse),
    Announce(AnnounceResponse),
    Offer(OfferMessage),
    Answer(AnswerMessage),
}

pub struct Tracker {
    url: String,
    tracker_socket: WebSocket,
    announce_interval: Option<i32>,
}

impl Tracker {
    pub fn new(url: String) -> Tracker {
        let tracker_socket = WebSocket::new(&url).unwrap();
        tracker_socket.set_binary_type(web_sys::BinaryType::Arraybuffer);
       

        Tracker {
            url,
            tracker_socket,
            announce_interval: None,
        }

    }
    pub fn start(&self) {
        let tracker_socket = self.tracker_socket.clone();
/*
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            let msg_vec = js_sys::Uint8Array::new(&e.data()).to_vec();
            match TrackerMessageResponse::deserialize(&mut Deserializer::new(Cursor::new(msg_vec)))
            {
                Err(e) => console::log_1(&format!("Error: {:?}", e).into()),
                Ok(msg) => self.handle_response(msg),
            }
        });

        tracker_socket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
*/
    }
    fn handle_response(&self, msg: TrackerMessageResponse) {
        match msg {
            TrackerMessageResponse::Announce(announce_response) => {
                
            }
            TrackerMessageResponse::Scrape(scrape_response) => {
                
            }
            TrackerMessageResponse::Offer(offer) => {
                
            }
            TrackerMessageResponse::Answer(answer) => {
                
            }
        }
    }
    fn set_announce_interval(&self, interval: i32) {
        // If the interval is already set, clear it
        if self.announce_interval.is_some() {
            window()
                .unwrap()
                .clear_interval_with_handle(self.announce_interval.unwrap());
        }
        /*
        // Define the closure
        let closure = Closure::<dyn FnMut()>::wrap(Box::new(move || {
            let msg = AnnounceMessage {
                info_hash: "test".to_string(),
                peer_id: "test".to_string(),
                uploaded: 0,
                downloaded: 0,
                left: 0,
                event: AnnounceEvent::Update,
            };
            self.announce(msg);
        }));

        // Set the interval
        self.announce_interval = Some(
            window()
                .unwrap()
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    interval,
                )
                .unwrap(),
        );
        closure.forget();
        */
    }
    pub fn scrape(&self, info_hash: Vec<String>) -> Result<(), JsValue> {
        let scrape_message = ScrapeMessage { info_hash };
        self.send(TrackerMessage::Scrape(scrape_message))
    }
    pub fn announce(&self, msg: AnnounceMessage) -> Result<(), JsValue> {
        self.send(TrackerMessage::Announce(msg))
    }
    fn send(&self, msg: TrackerMessage) -> Result<(), JsValue> {
        let mut buf = Vec::new();
        msg.serialize(&mut Serializer::new(&mut buf)).unwrap();
        self.tracker_socket.send_with_u8_array(&buf)
    }
}
