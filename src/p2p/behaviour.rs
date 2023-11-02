use libp2p::{
    kad::{record::store::MemoryStore, Kademlia, KademliaEvent},
    ping, request_response,
    swarm::NetworkBehaviour,
};

use super::protocol::segment_protocol::{SegmentExchangeCodec, SegmentRequest, SegmentResponse};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ComposedSwarmEvent")]
pub struct ComposedSwarmBehaviour {
    pub ping: ping::Behaviour,
    pub segment_rr: request_response::Behaviour<SegmentExchangeCodec>,
    pub kademlia: Kademlia<MemoryStore>,
}

#[derive(Debug)]
pub enum ComposedSwarmEvent {
    Ping(ping::Event),
    RequestResponse(request_response::Event<SegmentRequest, SegmentResponse>),
    Kademlia(KademliaEvent),
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

impl From<KademliaEvent> for ComposedSwarmEvent {
    fn from(event: KademliaEvent) -> Self {
        ComposedSwarmEvent::Kademlia(event)
    }
}
