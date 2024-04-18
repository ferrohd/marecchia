use std::time::Duration;

use libp2p::{
    autonat,
    gossipsub::{self, MessageAuthenticity},
    identity::Keypair,
    identify,
    ping, relay,
    rendezvous::client as rendezvous,
    swarm::NetworkBehaviour,
};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ComposedSwarmEvent")]
pub struct ComposedSwarmBehaviour {
    pub ping: ping::Behaviour,
    pub identify: identify::Behaviour,
    pub rendezvous: rendezvous::Behaviour,
    pub relay: relay::client::Behaviour,
    pub pubsub: gossipsub::Behaviour,
}

impl ComposedSwarmBehaviour {
    pub fn new(keypair: &Keypair, relay_behaviour: relay::client::Behaviour) -> Self {
        let peer_id = keypair.public().to_peer_id();
        // Define the various behaviours of the swarm.
        let ping_config = ping::Config::new()
            .with_interval(Duration::from_secs(5))
            .with_timeout(Duration::from_secs(10));
        let ping = ping::Behaviour::new(ping_config);

        let autonat_config = autonat::Config::default();
        let _autonat = autonat::Behaviour::new(peer_id, autonat_config);

        let identify_config =
            identify::Config::new("/marecchia-identify/0.0.1".to_string(), keypair.public());
        let identify = identify::Behaviour::new(identify_config);

        let rendezvous = rendezvous::Behaviour::new(keypair.to_owned());
        // TODO: FINISH CONFIG
        let gossipsub_config = gossipsub::Config::default();
        let pubsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(keypair.to_owned()),
            gossipsub_config,
        )
        .unwrap();

        Self {
            ping,
            //autonat,
            identify,
            pubsub,
            rendezvous,
            relay: relay_behaviour,
        }
    }
}

#[derive(Debug)]
pub enum ComposedSwarmEvent {
    Ping(ping::Event),
    Identify(identify::Event),
    Rendezvous(rendezvous::Event),
    Relay(relay::client::Event),
    Gossipsub(gossipsub::Event),
}

impl From<ping::Event> for ComposedSwarmEvent {
    fn from(event: ping::Event) -> Self {
        ComposedSwarmEvent::Ping(event)
    }
}

impl From<identify::Event> for ComposedSwarmEvent {
    fn from(event: identify::Event) -> Self {
        ComposedSwarmEvent::Identify(event)
    }
}

impl From<rendezvous::Event> for ComposedSwarmEvent {
    fn from(event: rendezvous::Event) -> Self {
        ComposedSwarmEvent::Rendezvous(event)
    }
}

impl From<relay::client::Event> for ComposedSwarmEvent {
    fn from(event: relay::client::Event) -> Self {
        ComposedSwarmEvent::Relay(event)
    }
}

impl From<gossipsub::Event> for ComposedSwarmEvent {
    fn from(event: gossipsub::Event) -> Self {
        ComposedSwarmEvent::Gossipsub(event)
    }
}
