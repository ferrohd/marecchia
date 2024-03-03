use libp2p::core::Multiaddr;
use libp2p::futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use libp2p::gossipsub::{self, IdentTopic, Topic};
use libp2p::multiaddr::Protocol;
use libp2p::rendezvous::{client as rendezvous, Cookie, Namespace, Ttl};
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::{ping, PeerId};
use std::collections::{hash_map, HashMap, HashSet};
use std::error::Error;

use super::behaviour::*;

pub struct EventLoop {
    namespace: Namespace,
    cookie: Cookie,
    swarm: Swarm<ComposedSwarmBehaviour>,
    command_receiver: mpsc::Receiver<Command>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
    pending_request_file:
        HashMap<String, oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send>>>>,
}

impl EventLoop {
    pub fn new(
        namespace: Namespace,
        swarm: Swarm<ComposedSwarmBehaviour>,
        command_receiver: mpsc::Receiver<Command>,
    ) -> Self {
        Self {
            cookie: Cookie::for_namespace(namespace.clone()),
            namespace,
            swarm,
            command_receiver,
            pending_dial: Default::default(),
            pending_request_file: Default::default(),
        }
    }

    pub async fn run(mut self) {
        // Register with the rendezvous node.
        let rendezvous_node = PeerId::random();
        match self.swarm.behaviour_mut().rendezvous.register(
            self.namespace.clone(),
            rendezvous_node,
            Some(60),
        ) {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Failed to register with rendezvous node: {:?}", e);
                return;
            }
        }

