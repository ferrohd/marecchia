use libp2p::{
    autonat,
    core::{self, muxing::StreamMuxerBox, transport::Boxed},
    dns,
    futures::StreamExt,
    identity::{self, Keypair},
    kad, noise, ping, plaintext, request_response as rr,
    swarm::{self, SwarmEvent},
    tcp, wasm_ext, webrtc, websocket, yamux, PeerId, Transport,
};
use std::{io, iter, num::NonZeroU32, time::Duration};

mod segment_protocol;
mod event_loop;
mod behaviour;

use behaviour::ComposedSwarmBehaviour;

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a random PeerId
    let keypair = identity::Keypair::generate_ed25519();
    let local_peer_id = keypair.public().to_peer_id();

    // Create a transport
    let transport = create_transport(&keypair)
        .await
        .expect("Cannot create transport");

    // Create a Swarm to manage peers and events
    let mut swarm = {
        // Define the various behaviours of the swarm.
        let ping_config = ping::Config::new()
            .with_max_failures(NonZeroU32::new(1).unwrap())
            .with_timeout(Duration::from_secs(5))
            .with_interval(Duration::from_secs(5));
        let ping = ping::Behaviour::new(ping_config);

        let autonat_config = autonat::Config::default();
        let autonat = autonat::Behaviour::new(local_peer_id, autonat_config);

        let kademlia_config = kad::KademliaConfig::default();
        let kademlia_store = kad::store::MemoryStore::new(local_peer_id);
        let kademlia = kad::Kademlia::with_config(local_peer_id, kademlia_store, kademlia_config);

        let rr_codec = segment_protocol::SegmentExchangeCodec();
        let rr_protocol = segment_protocol::SegmentExchangeProtocol();
        let request_response = rr::Behaviour::new(
            rr_codec,
            // ! Pro Tip: Multiple protocols can be used here.
            iter::once((rr_protocol, rr::ProtocolSupport::Full)),
            Default::default(),
        );

        let behaviour = ComposedSwarmBehaviour {
            //ping,
            //autonat,
            kademlia,
            request_response,
        };

        // Do wthings with behaviours

        swarm::Swarm::with_threadpool_executor(transport, behaviour, local_peer_id)
    };
    Ok(())
}

async fn create_transport(keypair: &Keypair) -> Result<Boxed<(PeerId, StreamMuxerBox)>, io::Error> {
    let dns_tcp =
        dns::DnsConfig::system(tcp::tokio::Transport::new(tcp::Config::new().nodelay(true)))
            .await?
            .upgrade(core::upgrade::Version::V1)
            .authenticate(noise::NoiseAuthenticated::xx(&keypair).unwrap())
            .multiplex(yamux::YamuxConfig::default())
            .timeout(Duration::from_secs(20))
            .boxed();
    Ok(dns_tcp)
}