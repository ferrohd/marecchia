use libp2p::{
    futures::StreamExt,
    identify,
    multiaddr::Protocol,
    noise, ping, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr,
};
use libp2p_webrtc as webrtc;
use tokio::signal::unix::SignalKind;

use std::time::Duration;
use std::{
    error::Error,
    net::{Ipv4Addr, Ipv6Addr},
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    // Results in PeerID 12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN which is
    // used as the rendezvous point by the other peer examples.
    let keypair = libp2p::identity::Keypair::ed25519_from_bytes([0; 32])?;
    let rendezvous_id = keypair.public().to_peer_id();

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| SwarmBehaviour {
            identify: identify::Behaviour::new(identify::Config::new(
                "rendezvous-example/1.0.0".to_string(),
                key.public(),
            )),
            rendezvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(1))),
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(5)))
        .build();

    let listen_addr = Multiaddr::empty()
        .with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
        .with(Protocol::Ip6(Ipv6Addr::UNSPECIFIED))
        .with(Protocol::Udp(0))
        .with(Protocol::P2p(rendezvous_id));

    swarm.listen_on(listen_addr)?;

    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt())?;
    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())?;

    tokio::select! {
        _ = rendezvous_loop(&mut swarm) => {}
        _ = sigint.recv() => {
            tracing::info!("Received SIGINT, shutting down...");
        }
        _ = sigterm.recv() => {
            tracing::info!("Received SIGTERM, shutting down...");
        }
    }

    Ok(())
}

async fn rendezvous_loop(swarm: &mut libp2p::Swarm<SwarmBehaviour>) {
    while let Some(event) = swarm.next().await {
        match event {
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                tracing::info!("Connected to {}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                tracing::info!("Disconnected from {}", peer_id);
            }
            SwarmEvent::Behaviour(event) => {
                handle_behaviour_event(swarm, event).await;
            }
            other => {
                tracing::debug!("Unhandled {:?}", other);
            }
        }
    }
}

async fn handle_behaviour_event(
    swarm: &mut libp2p::Swarm<SwarmBehaviour>,
    event: ComposedSwarmEvent,
) {
    match event {
        ComposedSwarmEvent::Identify(event) => {
            handle_identify_event(swarm, event).await;
        }
        ComposedSwarmEvent::Rendezvous(event) => {
            handle_rendezvous_event(swarm, event).await;
        }
        ComposedSwarmEvent::Ping(event) => {
            handle_ping_event(swarm, event).await;
        }
    }
}

async fn handle_identify_event(swarm: &mut libp2p::Swarm<SwarmBehaviour>, event: identify::Event) {
    match event {
        identify::Event::Received { peer_id, info } => {
            tracing::info!("Received: {} {:?}", peer_id, info);
        }
        identify::Event::Sent { peer_id } => {
            tracing::info!("Sent: {}", peer_id);
        }
        identify::Event::Error { peer_id, error } => {
            let _ = swarm.disconnect_peer_id(peer_id);
            tracing::info!("Error: {} {:?}", peer_id, error);
        }
        identify::Event::Pushed { peer_id, info } => {
            tracing::info!("Pushed: {} {:?}", peer_id, info);
        }
    }
}

async fn handle_rendezvous_event(
    _swarm: &mut libp2p::Swarm<SwarmBehaviour>,
    event: rendezvous::server::Event,
) {
    match event {
        rendezvous::server::Event::PeerRegistered { peer, registration } => {
            tracing::info!(
                "Peer {} registered for namespace '{}'",
                peer,
                registration.namespace
            );
        }
        rendezvous::server::Event::PeerNotRegistered {
            peer,
            namespace,
            error,
        } => {
            tracing::info!(
                "Failed to register peer {} for {}: {:?}",
                peer,
                namespace,
                error
            );
        }
        rendezvous::server::Event::PeerUnregistered { peer, namespace } => {
            tracing::info!("Peer {} unregistered from namespace '{}'", peer, namespace);
        }
        rendezvous::server::Event::DiscoverServed {
            enquirer,
            registrations,
        } => {
            tracing::info!(
                "Served peer {} with {} registrations",
                enquirer,
                registrations.len()
            );
        }
        rendezvous::server::Event::DiscoverNotServed { enquirer, error } => {
            tracing::info!("Failed to serve peer {}: {:?}", enquirer, error);
        }
        rendezvous::server::Event::RegistrationExpired(registration) => {
            tracing::info!("Registration expired: {:?}", registration);
        }
    }
}

async fn handle_ping_event(swarm: &mut libp2p::Swarm<SwarmBehaviour>, event: ping::Event) {
    match event.result {
        Ok(duration) => {
            tracing::info!("Ping: {}ms", duration.as_millis());
        }
        Err(error) => {
            let _ = swarm.disconnect_peer_id(event.peer);
            tracing::info!("Ping: {:?}", error);
        }
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "ComposedSwarmEvent")]
struct SwarmBehaviour {
    identify: identify::Behaviour,
    rendezvous: rendezvous::server::Behaviour,
    ping: ping::Behaviour,
}

#[derive(Debug)]
enum ComposedSwarmEvent {
    Identify(identify::Event),
    Rendezvous(rendezvous::server::Event),
    Ping(ping::Event),
}

impl From<identify::Event> for ComposedSwarmEvent {
    fn from(event: identify::Event) -> Self {
        ComposedSwarmEvent::Identify(event)
    }
}

impl From<rendezvous::server::Event> for ComposedSwarmEvent {
    fn from(event: rendezvous::server::Event) -> Self {
        ComposedSwarmEvent::Rendezvous(event)
    }
}

impl From<ping::Event> for ComposedSwarmEvent {
    fn from(event: ping::Event) -> Self {
        ComposedSwarmEvent::Ping(event)
    }
}