        loop {
            libp2p::futures::select! {
                event = self.swarm.next() => self.handle_event(event.expect("Swarm stream to be infinite.")).await,
                command = self.command_receiver.next() => match command {
                    Some(c) => self.handle_command(c).await,
                    // Command channel closed, thus shutting down the network event loop.
                    None=>  return,
                },
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<ComposedSwarmEvent>) {
        match event {
            SwarmEvent::Behaviour(behaviour) => self.handle_behaviour_event(behaviour).await,
            SwarmEvent::NewListenAddr { address, .. } => {
                // Started listening on a new address.
                tracing::info!("Local node is listening on {:?}", address);
            }
            SwarmEvent::IncomingConnection {
                connection_id,
                local_addr,
                send_back_addr,
            } => {
                // A new incoming connection has been established.
                tracing::info!(
                    "Incoming connection from {:?} with conn_id {:?}",
                    local_addr,
                    connection_id
                );
            }
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                // A new connection has been established. (Either incoming or outgoing)
                tracing::info!("Connection established with peer {:?}", peer_id);
                if endpoint.is_dialer() {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::ConnectionClosed {
                connection_id,
                peer_id,
                num_established,
                endpoint,
                cause,
            } => {
                // A connection has been closed.
                tracing::info!("Connection {:?} closed  on endpoint {:?} with peer {:?} with error {:?}, {:?} connections remaining", connection_id, endpoint, peer_id, cause, num_established);
            }
            SwarmEvent::OutgoingConnectionError {
                connection_id,
                peer_id,
                error,
                ..
            } => {
                // An outgoing connection has failed.
                tracing::error!(
                    "Outgoing connection {:?} to peer {:?} failed with error {:?}",
                    connection_id,
                    peer_id,
                    error
                );
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Err(Box::new(error)));
                    }
                }
            }
            SwarmEvent::IncomingConnectionError {
                connection_id,
                local_addr,
                send_back_addr,
                error,
            } => {
                // An incoming connection has failed during initial handshake.
                tracing::error!(
                    "Incoming  handshake connection {:?} on {:?} from {:?} failed with error {:?}",
                    connection_id,
                    local_addr,
                    send_back_addr,
                    error
                );
            }
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => println!(
                "Dialing user {:?} with connection id {:?} ",
                connection_id, peer_id
            ),
            SwarmEvent::ExpiredListenAddr {
                listener_id: _listener,
                address,
            } => {
                // A listening address has expired.
                tracing::warn!("Listening address {:?} expired", address);
            }
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } => {
                // A listener has been closed.
                tracing::info!(
                    "Listener {:?} on address: {:?} closed with reason {:?}",
                    listener_id,
                    addresses,
                    reason
                );
            }
            SwarmEvent::ListenerError { listener_id, error } => {
                // A listener has encountered a non fatal error.
                tracing::warn!(
                    "Listener {:?} encountered a non fatal error {:?}",
                    listener_id,
                    error
                );
            }
            // TODO: handle the rest
            _ => {}
        }
    }

    async fn handle_behaviour_event(&mut self, event: ComposedSwarmEvent) {
        match event {
            ComposedSwarmEvent::Ping(event) => self.handle_ping_event(event).await,
            ComposedSwarmEvent::Rendezvous(event) => self.handle_rendezvous_event(event).await,
            ComposedSwarmEvent::Gossipsub(event) => self.handle_gossipsub_event(event).await,
        }
    }

    async fn handle_ping_event(&mut self, ping_event: ping::Event) {
        match ping_event.result {
            Ok(duration) => {
                // The ping was successful.
                tracing::debug!(
                    "Ping to peer {:?} successful, {:?}",
                    ping_event.peer,
                    duration
                )
            }
            Err(failure) => {
                // Disconnect from the peer.
                tracing::warn!(
                    "Ping to peer {:?} failed, disconnecting. Reason: {:?}",
                    ping_event.peer,
                    failure
                );
                let _ = self.swarm.disconnect_peer_id(ping_event.peer);
            }
        }
    }

    async fn handle_rendezvous_event(&mut self, rendezvous_event: rendezvous::Event) {
        match rendezvous_event {
            rendezvous::Event::Discovered {
                rendezvous_node,
                registrations,
                cookie,
            } => {
                // Discovered peers from the rendezvous node.
                tracing::info!(
                    "Discovered {:?} peers from rendezvous node {:?}",
                    registrations.len(),
                    rendezvous_node
                );
                // Update cookie (next requets avoid dsicovering the same peers again)
                self.cookie = cookie;
                for registration in registrations {
                    for address in registration.record.addresses() {
                        let peer = registration.record.peer_id();
                        tracing::info!("Dialing peer {:?} with address {:?}", peer, address);

                        let p2p_suffix = Protocol::P2p(peer);
                        let address_with_p2p =
                            if !address.ends_with(&Multiaddr::empty().with(p2p_suffix.clone())) {
                                address.clone().with(p2p_suffix)
                            } else {
                                address.clone()
                            };

                        self.swarm.dial(address_with_p2p).unwrap();
                    }
                }
            }
            rendezvous::Event::DiscoverFailed {
                rendezvous_node,
                namespace,
                error,
            } => {
                // Failed to discover peers from the rendezvous node.
                tracing::error!(
                    "Failed to discover peers from rendezvous node {:?} in namespace {:?} with error {:?}",
                    rendezvous_node,
                    namespace,
                    error
                );
            }
            rendezvous::Event::Expired { peer } => {
                // Peer registration with the rendezvous node has expired. ()
                tracing::info!("Connection details we learned from node {:?} expired", peer);
            }
            rendezvous::Event::Registered {
                rendezvous_node,
                ttl,
                namespace,
            } => {
                // Registered with the rendezvous node.
                tracing::info!(
                    "Registered with rendezvous node {:?} in namespace {:?} with ttl {:?}",
                    rendezvous_node,
                    namespace,
                    ttl
                );
            }
            rendezvous::Event::RegisterFailed {
                rendezvous_node,
                namespace,
                error,
            } => {
                // Failed to register with the rendezvous node.
                tracing::error!(
                    "Failed to register with rendezvous node {:?} in namespace {:?} with error {:?}",
                    rendezvous_node,
                    namespace,
                    error
                );
            }
        }
    }

    async fn handle_gossipsub_event(&mut self, event: gossipsub::Event) {
        match event {
            gossipsub::Event::Message {
                propagation_source,
                message_id,
                message,
            } => {
                // A new message has been received.
                tracing::info!(
                    "Received message {:?} from {:?} with topic {:?}",
                    message_id,
                    propagation_source,
                    message.topic.as_str()
                );
            }
            gossipsub::Event::Subscribed { peer_id, topic } => {
                // A remote subscribed to a new topic.
                tracing::info!("Remote peer {:?} subscribed to topic {:?}", peer_id, topic);
            }
            gossipsub::Event::Unsubscribed { peer_id, topic } => {
                // A remote unsubscribed from a topic.
                tracing::info!(
                    "Remote peer {:?} unsubscribed from topic {:?}",
                    peer_id,
                    topic
                );
            }
            gossipsub::Event::GossipsubNotSupported { peer_id } => {
                // A remote peer does not support gossipsub.
                tracing::warn!(
                    "Remote peer {:?} connected but does not support gossipsub",
                    peer_id
                );
            }
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::StartListening { addr, sender } => match self.swarm.listen_on(addr) {
                Ok(_) => {
                    let _ = sender.send(Ok(()));
                }
                Err(e) => {
                    let _ = sender.send(Err(Box::new(e)));
                }
            },
            Command::Dial {
                peer_id,
                peer_addr,
                sender,
            } => {
                // If not already dialing, dial the peer.
                if let hash_map::Entry::Vacant(e) = self.pending_dial.entry(peer_id) {
                    match self
                        .swarm
                        .dial(peer_addr.with(Protocol::P2p(peer_id.into())))
                    {
                        Ok(()) => {
                            e.insert(sender);
                        }
                        Err(e) => {
                            let _ = sender.send(Err(Box::new(e)));
                        }
                    }
                }
            }
            Command::ProvideSegment { segment_id, data } => {
                let topic = IdentTopic::new(segment_id);
                self.swarm.behaviour_mut().pubsub.publish(topic, data);
            }
            Command::RequestSegment { segment_id, sender } => {
                let topic = IdentTopic::new(segment_id);
                let _ = self.swarm.behaviour_mut().pubsub.subscribe(&topic);
                self.pending_request_file.insert(topic.to_string(), sender);
            }
        }
    }
}

#[derive(Debug)]
pub enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    Dial {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    ProvideSegment {
        segment_id: String,
        data: Vec<u8>,
    },
    RequestSegment {
        segment_id: String,
        sender: oneshot::Sender<Result<Vec<u8>, Box<dyn Error + Send>>>,
    },
}
