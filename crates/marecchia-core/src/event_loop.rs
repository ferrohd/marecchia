use libp2p::core::Multiaddr;
use libp2p::futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use libp2p::gossipsub::{self, IdentTopic, SubscriptionError};
use libp2p::multiaddr::Protocol;
use libp2p::rendezvous::{client as rendezvous, Cookie, Namespace};
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::{identify, ping, relay, PeerId};
use std::collections::{HashMap, VecDeque};
use wasm_bindgen::JsError;

use super::behaviour::*;

pub struct EventLoop {
    namespace: Namespace,
    cookie: Cookie,
    swarm: Swarm<ComposedSwarmBehaviour>,
    command_receiver: mpsc::Receiver<Command>,
    segment_request: SegmentRequestCache,
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
            segment_request: SegmentRequestCache::new(10),
        }
    }

    pub async fn run(mut self) {
        // Register with the rendezvous node.
        // TODO: move the rendevouz connection in the client.rs file
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
                event = self.swarm.next() => match event {
                    Some(e) => self.handle_event(e).await,
                    None => return,
                },
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
                send_back_addr: _,
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
                tracing::info!(
                    "Connection established with peer {:?} as {:?}",
                    peer_id,
                    endpoint.to_endpoint()
                );
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
            ComposedSwarmEvent::Identify(event) => self.handle_identify_event(event).await,
            ComposedSwarmEvent::Rendezvous(event) => self.handle_rendezvous_event(event).await,
            ComposedSwarmEvent::Relay(event) => self.handle_relay_event(event).await,
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

    async fn handle_identify_event(&mut self, identify_event: identify::Event) {
        match identify_event {
            identify::Event::Received {
                peer_id,
                info,
                connection_id,
            } => {
                // Received identification information from a peer.
                tracing::info!(
                    "Conn: {}, Received identification information from peer {:?} with info {:?}",
                    connection_id,
                    peer_id,
                    info,
                );
            }
            identify::Event::Sent {
                peer_id,
                connection_id,
            } => {
                // Sent identification information to a peer in response to a request.
                tracing::info!(
                    "Conn {}, Sent identification information to peer {:?}",
                    connection_id,
                    peer_id
                );
            }
            identify::Event::Pushed {
                peer_id,
                info,
                connection_id,
            } => {
                //Identification information of the local node has been actively pushed to a peer
                tracing::info!(
                    "Con {}, Pushed identification information to peer {:?} with info {:?}",
                    connection_id,
                    peer_id,
                    info
                );
            }
            identify::Event::Error {
                peer_id,
                error,
                connection_id,
            } => {
                // Failed to send identification information to a peer.
                tracing::error!(
                    "Conn {}, Failed to send identification information to peer {:?} with error {:?}",
                    connection_id,
                    peer_id,
                    error
                );
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

    async fn handle_relay_event(&mut self, event: relay::client::Event) {
        match event {
            relay::client::Event::ReservationReqAccepted {
                relay_peer_id,
                renewal,
                limit,
            } => {
                // A reservation request has been accepted.
                tracing::info!(
                    "Reservation request accepted for relay {:?} with renewal {:?} and limit {:?}",
                    relay_peer_id,
                    renewal,
                    limit
                );
            }
            relay::client::Event::InboundCircuitEstablished { src_peer_id, limit } => {
                // An inbound circuit has been established.
                tracing::info!(
                    "Inbound circuit established with peer {:?} with limit {:?}",
                    src_peer_id,
                    limit
                );
            }
            relay::client::Event::OutboundCircuitEstablished {
                relay_peer_id,
                limit,
            } => {
                // An outbound circuit has been established.
                tracing::info!(
                    "Outbound circuit established with relay {:?} with limit {:?}",
                    relay_peer_id,
                    limit
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
                if let Some(sender) = self.segment_request.remove(message.topic.as_str()) {
                    let _ = sender.send(Ok(message.data));
                }
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
                    "Remote peer {:?} connected but does not support gossipsub, disconnecting",
                    peer_id
                );
                let _ = self.swarm.disconnect_peer_id(peer_id);
            }
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Dial {
                peer_id,
                peer_addr,
                sender,
            } => {
                tracing::info!("Dialing peer {:?} with addr {:?}", peer_id, peer_addr);
                match self.swarm.dial(peer_addr.with(Protocol::P2p(peer_id))) {
                    Ok(_) => {
                        tracing::info!("Successfully dialed peer {:?}", peer_id);
                        let _ = sender.send(Ok(()));
                    }
                    Err(e) => {
                        tracing::error!("Failed to dial peer {:?} with error {:?}", peer_id, e);
                        let _ = sender.send(Err(e.into()));
                    }
                }
            }
            Command::ProvideSegment { segment_id, data } => {
                let segment_id_clone = segment_id.clone();
                let topic = IdentTopic::new(segment_id);
                match self.swarm.behaviour_mut().pubsub.publish(topic, data) {
                    Ok(message_id) => {
                        tracing::info!(
                            "Published segment {:?} with message {:?}",
                            segment_id_clone,
                            message_id
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to publish segment {:?} with error {:?}",
                            segment_id_clone,
                            e
                        );
                    }
                }
            }
            Command::RequestSegment { segment_id, sender } => {
                let segment_id_clone = segment_id.clone();
                let topic = IdentTopic::new(segment_id);
                match self.swarm.behaviour_mut().pubsub.subscribe(&topic) {
                    Ok(_) => {
                        tracing::info!("Subscribed to topic {:?}", topic);
                        self.segment_request.insert(segment_id_clone, sender);
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to subscribe to topic {:?} with error {:?}",
                            topic,
                            e
                        );
                        let _ = sender.send(Err(e.into()));
                    }
                }
            }
            Command::Quit => {
                tracing::info!("Shutting down the network event loop");
            }
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Dial {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        sender: oneshot::Sender<Result<(), JsError>>,
    },
    ProvideSegment {
        segment_id: String,
        data: Vec<u8>,
    },
    RequestSegment {
        segment_id: String,
        sender: oneshot::Sender<Result<Vec<u8>, RequestError>>,
    },
    Quit,
}

#[derive(Debug)]
pub enum RequestError {
    Timeout,
    SubscribeError(SubscriptionError),
}

impl From<RequestError> for JsError {
    fn from(error: RequestError) -> Self {
        match error {
            RequestError::Timeout => JsError::new("Request timed out"),
            RequestError::SubscribeError(e) => {
                JsError::new(&format!("Subscription error: {:?}", e))
            }
        }
    }
}

impl From<SubscriptionError> for RequestError {
    fn from(error: SubscriptionError) -> Self {
        RequestError::SubscribeError(error)
    }
}

pub struct SegmentRequestCache {
    requests: HashMap<String, oneshot::Sender<Result<Vec<u8>, RequestError>>>,
    order: VecDeque<String>,
    capacity: usize,
}

impl SegmentRequestCache {
    pub fn new(capacity: usize) -> Self {
        SegmentRequestCache {
            requests: HashMap::new(),
            order: VecDeque::new(),
            capacity,
        }
    }

    pub fn insert(&mut self, key: String, value: oneshot::Sender<Result<Vec<u8>, RequestError>>) {
        // Perform the insertion
        self.requests.insert(key.clone(), value);
        self.order.push_back(key.clone());

        // Check for capacity overflow and remove the oldest item if necessary
        if self.order.len() > self.capacity {
            if let Some(oldest_key) = self.order.pop_front() {
                if let Some(channel) = self.requests.remove(&oldest_key) {
                    // The channel has been popped, return the Timeout Erro
                    let _ = channel.send(Err(RequestError::Timeout));
                }
            }
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<oneshot::Sender<Result<Vec<u8>, RequestError>>> {
        if let Some(sender) = self.requests.remove(key) {
            self.order.retain(|k| k != key); // Remove the key from the order tracking
            return Some(sender);
        }
        None
    }
}
