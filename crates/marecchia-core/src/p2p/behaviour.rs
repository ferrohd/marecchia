use std::time::Duration;

use libp2p::{
    autonat,
    gossipsub::{self, MessageAuthenticity},
    identity::Keypair,
    ping,
    rendezvous::client as rendezvous,
    swarm::NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ComposedSwarmEvent")]
pub struct ComposedSwarmBehaviour {
    pub ping: ping::Behaviour,
    pub rendezvous: rendezvous::Behaviour,
    pub pubsub: gossipsub::Behaviour,
}

impl From<&Keypair> for ComposedSwarmBehaviour {
    fn from(keypair: &Keypair) -> Self {
        let peer_id = keypair.public().to_peer_id();
        // Define the various behaviours of the swarm.
        let ping_config = ping::Config::new()
        .with_interval(Duration::from_secs(5))
        .with_timeout(Duration::from_secs(10));
        let ping = ping::Behaviour::new(ping_config);

        let autonat_config = autonat::Config::default();
        let _autonat = autonat::Behaviour::new(peer_id, autonat_config);

        let rendezvous = rendezvous::Behaviour::new(keypair.to_owned());
        // TODO: FINISH CONFIG
        let gossipsub_config = gossipsub::Config::default();
        let pubsub =
            gossipsub::Behaviour::new(MessageAuthenticity::Author(peer_id), gossipsub_config)
                .unwrap();
        Self {
            ping,
            //autonat,
            pubsub,
            rendezvous,
        }
    }
}

#[derive(Debug)]
pub enum ComposedSwarmEvent {
    Ping(ping::Event),
    Rendezvous(rendezvous::Event),
    Gossipsub(gossipsub::Event),
}

impl From<ping::Event> for ComposedSwarmEvent {
    fn from(event: ping::Event) -> Self {
        ComposedSwarmEvent::Ping(event)
    }
}

impl From<rendezvous::Event> for ComposedSwarmEvent {
    fn from(event: rendezvous::Event) -> Self {
        ComposedSwarmEvent::Rendezvous(event)
    }
}

impl From<gossipsub::Event> for ComposedSwarmEvent {
    fn from(event: gossipsub::Event) -> Self {
        ComposedSwarmEvent::Gossipsub(event)
    }
}
