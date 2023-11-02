use libp2p::{
    kad::{record::store::MemoryStore, Behaviour, self},
    ping, request_response,
    swarm::NetworkBehaviour,
};

use super::protocol::segment_protocol::{SegmentExchangeCodec, SegmentRequest, SegmentResponse};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ComposedSwarmEvent")]
pub struct ComposedSwarmBehaviour {
    pub ping: ping::Behaviour,
    pub segment_rr: request_response::Behaviour<SegmentExchangeCodec>,
    pub kademlia: Behaviour<MemoryStore>,
}

#[derive(Debug)]
pub enum ComposedSwarmEvent {
    Ping(ping::Event),
    RequestResponse(request_response::Event<SegmentRequest, SegmentResponse>),
    Kademlia(kad::Event),
}

impl From<ping::Event> for ComposedSwarmEvent {
    fn from(event: ping::Event) -> Self {
        ComposedSwarmEvent::Ping(event)
    }
}

impl From<request_response::Event<SegmentRequest, SegmentResponse>> for ComposedSwarmEvent {
    fn from(event: request_response::Event<SegmentRequest, SegmentResponse>) -> Self {
        ComposedSwarmEvent::RequestResponse(event)
    }
}

impl From<kad::Event> for ComposedSwarmEvent {
    fn from(event: kad::Event) -> Self {
        ComposedSwarmEvent::Kademlia(event)
    }
}
