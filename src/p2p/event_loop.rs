use libp2p::core::Multiaddr;
use libp2p::futures::{
    channel::{mpsc, oneshot},
    prelude::*,
};
use libp2p::kad::{self, GetProvidersError, GetProvidersOk, QueryId, QueryResult, RecordKey};
use libp2p::multiaddr::Protocol;
use libp2p::request_response::{
    self, Event as RequestResponseEvent, OutboundRequestId, ResponseChannel,
};
use libp2p::swarm::{Swarm, SwarmEvent};
use libp2p::{ping, PeerId};
use std::collections::{hash_map, HashMap, HashSet};
use std::error::Error;

use super::behaviour::*;

pub struct EventLoop {
    swarm: Swarm<ComposedSwarmBehaviour>,
    command_receiver: mpsc::Receiver<LoopCommand>,
    event_sender: mpsc::Sender<LoopEvent>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
    pending_start_providing: HashMap<QueryId, oneshot::Sender<()>>,
    pending_get_providers: HashMap<QueryId, oneshot::Sender<HashSet<PeerId>>>,
    pending_request_file:
        HashMap<OutboundRequestId, oneshot::Sender<Result<Option<Vec<u8>>, Box<dyn Error + Send>>>>,
}

impl EventLoop {
    pub fn new(
        swarm: Swarm<ComposedSwarmBehaviour>,
        command_receiver: mpsc::Receiver<LoopCommand>,
        event_sender: mpsc::Sender<LoopEvent>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            event_sender,
            pending_dial: Default::default(),
            pending_start_providing: Default::default(),
            pending_get_providers: Default::default(),
            pending_request_file: Default::default(),
        }
    }

    pub async fn run(mut self) {
        loop {
            libp2p::futures::select! {
                event = self.swarm.next() => self.handle_event(event.expect("Swarm stream to be infinite.")).await  ,
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
                let local_peer_id = *self.swarm.local_peer_id();
                eprintln!(
                    "Local node is listening on {:?}",
                    address.with(Protocol::P2p(local_peer_id.into()))
                );
            }
            SwarmEvent::IncomingConnection { .. } => {}
            SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                if endpoint.is_dialer() {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Ok(()));
                    }
                }
            }
            SwarmEvent::ConnectionClosed { .. } => {}
            SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                if let Some(peer_id) = peer_id {
                    if let Some(sender) = self.pending_dial.remove(&peer_id) {
                        let _ = sender.send(Err(Box::new(error)));
                    }
                }
            }
            SwarmEvent::IncomingConnectionError { .. } => {}
            SwarmEvent::Dialing {
                peer_id,
                connection_id,
            } => println!(
                "Dialing user with conn_id {:?} and peer_id {:?}",
                connection_id,
                peer_id.unwrap()
            ),
            SwarmEvent::ExpiredListenAddr { .. } => {}
            SwarmEvent::ListenerClosed {
                listener_id,
                addresses,
                reason,
            } => {}
            SwarmEvent::ListenerError { listener_id, error } => {}
            // TODO: handle the rest
            _ => {}
        }
    }

    async fn handle_behaviour_event(&mut self, behaviour_event: ComposedSwarmEvent) {
        match behaviour_event {
            ComposedSwarmEvent::Ping(event) => self.handle_ping_event(event).await,
            ComposedSwarmEvent::Kademlia(event) => self.handle_kademlia_event(event).await,
            ComposedSwarmEvent::RequestResponse(event) => self.handle_segment_rr_event(event).await,
        }
    }

    async fn handle_ping_event(&mut self, ping_event: ping::Event) {}

    async fn handle_kademlia_event(&mut self, kaemlia_event: kad::Event) {
        match kaemlia_event {
            kad::Event::ModeChanged { new_mode } => {}
            kad::Event::RoutingUpdated { peer, .. } => {}
            kad::Event::UnroutablePeer { peer } => {}
            kad::Event::PendingRoutablePeer { peer, address } => {}
            kad::Event::RoutablePeer { peer, address } => {}
            kad::Event::InboundRequest { request } => {}
            kad::Event::OutboundQueryProgressed {
                id,
                result,
                stats,
                step,
            } => {
                match result {
                    QueryResult::StartProviding(_) => {
                        // Start providing the segment. Emit event.
                        let sender: oneshot::Sender<()> = self
                            .pending_start_providing
                            .remove(&id)
                            .expect("Completed query to be previously pending.");
                        let _ = sender.send(());
                    }
                    QueryResult::GetProviders(result) => {
                        match result {
                            Ok(providers_ok) => {
                                match providers_ok {
                                    GetProvidersOk::FoundProviders { providers, key } => {
                                        // Found providers of the segment. Emit event.
                                        if let Some(sender) = self.pending_get_providers.remove(&id)
                                        {
                                            sender
                                                .send(providers)
                                                .expect("Receiver not to be dropped");
                                            // Finish the query. We are only interested in the first result. Tell the swarm to stop querying.
                                            self.swarm
                                                .behaviour_mut()
                                                .kademlia
                                                .query_mut(&id)
                                                .unwrap()
                                                .finish();
                                        }
                                    }
                                    GetProvidersOk::FinishedWithNoAdditionalRecord {
                                        closest_peers,
                                    } => {
                                        // No providers of the segment found.
                                        // ! Start downloading from the CDN.
                                        // ? Should we try to find providers of the next segment?
                                    }
                                }
                            }
                            Err(providers_err) => match providers_err {
                                GetProvidersError::Timeout { key, closest_peers } => {
                                    // The query of the segment timed out.
                                    // ? Should we use the timeout to force a threshold within the segment must be found?
                                    // ! Start downloading from the CDN.
                                }
                            },
                        }
                    }
                    // The Kademlia DHT is used to find owners of a segment.
                    // The segment is not stored in the DHT. The value of a key is never accessed.
                    QueryResult::Bootstrap(result) => {}
                    QueryResult::GetRecord(result) => {}
                    QueryResult::PutRecord(result) => {}
                    QueryResult::GetClosestPeers(result) => {}
                    _ => {} // Ignore events from automatic queries.
                }
            }
        }
    }
    async fn handle_segment_rr_event(
        &mut self,
        request_response_event: RequestResponseEvent<SegmentRequest, SegmentResponse>,
    ) {
        match request_response_event {
            RequestResponseEvent::Message { message, .. } => match message {
                request_response::Message::Request {
                    request, channel, ..
                } => {
                    // Received a segment request, emit it.
                    self.event_sender
                        .send(LoopEvent::SegmentRequest {
                            segment_id: request.0,
                            channel,
                        })
                        .await
                        .expect("Event receiver not to be dropped.");
                }
                request_response::Message::Response {
                    request_id,
                    response,
                } => {
                    // Received a segment response, remove from the pending requests and emit it. (Enjoy your mojito)
                    let _ = self
                        .pending_request_file
                        .remove(&request_id)
                        .expect("Request to still be pending.")
                        .send(Ok(response.0));
                }
            },
            RequestResponseEvent::OutboundFailure {
                request_id, error, ..
            } => {
                // The request failed, remove from the pending requests and emit an error.
                let _ = self
                    .pending_request_file
                    .remove(&request_id)
                    .expect("Request to still be pending.")
                    .send(Err(Box::new(error)));
            }
            RequestResponseEvent::InboundFailure {
                peer,
                request_id,
                error,
            } => {
                // ! Download from CDN
            }
            RequestResponseEvent::ResponseSent { .. } => {}
        }
    }

    async fn handle_command(&mut self, command: LoopCommand) {
        match command {
            LoopCommand::Dial {
                peer_id,
                peer_addr,
                sender,
            } => {
                // If not already dialing, dial the peer.
                if let hash_map::Entry::Vacant(e) = self.pending_dial.entry(peer_id) {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, peer_addr.clone());
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
            LoopCommand::StartProviding { segment_id, sender } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .start_providing(segment_id.into_bytes().into())
                    .expect("No store error.");
                self.pending_start_providing.insert(query_id, sender);
            }
            LoopCommand::StopProviding { segment_id, sender } => {
                let key: RecordKey = segment_id.into_bytes().into();
                self.swarm.behaviour_mut().kademlia.stop_providing(&key);
                let _ = sender.send(());
            }
            LoopCommand::GetProviders { segment_id, sender } => {
                let query_id = self
                    .swarm
                    .behaviour_mut()
                    .kademlia
                    .get_providers(segment_id.into_bytes().into());
                self.pending_get_providers.insert(query_id, sender);
            }
            LoopCommand::RequestSegment {
                segment_id: file_name,
                peer,
                sender,
            } => {
                let request_id = self
                    .swarm
                    .behaviour_mut()
                    .segment_rr
                    .send_request(&peer, SegmentRequest(file_name));
                self.pending_request_file.insert(request_id, sender);
            }
            LoopCommand::RespondSegment {
                segment_data,
                channel,
            } => {
                self.swarm
                    .behaviour_mut()
                    .segment_rr
                    .send_response(channel, SegmentResponse(segment_data))
                    .expect("Connection to peer to be still open.");
            }
        }
    }
}

#[derive(Debug)]
pub enum LoopCommand {
    Dial {
        peer_id: PeerId,
        peer_addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    StartProviding {
        segment_id: String,
        sender: oneshot::Sender<()>,
    },
    StopProviding {
        segment_id: String,
        sender: oneshot::Sender<()>,
    },
    GetProviders {
        segment_id: String,
        sender: oneshot::Sender<HashSet<PeerId>>,
    },
    RequestSegment {
        segment_id: String,
        peer: PeerId,
        sender: oneshot::Sender<Result<Option<Vec<u8>>, Box<dyn Error + Send>>>,
    },
    RespondSegment {
        segment_data: Option<Vec<u8>>,
        channel: ResponseChannel<SegmentResponse>,
    },
}

#[derive(Debug)]
pub enum LoopEvent {
    SegmentRequest {
        segment_id: String,
        channel: ResponseChannel<SegmentResponse>,
    },
}
