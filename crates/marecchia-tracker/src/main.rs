use libp2p::{
    core::Multiaddr,
    futures::StreamExt,
    identify, identity,
    metrics::{Metrics, Recorder},
    multiaddr::Protocol,
    noise, ping, relay, rendezvous,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
use libp2p_metrics::Registry;
use opentelemetry::{KeyValue, trace::TracerProvider as _};
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::trace::SdkTracerProvider;
use sha3::{Digest, Sha3_512};
use std::{error::Error, net::Ipv4Addr, time::Duration};
use tokio::signal::unix::SignalKind;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_tracing()?;
    let mut metric_registry = Registry::default();

    // Results in PeerID 12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN which is
    // used as the rendezvous point by the other peer examples.
    let keypair = libp2p::identity::Keypair::ed25519_from_bytes([0; 32])?;
    let tracker_id = keypair.public().to_peer_id();

    let digest = Sha3_512::digest(&tracker_id.to_bytes());

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        //.with_tcp(
        //    tcp::Config::default(),
        //    noise::Config::new,
        //    yamux::Config::default,
        //)?
        .with_websocket(
            |key: &_| noise::Config::new(key),
            || yamux::Config::default(),
        )
        .await?
        .with_bandwidth_metrics(&mut metric_registry)
        .with_behaviour(|key| SwarmBehaviour {
            identify: identify::Behaviour::new(identify::Config::new(
                "/marecchia-identify/0.0.1".to_string(),
                key.public(),
            )),
            rendezvous: rendezvous::server::Behaviour::new(rendezvous::server::Config::default()),
            relay: relay::Behaviour::new(tracker_id, relay::Config::default()),
            ping: ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(1))),
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(5)))
        .build();

    let metrics = Metrics::new(&mut metric_registry);

    let listen_addr = Multiaddr::empty()
        .with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
        //.with(Protocol::Ip6(Ipv6Addr::UNSPECIFpIED))
        //.with(Protocol::Tcp(25565))
        .with(Protocol::Udp(25565))
        .with(Protocol::QuicV1);
    //.with(Protocol::WebTransport);
    //.with(Protocol::Certhash(Multihash::<64>::default()))
    //.with(Protocol::P2p(rendezvous_id));

    swarm.listen_on(listen_addr)?;

    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt())?;
    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())?;

    tokio::select! {
        _ = rendezvous_loop(&mut swarm, &metrics) => {}
        _ = sigint.recv() => {
            tracing::info!("Received SIGINT, shutting down...");
        }
        _ = sigterm.recv() => {
            tracing::info!("Received SIGTERM, shutting down...");
        }
    }

    Ok(())
}

async fn rendezvous_loop(swarm: &mut libp2p::Swarm<SwarmBehaviour>, metrics: &Metrics) {
    while let Some(event) = swarm.next().await {
        metrics.record(&event);
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
        ComposedSwarmEvent::Relay(event) => {
            handle_relay_event(swarm, event).await;
        }
        ComposedSwarmEvent::Ping(event) => {
            handle_ping_event(swarm, event).await;
        }
    }
}

async fn handle_identify_event(swarm: &mut libp2p::Swarm<SwarmBehaviour>, event: identify::Event) {
    match event {
        identify::Event::Received {
            peer_id,
            connection_id,
            info,
        } => {
            tracing::info!(
                "Received from connection {}: {} {:?}",
                connection_id,
                peer_id,
                info
            );
        }
        identify::Event::Sent {
            peer_id,
            connection_id,
        } => {
            tracing::info!("Sent from connection {}: {}", connection_id, peer_id);
        }
        identify::Event::Error {
            peer_id,
            connection_id,
            error,
        } => {
            let _ = swarm.disconnect_peer_id(peer_id);
            tracing::info!(
                "Error from connection {}: {} {:?}",
                connection_id,
                peer_id,
                error
            );
        }
        identify::Event::Pushed {
            peer_id,
            connection_id,
            info,
        } => {
            tracing::info!(
                "Pushed to connection {}: {} {:?}",
                connection_id,
                peer_id,
                info
            );
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

async fn handle_relay_event(swarm: &mut libp2p::Swarm<SwarmBehaviour>, event: relay::Event) {
    match event {
        relay::Event::ReservationReqAccepted {
            src_peer_id,
            renewed,
        } => {
            tracing::info!(
                "Reservation request accepted from {} (renewed: {})",
                src_peer_id,
                renewed
            );
        }
        relay::Event::ReservationReqDenied { src_peer_id } => {
            tracing::info!("Reservation request denied from {}", src_peer_id);
        }
        relay::Event::CircuitReqAccepted {
            src_peer_id,
            dst_peer_id,
        } => {
            tracing::info!(
                "Circuit request accepted from {} to {}",
                src_peer_id,
                dst_peer_id
            );
        }
        relay::Event::CircuitReqDenied {
            src_peer_id,
            dst_peer_id,
        } => {
            tracing::info!(
                "Circuit request denied from {} to {}",
                src_peer_id,
                dst_peer_id
            );
        }
        relay::Event::CircuitClosed {
            src_peer_id,
            dst_peer_id,
            error,
        } => {
            tracing::info!(
                "Circuit closed from {} to {}: {:?}",
                src_peer_id,
                dst_peer_id,
                error
            );
        }
        relay::Event::ReservationTimedOut { src_peer_id } => {
            tracing::info!("Reservation timed out from {}", src_peer_id);
        }
        _ => {
            tracing::info!("Received deprecated event: {:?}", event);
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
    relay: relay::Behaviour,
    ping: ping::Behaviour,
}

#[derive(Debug)]
enum ComposedSwarmEvent {
    Identify(identify::Event),
    Rendezvous(rendezvous::server::Event),
    Relay(relay::Event),
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

impl From<relay::Event> for ComposedSwarmEvent {
    fn from(event: relay::Event) -> Self {
        ComposedSwarmEvent::Relay(event)
    }
}

impl From<ping::Event> for ComposedSwarmEvent {
    fn from(event: ping::Event) -> Self {
        ComposedSwarmEvent::Ping(event)
    }
}

fn setup_tracing() -> Result<(), Box<dyn Error>> {
    let resource = opentelemetry_sdk::resource::Resource::builder()
        .with_attribute(KeyValue::new("service.name", "libp2p"))
        .build();
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(SpanExporter::builder().with_tonic().build()?)
        .with_resource(resource)
        .build();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(EnvFilter::from_default_env()))
        .with(
            tracing_opentelemetry::layer()
                .with_tracer(provider.tracer("libp2p-subscriber"))
                .with_filter(EnvFilter::from_default_env()),
        )
        .init();

    Ok(())
}
