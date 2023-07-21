use std::{collections::HashSet, error::Error, io, iter, time::Duration};

use async_std::stream::Stream;
use libp2p::{
    autonat,
    core::{self, muxing::StreamMuxerBox, transport::Boxed},
    dns,
    futures::{
        channel::{mpsc, oneshot},
        SinkExt,
    },
    identity::{self, Keypair},
    kad, noise, ping,
    request_response::{self, ProtocolSupport, ResponseChannel},
    swarm, tcp, yamux, Multiaddr, PeerId, Transport,
};

use super::{
    behaviour::ComposedSwarmBehaviour,
    event_loop::{EventLoop, LoopCommand, LoopEvent},
    segment_protocol::{self, SegmentResponse},
};

pub async fn new(
    secret_key_seed: Option<u8>,
) -> Result<(Client, impl Stream<Item = LoopEvent>, EventLoop), Box<dyn Error>> {
    // Create a public/private key pair, either random or based on a seed.
    let keypair = match secret_key_seed {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            identity::Keypair::ed25519_from_bytes(&mut bytes).expect(
                "this returns `Err` only if the length is wrong; the length is correct; qed",
            )
        }
        None => identity::Keypair::generate_ed25519(),
    };
    let peer_id = keypair.public().to_peer_id();

    let transport = create_transport(&keypair)
        .await
        .expect("Cannot create transport");
    // Build the Swarm, connecting the lower layer transport logic with the
    // higher layer network behaviour logic.
    let swarm = {
        // Define the various behaviours of the swarm.
        let ping_config = ping::Config::new()
            .with_timeout(Duration::from_secs(5))
            .with_interval(Duration::from_secs(5));
        let ping = ping::Behaviour::new(ping_config);

        let autonat_config = autonat::Config::default();
        let autonat = autonat::Behaviour::new(peer_id, autonat_config);

        let kademlia_config = kad::KademliaConfig::default()
            .set_connection_idle_timeout(Duration::from_secs(60))
            .set_provider_publication_interval(Some(Duration::from_secs(30)))
            .set_provider_record_ttl(None)
            .set_publication_interval(Some(Duration::from_secs(30)))
            .set_record_ttl(Some(Duration::from_secs(60)))
            .set_replication_interval(Some(Duration::from_secs(5)))
            .to_owned();
        let kademlia_store = kad::store::MemoryStore::new(peer_id);
        let kademlia = kad::Kademlia::with_config(peer_id, kademlia_store, kademlia_config);

        // Not needed, SegmentExchangeProtocol implements Codec and Default, gets instantiated in request_response::Behaviour::new
        //let rr_codec = segment_protocol::SegmentExchangeCodec();
        let segment_protocol = segment_protocol::SegmentExchangeProtocol();
        let request_response = request_response::Behaviour::new(
            [(segment_protocol, ProtocolSupport::Full)],
            request_response::Config::default(),
        );

        let behaviour = ComposedSwarmBehaviour {
            ping,
            //autonat,
            kademlia,
            segment_rr: request_response,
        };

        // Do wthings with behaviours

        swarm::SwarmBuilder::with_wasm_executor(transport, behaviour, peer_id)
            .max_negotiating_inbound_streams(128)
            .build()
    };

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
    sender: mpsc::Sender<LoopCommand>,
}

impl Client {
    /// Listen for incoming connections on the given address.
    pub async fn start_listening(&mut self, addr: Multiaddr) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(LoopCommand::StartListening { addr, sender })
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
            .send(LoopCommand::Dial {
                peer_id,
                peer_addr,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Advertise the local node as the provider of the given file on the DHT.
    pub async fn start_providing(&mut self, segment_id: String) {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(LoopCommand::StartProviding { segment_id, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.");
    }

    /// Find the providers for the given file on the DHT.
    pub async fn get_providers(&mut self, segment_id: String) -> HashSet<PeerId> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(LoopCommand::GetProviders { segment_id, sender })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Request the content of the given file from the given peer.
    pub async fn request_segment(
        &mut self,
        peer: PeerId,
        segment_id: String,
    ) -> Result<Option<Vec<u8>>, Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();
        self.sender
            .send(LoopCommand::RequestSegment {
                segment_id,
                peer,
                sender,
            })
            .await
            .expect("Command receiver not to be dropped.");
        receiver.await.expect("Sender not be dropped.")
    }

    /// Respond with the provided file content to the given request.
    pub async fn respond_segment(
        &mut self,
        file: Option<Vec<u8>>,
        channel: ResponseChannel<SegmentResponse>,
    ) {
        self.sender
            .send(LoopCommand::RespondSegment {
                segment_data: file,
                channel,
            })
            .await
            .expect("Command receiver not to be dropped.");
    }
}

async fn create_transport(keypair: &Keypair) -> Result<Boxed<(PeerId, StreamMuxerBox)>, io::Error> {
    let dns_tcp =
        dns::DnsConfig::system(tcp::tokio::Transport::new(tcp::Config::new().nodelay(true)))
            .await?
            .upgrade(core::upgrade::Version::V1)
            .authenticate(noise::Config::new(&keypair).unwrap())
            .multiplex(yamux::Config::default())
            .timeout(Duration::from_secs(20))
            .boxed();
    Ok(dns_tcp)
}
