use std::time::Duration;

use libp2p::{
    autonat,
    identity::Keypair,
    kad::{self, store::MemoryStore, Behaviour},
    ping,
    request_response::{self, ProtocolSupport},
    swarm::NetworkBehaviour,
    PeerId, StreamProtocol,
};
use serde::{Deserialize, Serialize};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ComposedSwarmEvent")]
pub struct ComposedSwarmBehaviour {
    pub ping: ping::Behaviour,
    pub segment_rr: request_response::cbor::Behaviour<SegmentRequest, SegmentResponse>,
    pub kademlia: Behaviour<MemoryStore>,
}

impl From<PeerId> for ComposedSwarmBehaviour {
    fn from(peer_id: PeerId) -> Self {
        // Define the various behaviours of the swarm.
        let ping_config = ping::Config::new()
            .with_timeout(Duration::from_secs(10))
            .with_interval(Duration::from_secs(5));
        let ping = ping::Behaviour::new(ping_config);

        let autonat_config = autonat::Config::default();
        let autonat = autonat::Behaviour::new(peer_id, autonat_config);

        let kademlia_config = kad::Config::default()
            //.set_connection_idle_timeout(Duration::from_secs(60))
            .set_provider_publication_interval(Some(Duration::from_secs(30)))
            .set_provider_record_ttl(None)
            .set_publication_interval(Some(Duration::from_secs(30)))
            .set_record_ttl(Some(Duration::from_secs(60)))
            .set_replication_interval(Some(Duration::from_secs(5)))
            .to_owned();
        let kademlia_store = kad::store::MemoryStore::new(peer_id);
        let kademlia = kad::Behaviour::with_config(peer_id, kademlia_store, kademlia_config);

        let request_response = request_response::Behaviour::new(
            [(StreamProtocol::new("/segment/1"), ProtocolSupport::Full)],
            request_response::Config::default(),
        );

        Self {
            ping,
            //autonat,
            kademlia,
            segment_rr: request_response,
        }
    }
}

impl From<&Keypair> for ComposedSwarmBehaviour {
    fn from(keypair: &Keypair) -> Self {
        let peer_id = keypair.public().to_peer_id();
        Self::from(peer_id)
    }
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

// Request segment id
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentRequest(pub String);

// Response segment data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SegmentResponse(pub Option<Vec<u8>>);
