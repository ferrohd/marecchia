use std::{iter, collections::HashSet, error::Error};

use async_std::stream::Stream;
use libp2p::{identity::{self, ed25519}, Multiaddr, futures::{channel::{mpsc, oneshot}, SinkExt}, PeerId, request_response::{ProtocolSupport, self, ResponseChannel}, kad::{store::MemoryStore, Kademlia}, Swarm};

use super::{event_loop::{EventLoop, Command, Event}, segment_protocol::*, behaviour::ComposedSwarmBehaviour};

pub async fn new(
    secret_key_seed: Option<u8>,
) -> Result<(Client, impl Stream<Item = Event>, EventLoop), Box<dyn Error>> {
    // Create a public/private key pair, either random or based on a seed.
    let id_keys = match secret_key_seed {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            let secret_key = ed25519::SecretKey::from_bytes(&mut bytes).expect(
                "this returns `Err` only if the length is wrong; the length is correct; qed",
            );
            identity::Keypair::Ed25519(secret_key.into())
        }
        None => identity::Keypair::generate_ed25519(),
    };
    let peer_id = id_keys.public().to_peer_id();
    
    // Build the Swarm, connecting the lower layer transport logic with the
    // higher layer network behaviour logic.
    let swarm = Swarm::with_threadpool_executor(
        libp2p::development_transport(id_keys).await?,
        ComposedSwarmBehaviour {
            kademlia: Kademlia::new(peer_id, MemoryStore::new(peer_id)),
            request_response: request_response::Behaviour::new(
                SegmentExchangeCodec(),
                iter::once((SegmentExchangeProtocol(), ProtocolSupport::Full)),
                Default::default(),
            ),
        },
        peer_id,
    );
    
    let (command_sender, command_receiver) = mpsc::channel(0);
    let (event_sender, event_receiver) = mpsc::channel(0);
    
    Ok((
        Client {
            sender: command_sender,
        },
        event_receiver,
        EventLoop::new(swarm, command_receiver, event_sender),
    ))
}

#[derive(Clone)]
pub struct Client {
    sender: mpsc::Sender<Command>,
}

impl Client {
    /// Listen for incoming connections on the given address.
    pub async fn start_listening(
        &mut self,
        addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
        .send(Command::StartListening { addr, sender })
        .await
        .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }
    
    /// Dial the given peer at the given address.
    pub async fn dial(
        &mut self,
        peer_id: PeerId,
        peer_addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
        .send(Command::Dial {
            peer_id,
            peer_addr,
            sender,
        })
        .await
        .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }
    
    /// Advertise the local node as the provider of the given file on the DHT.
    pub async fn start_providing(&mut self, file_name: String) {
        let (sender, receiver) = oneshot::channel();
        self.sender
        .send(Command::StartProviding { file_name, sender })
        .await
        .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.");
    }
    
    /// Find the providers for the given file on the DHT.
    pub async fn get_providers(&mut self, file_name: String) -> HashSet<PeerId> {
        let (sender, receiver) = oneshot::channel();
        self.sender
        .send(Command::GetProviders { file_name, sender })
        .await
        .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }
    
    /// Request the content of the given file from the given peer.
    pub async fn request_file(
        &mut self,
        peer: PeerId,
        file_name: String,
    ) -> Result<Vec<u8>, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
        .send(Command::RequestFile {
            file_name,
            peer,
            sender,
        })
        .await
        .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not be dropped.")
    }
    
    /// Respond with the provided file content to the given request.
    pub async fn respond_file(
        &mut self,
        file: Vec<u8>,
        channel: ResponseChannel<SegmentResponse>,
    ) {
        self.sender
        .send(Command::RespondFile { file, channel })
        .await
        .expect("Command receiver not to be dropped.");
    }
}