use libp2p::kad::record::store::MemoryStore;
use libp2p::{
    kad::{Kademlia, KademliaEvent},
    request_response,
    swarm::NetworkBehaviour,
};

use super::segment_protocol::{SegmentExchangeCodec, SegmentRequest, SegmentResponse};

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "ComposedSwarmEvent")]
pub struct ComposedSwarmBehaviour {
    pub request_response: request_response::Behaviour<SegmentExchangeCodec>,
    pub kademlia: Kademlia<MemoryStore>,
}

#[derive(Debug)]
pub enum ComposedSwarmEvent {
    RequestResponse(request_response::Event<SegmentRequest, SegmentResponse>),
    Kademlia(KademliaEvent),
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
